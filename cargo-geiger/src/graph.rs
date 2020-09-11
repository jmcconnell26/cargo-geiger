use cargo::core::dependency::DepKind;
use cargo::core::package::PackageSet;
use cargo::core::{Dependency, PackageId, Resolve};
use cargo::util::CargoResult;
use cargo_metadata::{DependencyKind, Metadata};
use cargo_platform::Cfg;
use petgraph::graph::NodeIndex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone)]
pub enum ExtraDeps {
    All,
    Build,
    Dev,
    NoMore,
}

impl ExtraDeps {
    pub fn allows(&self, dep: DepKind) -> bool {
        match (self, dep) {
            (_, DepKind::Normal) => true,
            (ExtraDeps::All, _) => true,
            (ExtraDeps::Build, DepKind::Build) => true,
            (ExtraDeps::Dev, DepKind::Development) => true,
            _ => false,
        }
    }
}

/// Representation of the package dependency graph
pub struct Graph {
    pub graph: petgraph::Graph<Node, DepKind>,
    pub nodes: HashMap<PackageId, NodeIndex>,
}

pub struct GraphCargoMetadata {
    pub graph: petgraph::Graph<NodeCargoMetadata, DependencyKind>,
    pub nodes: HashMap<cargo_metadata::PackageId, NodeIndex>,
}

/// Representation of a node within the package dependency graph
pub struct Node {
    pub id: PackageId,
    // TODO: Investigate why this was needed before the separation of printing
    // and graph traversal and if it should be added back.
    //pack: &'a Package,
}

pub struct NodeCargoMetadata {
    pub id: cargo_metadata::PackageId,
}

// Almost unmodified compared to the original in cargo-tree, should be fairly
// simple to move this and the dependency graph structure out to a library.
/// Function to build a graph of packages dependencies
pub fn build_graph<'a>(
    resolve: &'a Resolve,
    packages: &'a PackageSet,
    root: PackageId,
    target: Option<&str>,
    cfgs: Option<&[Cfg]>,
    extra_deps: ExtraDeps,
) -> CargoResult<Graph> {
    let mut graph = Graph {
        graph: petgraph::Graph::new(),
        nodes: HashMap::new(),
    };
    let node = Node {
        id: root,
        //pack: packages.get_one(root)?,
    };
    graph.nodes.insert(root, graph.graph.add_node(node));

    let mut pending = vec![root];

    let graph_configuration = GraphConfiguration {
        target,
        cfgs,
        extra_deps,
    };

    while let Some(package_id) = pending.pop() {
        add_package_dependencies_to_graph(
            resolve,
            package_id,
            packages,
            &graph_configuration,
            &mut graph,
            &mut pending,
        )?;
    }

    Ok(graph)
}

pub fn build_graph_cargo_metadata(
    cfgs: Option<&[Cfg]>,
    extra_deps: ExtraDeps,
    metadata: &Metadata,
    package_hash_map: HashMap<cargo_metadata::PackageId, cargo_metadata::Package>,
    target: Option<&str>,
) -> CargoResult<GraphCargoMetadata> {
    let mut graph = GraphCargoMetadata {
        graph: petgraph::Graph::new(),
        nodes: HashMap::new(),
    };

    let resolve = metadata
        .clone()
        .resolve
        .unwrap()
        .clone();

    let root = resolve.root.unwrap();

    let node = NodeCargoMetadata { id: root.clone() };

    graph.nodes.insert(root.clone(), graph.graph.add_node(node));

    let mut pending_packages = vec![root];

    let graph_configuration = GraphConfiguration {
        target,
        cfgs,
        extra_deps,
    };

    while let Some(package_id) = pending_packages.pop() {
        // add package dependencies to graph
        add_package_dependencies_to_graph_cargo_metadata(
            &graph_configuration,
            &mut graph,
            metadata,
            &package_id,
            &package_hash_map,
            &mut pending_packages,
        )?;
    }

    Ok(graph)
}

struct GraphConfiguration<'a> {
    target: Option<&'a str>,
    cfgs: Option<&'a [Cfg]>,
    extra_deps: ExtraDeps,
}

fn add_graph_node_if_not_present_and_edge(
    dependency: &Dependency,
    dependency_package_id: PackageId,
    graph: &mut Graph,
    index: NodeIndex,
    pending_packages: &mut Vec<PackageId>,
) {
    let dependency_index = match graph.nodes.entry(dependency_package_id) {
        Entry::Occupied(e) => *e.get(),
        Entry::Vacant(e) => {
            pending_packages.push(dependency_package_id);
            let node = Node {
                id: dependency_package_id,
                //pack: packages.get_one(dep_id)?,
            };
            *e.insert(graph.graph.add_node(node))
        }
    };
    graph
        .graph
        .add_edge(index, dependency_index, dependency.kind());
}

fn add_graph_node_if_not_present_and_edge_cargo_metadata(
    dependency: &cargo_metadata::Dependency,
    dependency_package_id: cargo_metadata::PackageId,
    graph: &mut GraphCargoMetadata,
    index: NodeIndex,
    pending_packages: &mut Vec<cargo_metadata::PackageId>
) {
    let dependency_index = match graph.nodes.entry(dependency_package_id.clone()) {
        Entry::Occupied(e) => *e.get(),
        Entry::Vacant(e) => {
            pending_packages.push(dependency_package_id.clone());
            let node = NodeCargoMetadata {
                id: dependency_package_id.clone(),
            };
            *e.insert(graph.graph.add_node(node))
        }
    };

    graph
        .graph
        .add_edge(index, dependency_index, dependency.kind);
}

#[doc(hidden)]
fn add_package_dependencies_to_graph<'a>(
    resolve: &'a Resolve,
    package_id: PackageId,
    packages: &'a PackageSet,
    graph_configuration: &GraphConfiguration,
    graph: &mut Graph,
    pending_packages: &mut Vec<PackageId>,
) -> CargoResult<()> {
    let index = graph.nodes[&package_id];
    let package = packages.get_one(package_id)?;

    for (raw_dependency_package_id, _) in resolve.deps_not_replaced(package_id)
    {
        let dependency_iterator = package
            .dependencies()
            .iter()
            .filter(|d| d.matches_ignoring_source(raw_dependency_package_id))
            .filter(|d| graph_configuration.extra_deps.allows(d.kind()))
            .filter(|d| {
                d.platform()
                    .and_then(|p| {
                        graph_configuration.target.map(|t| {
                            match graph_configuration.cfgs {
                                None => false,
                                Some(cfgs) => p.matches(t, cfgs),
                            }
                        })
                    })
                    .unwrap_or(true)
            });

        let dependency_package_id =
            match resolve.replacement(raw_dependency_package_id) {
                Some(id) => id,
                None => raw_dependency_package_id,
            };

        for dependency in dependency_iterator {
            add_graph_node_if_not_present_and_edge(
                dependency,
                dependency_package_id,
                graph,
                index,
                pending_packages,
            );
        }
    }

    Ok(())
}

fn add_package_dependencies_to_graph_cargo_metadata(
    _graph_configuration: &GraphConfiguration,
    graph: &mut GraphCargoMetadata,
    metadata: &Metadata,
    package_id: &cargo_metadata::PackageId,
    package_hash_map: &HashMap<cargo_metadata::PackageId, cargo_metadata::Package>,
    pending_packages: &mut Vec<cargo_metadata::PackageId>
) -> CargoResult<()> {
    let index = graph.nodes[&package_id];
    let package = metadata.packages
        .iter()
        .filter(|p| p.id.eq(package_id))
        .collect::<Vec::<&cargo_metadata::Package>>()
        .pop()
        .unwrap();

    for (package_id, package) in package_hash_map.into_iter() {
        // TODO - filter dependencies further here

        // TODO - Fix here to make deps handle properly

        println!("{}", package_id.repr);

        let dependency_id = metadata.packages
            .iter()
            .filter(|p| p.name == dependency.name)
            .map(|p| p.id.clone())
            .collect::<Vec::<cargo_metadata::PackageId>>()
            .pop()
            .unwrap();


        add_graph_node_if_not_present_and_edge_cargo_metadata(
            dependency,
            dependency_id,
            graph,
            index,
            pending_packages
        );
    }

    Ok(())
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    #[test]
    fn extra_deps_allows_test() {
        assert_eq!(ExtraDeps::All.allows(DepKind::Normal), true);
        assert_eq!(ExtraDeps::Build.allows(DepKind::Normal), true);
        assert_eq!(ExtraDeps::Dev.allows(DepKind::Normal), true);
        assert_eq!(ExtraDeps::NoMore.allows(DepKind::Normal), true);

        assert_eq!(ExtraDeps::All.allows(DepKind::Build), true);
        assert_eq!(ExtraDeps::All.allows(DepKind::Development), true);

        assert_eq!(ExtraDeps::Build.allows(DepKind::Build), true);
        assert_eq!(ExtraDeps::Build.allows(DepKind::Development), false);

        assert_eq!(ExtraDeps::Dev.allows(DepKind::Build), false);
        assert_eq!(ExtraDeps::Dev.allows(DepKind::Development), true);
    }
}
