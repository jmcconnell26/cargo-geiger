use cargo_metadata::{DependencyKind, DepKindInfo, Metadata, PackageId};
use cargo::CargoResult;
use std::cmp::min;
use std::collections::HashMap;

pub fn build_dependency_kind_hashmap(
    metadata: &Metadata,
    root_package_id: PackageId
) -> CargoResult<HashMap<PackageId, DependencyKind>> {
    let nodes = &metadata.resolve.as_ref().unwrap().nodes;
    let id_to_node: HashMap<PackageId, &cargo_metadata::Node> = nodes
        .iter()
        .map(|n| (n.id.clone(), n))
        .collect();

    let mut id_to_dep_kind: HashMap<PackageId, PrivateDependencyKind> = HashMap::new();
    id_to_dep_kind.insert(root_package_id.clone(), PrivateDependencyKind::Normal);

    let mut current_queue: Vec<&cargo_metadata::Node> = vec![id_to_node[&root_package_id]];
    let mut next_step_queue: Vec<&cargo_metadata::Node> = Vec::new();
    while current_queue.len() > 0 {
        for parent in current_queue.drain(..) {
            let parent_dep_kind = id_to_dep_kind[&parent.id];
            for child in &parent.deps {
                let child_id = child.pkg.clone();
                let dep_kind = strongest_dep_kind(child.dep_kinds.as_slice());
                let dep_kind = min(dep_kind, parent_dep_kind);
                let dep_kind_on_previous_visit = id_to_dep_kind.get(&child_id);
                if dep_kind_on_previous_visit == None || &dep_kind > dep_kind_on_previous_visit.unwrap() {
                    // if we haven't visited this node in dependency graph yet
                    // or if we've visited it with a weaker dependency type,
                    // records its new dependency type and add it to the queue to visit its dependencies
                    id_to_dep_kind.insert(child_id.clone(), dep_kind);
                    next_step_queue.push(id_to_node[&child_id]);
                }
            }
        }
        std::mem::swap(&mut next_step_queue, &mut current_queue);
    }

    Ok(id_to_dep_kind
        .iter()
        .map(|(package_id, private_dep_kind)| (package_id.clone(), DependencyKind::from(*private_dep_kind)))
        .collect::<HashMap<PackageId, DependencyKind>>())
}

impl From<PrivateDependencyKind> for DependencyKind {
    fn from(private_kind: PrivateDependencyKind) -> Self {
        match private_kind {
            PrivateDependencyKind::Development => panic!("Cannot convert development dependency to serializable format"),
            PrivateDependencyKind::Build => DependencyKind::Build,
            PrivateDependencyKind::Normal => DependencyKind::Normal,
        }
    }
}

/// The fields are ordered from weakest to strongest so that casting to integer would make sense
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum PrivateDependencyKind {
    Development,
    Build,
    Normal,
}

impl From<&cargo_metadata::DependencyKind> for PrivateDependencyKind {
    fn from(kind: &cargo_metadata::DependencyKind) -> Self {
        match kind {
            cargo_metadata::DependencyKind::Normal => PrivateDependencyKind::Normal,
            cargo_metadata::DependencyKind::Development => PrivateDependencyKind::Development,
            cargo_metadata::DependencyKind::Build => PrivateDependencyKind::Build,
            _ => panic!("Unknown dependency kind")
        }
    }
}

fn strongest_dep_kind(dependencies: &[DepKindInfo]) -> PrivateDependencyKind {
    dependencies
        .iter()
        .map(|d| PrivateDependencyKind::from(&d.kind))
        .max()
        .unwrap_or(PrivateDependencyKind::Normal) // for compatibility with Rust earlier than 1.41
}