//! This module provides the bulk of the code for the `cargo-geiger` executable.

// TODO: Review the module structure in this crate. There is very tight coupling
// between the main.rs and this module. Should this module be split into smaller
// parts? The printing and scanning can probably be further decoupled to provide
// a better base for adding more output formats.

// TODO: Investigate how cargo-clippy is implemented. Is it using syn?  Is is
// using rustc? Is it implementing a compiler plugin?

// TODO: Consider making this a lib.rs (again) and expose a full API, excluding
// only the terminal output..? That API would be dependent on cargo.

use cargo::core::package::PackageSet;
use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::ResolveOpts;
use cargo::core::{Package, PackageId, PackageIdSpec, Resolve, Workspace};
use cargo::ops;
use cargo::util::{self, important_paths, CargoResult};
use cargo::Config;
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand};
use cargo_platform::Cfg;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::{self, FromStr};

pub fn build_package_hash_map(
    manifest_path: &Option<PathBuf>,
    root_package_id: cargo_metadata::PackageId,
) -> CargoResult<HashMap<cargo_metadata::PackageId, cargo_metadata::Package>> {
    let mut package_hash_map = HashMap::new();

    let cargo_metadata = get_cargo_metadata(manifest_path)?;
    let resolve_package_ids = cargo_metadata
        .resolve
        .unwrap()
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect::<HashSet<cargo_metadata::PackageId>>();

    build_package_hash_map_inner(
        manifest_path,
        &mut package_hash_map,
        &resolve_package_ids,
        root_package_id,
    )?;

    Ok(package_hash_map)
}

fn build_package_hash_map_inner(
    manifest_path: &Option<PathBuf>,
    package_hash_map: &mut HashMap<
        cargo_metadata::PackageId,
        cargo_metadata::Package,
    >,
    resolve_package_ids: &HashSet<cargo_metadata::PackageId>,
    root_package_id: cargo_metadata::PackageId,
) -> CargoResult<()> {
    let cargo_metadata = get_cargo_metadata(manifest_path)?;

    for package in cargo_metadata.packages.iter() {
        if !package_hash_map.contains_key(&package.id)
            && resolve_package_ids.contains(&package.id)
        {
            package_hash_map.insert(package.clone().id, package.clone());

            let package_manifest_path = package.clone().manifest_path;

            if package.id != root_package_id {
                build_package_hash_map_inner(
                    &Some(package_manifest_path.clone()),
                    package_hash_map,
                    resolve_package_ids,
                    package.id.clone(),
                )?;
            }
        }
    }

    Ok(())
}

pub fn get_cargo_metadata(
    manifest_path: &Option<PathBuf>,
) -> CargoResult<Metadata> {
    let manifest_path_str = manifest_path
        .as_ref()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();

    Ok(MetadataCommand::new()
        .manifest_path(manifest_path_str)
        .features(CargoOpt::AllFeatures)
        .exec()?)
}

/// TODO: Write proper documentation for this.
/// This function seems to be looking up the active flags for conditional
/// compilation (cargo_platform::Cfg instances).
pub fn get_cfgs(
    config: &Config,
    target: &Option<String>,
    ws: &Workspace,
) -> CargoResult<Option<Vec<Cfg>>> {
    let mut process = util::process(&config.load_global_rustc(Some(ws))?.path);
    process.arg("--print=cfg").env_remove("RUST_LOG");
    if let Some(ref s) = *target {
        process.arg("--target").arg(s);
    }
    let output = match process.exec_with_output() {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };
    let output = str::from_utf8(&output.stdout).unwrap();
    let lines = output.lines();
    Ok(Some(
        lines
            .map(|s| Cfg::from_str(s).map_err(|e| e.into()))
            .collect::<CargoResult<Vec<_>>>()?,
    ))
}

pub fn get_workspace(
    config: &Config,
    manifest_path: Option<PathBuf>,
) -> CargoResult<Workspace> {
    let root = match manifest_path {
        Some(path) => path,
        None => important_paths::find_root_manifest_for_wd(config.cwd())?,
    };
    Workspace::new(&root, config)
}

pub fn get_registry<'a>(
    config: &'a Config,
    package: &Package,
) -> CargoResult<PackageRegistry<'a>> {
    let mut registry = PackageRegistry::new(config)?;
    registry.add_sources(Some(package.package_id().source_id()))?;
    Ok(registry)
}

pub fn resolve<'a, 'cfg>(
    package_id: PackageId,
    registry: &mut PackageRegistry<'cfg>,
    ws: &'a Workspace<'cfg>,
    features: &[String],
    all_features: bool,
    no_default_features: bool,
) -> CargoResult<(PackageSet<'a>, Resolve)> {
    let dev_deps = true; // TODO: Review this.
    let uses_default_features = !no_default_features;
    let opts = ResolveOpts::new(
        dev_deps,
        features,
        all_features,
        uses_default_features,
    );
    let prev = ops::load_pkg_lockfile(ws)?;
    let resolve = ops::resolve_with_previous(
        registry,
        ws,
        &opts,
        prev.as_ref(),
        None,
        &[PackageIdSpec::from_package_id(package_id)],
        true,
    )?;
    let packages = ops::get_resolved_packages(
        &resolve,
        PackageRegistry::new(ws.config())?,
    )?;
    Ok((packages, resolve))
}

// TODO: Make a wrapper type for canonical paths and hide all mutable access.

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn get_cfgs_test() {
        let config = Config::default().unwrap();

        let target: Option<String> = None;

        let root =
            important_paths::find_root_manifest_for_wd(config.cwd()).unwrap();
        let workspace = Workspace::new(&root, &config).unwrap();

        let cfgs = get_cfgs(&config, &target, &workspace);

        assert!(cfgs.is_ok());
        let cfg_vec_option = cfgs.unwrap();
        assert!(cfg_vec_option.is_some());
        let cfg_vec = cfg_vec_option.unwrap();

        let names: Vec<&Cfg> = cfg_vec
            .iter()
            .filter(|cfg| matches!(cfg, Cfg::Name(_)))
            .collect();

        let key_pairs: Vec<&Cfg> = cfg_vec
            .iter()
            .filter(|cfg| matches!(cfg, Cfg::KeyPair(_, _)))
            .collect();

        assert!(names.len() > 0);
        assert!(key_pairs.len() > 0);
    }

    #[test]
    fn get_workspace_test() {
        let config = Config::default().unwrap();
        let manifest_path: Option<PathBuf> = None;

        let workspace_cargo_result = get_workspace(&config, manifest_path);
        assert!(workspace_cargo_result.is_ok());
        let workspace = workspace_cargo_result.unwrap();

        let package_result = workspace.current();
        assert!(package_result.is_ok());
        let package = package_result.unwrap();

        assert_eq!(package.package_id().name(), "cargo-geiger");
    }

    #[test]
    fn get_registry_test() {
        let config = Config::default().unwrap();
        let workspace = Workspace::new(
            &important_paths::find_root_manifest_for_wd(config.cwd()).unwrap(),
            &config,
        )
        .unwrap();
        let package = workspace.current().unwrap();

        let registry_result = get_registry(&config, &package);

        assert!(registry_result.is_ok());
        let registry = registry_result.unwrap();

        let package_ids = vec![package.package_id()];
        let package_set_result = registry.get(&package_ids);
        assert!(package_set_result.is_ok());
        let package_set = package_set_result.unwrap();

        assert_eq!(package_set.sources().len(), 1);
    }
}
