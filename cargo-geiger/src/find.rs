use crate::rs_file::{
    into_rs_code_file, into_rs_code_file_cargo_metadata, is_file_with_ext,
    PackageMetrics, RsFile, RsFileMetricsWrapper,
};
use crate::scan::ScanMode;

use cargo::core::package::PackageSet;
use cargo::core::{Package, PackageId};
use cargo::util::CargoResult;
use geiger::find_unsafe_in_file;
use geiger::IncludeTests;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Provides a more terse and searchable name for the wrapped generic
/// collection.
pub struct GeigerContext {
    pub package_id_to_metrics: HashMap<PackageId, PackageMetrics>,
}

pub struct _GeigerContextCargoMetadata {
    pub package_id_to_metrics:
        HashMap<cargo_metadata::PackageId, PackageMetrics>,
}

pub fn find_unsafe_in_packages<F>(
    packs: &PackageSet,
    allow_partial_results: bool,
    include_tests: IncludeTests,
    mode: ScanMode,
    mut progress_step: F,
) -> GeigerContext
where
    F: FnMut(usize, usize) -> CargoResult<()>,
{
    let mut pack_id_to_metrics = HashMap::new();
    let packs = packs.get_many(packs.package_ids()).unwrap();
    let pack_code_files: Vec<_> = find_rs_files_in_packages(&packs).collect();
    let pack_code_file_count = pack_code_files.len();
    for (i, (pack_id, rs_code_file)) in pack_code_files.into_iter().enumerate()
    {
        let (is_entry_point, p) = match rs_code_file {
            RsFile::LibRoot(pb) => (true, pb),
            RsFile::BinRoot(pb) => (true, pb),
            RsFile::CustomBuildRoot(pb) => (true, pb),
            RsFile::Other(pb) => (false, pb),
        };
        if let (false, ScanMode::EntryPointsOnly) = (is_entry_point, &mode) {
            continue;
        }
        match find_unsafe_in_file(&p, include_tests) {
            Err(e) => {
                if allow_partial_results {
                    eprintln!(
                        "Failed to parse file: {}, {:?} ",
                        &p.display(),
                        e
                    );
                } else {
                    panic!("Failed to parse file: {}, {:?} ", &p.display(), e);
                }
            }
            Ok(file_metrics) => {
                let package_metrics = pack_id_to_metrics
                    .entry(pack_id)
                    .or_insert_with(PackageMetrics::default);
                let wrapper = package_metrics
                    .rs_path_to_metrics
                    .entry(p)
                    .or_insert_with(RsFileMetricsWrapper::default);
                wrapper.metrics = file_metrics;
                wrapper.is_crate_entry_point = is_entry_point;
            }
        }
        let _ = progress_step(i, pack_code_file_count);
    }
    GeigerContext {
        package_id_to_metrics: pack_id_to_metrics,
    }
}

pub fn _find_unsafe_in_packages_cargo_metadata<F>(
    packages: &Vec<cargo_metadata::Package>,
    allow_partial_results: bool,
    include_tests: IncludeTests,
    mode: ScanMode,
    mut progress_step: F,
) -> _GeigerContextCargoMetadata
where
    F: FnMut(usize, usize) -> CargoResult<()>,
{
    let mut package_id_to_metrics = HashMap::new();
    let package_code_files: Vec<_> =
        find_rs_files_in_packages_cargo_metadata(&packages).collect();
    let package_code_file_count = package_code_files.len();

    for (i, (package_id, rs_code_file)) in
        package_code_files.into_iter().enumerate()
    {
        let (is_entry_point, path_buf) = match rs_code_file {
            RsFile::LibRoot(path_buf) => (true, path_buf),
            RsFile::BinRoot(path_buf) => (true, path_buf),
            RsFile::CustomBuildRoot(path_buf) => (true, path_buf),
            RsFile::Other(path_buf) => (false, path_buf),
        };

        if let (false, ScanMode::EntryPointsOnly) = (is_entry_point, &mode) {
            continue;
        }

        match find_unsafe_in_file(&path_buf, include_tests) {
            Err(e) => {
                if allow_partial_results {
                    eprintln!(
                        "Failed to parse file: {}, {:?} ",
                        &path_buf.display(),
                        e
                    );
                } else {
                    panic!(
                        "Failed to parse file: {}, {:?} ",
                        &path_buf.display(),
                        e
                    );
                }
            }
            Ok(file_metrics) => {
                let package_metrics = package_id_to_metrics
                    .entry(package_id)
                    .or_insert_with(PackageMetrics::default);
                let wrapper = package_metrics
                    .rs_path_to_metrics
                    .entry(path_buf)
                    .or_insert_with(RsFileMetricsWrapper::default);
                wrapper.metrics = file_metrics;
                wrapper.is_crate_entry_point = is_entry_point;
            }
        }

        let _ = progress_step(i, package_code_file_count);
    }

    _GeigerContextCargoMetadata {
        package_id_to_metrics,
    }
}

fn find_rs_files_in_dir(dir: &Path) -> impl Iterator<Item = PathBuf> {
    let walker = WalkDir::new(dir).into_iter();
    walker.filter_map(|entry| {
        let entry = entry.expect("walkdir error."); // TODO: Return result.
        if !is_file_with_ext(&entry, "rs") {
            return None;
        }
        Some(
            entry
                .path()
                .canonicalize()
                .expect("Error converting to canonical path"),
        ) // TODO: Return result.
    })
}

fn find_rs_files_in_package(pack: &Package) -> Vec<RsFile> {
    // Find all build target entry point source files.
    let mut canon_targets = HashMap::new();
    for t in pack.targets() {
        let path = t.src_path().path();
        let path = match path {
            None => continue,
            Some(p) => p,
        };
        if !path.exists() {
            // A package published to crates.io is not required to include
            // everything. We have to skip this build target.
            continue;
        }
        let canon = path
            .canonicalize() // will Err on non-existing paths.
            .expect("canonicalize for build target path failed."); // FIXME
        let targets = canon_targets.entry(canon).or_insert_with(Vec::new);
        targets.push(t);
    }
    let mut out = Vec::new();
    for p in find_rs_files_in_dir(pack.root()) {
        if !canon_targets.contains_key(&p) {
            out.push(RsFile::Other(p));
        }
    }
    for (k, v) in canon_targets.into_iter() {
        for target in v {
            out.push(into_rs_code_file(target.kind(), k.clone()));
        }
    }
    out
}

fn find_rs_files_in_package_cargo_metadata(
    package: &cargo_metadata::Package,
) -> Vec<RsFile> {
    let mut canonical_targets = HashMap::new();
    for target in package.targets.iter() {
        let path = target.clone().src_path;

        if !path.exists() {
            continue;
        }

        let canonical_path = path
            .canonicalize()
            .expect("canonicalize for build target path failed.");

        let targets = canonical_targets
            .entry(canonical_path)
            .or_insert_with(Vec::new);

        targets.push(target.clone());
    }

    let mut rs_files = Vec::new();

    // https://docs.rs/cargo/0.47.0/src/cargo/core/package.rs.html#194-196
    let package_root = package.manifest_path.parent().unwrap();
    for path_buf in find_rs_files_in_dir(package_root) {
        if !canonical_targets.contains_key(&path_buf) {
            rs_files.push(RsFile::Other(path_buf));
        }
    }

    for (path_buf, targets) in canonical_targets.into_iter() {
        for target in targets {
            // map string to TargetKind

            into_rs_code_file_cargo_metadata(target.kind, path_buf.clone());
        }
    }

    rs_files
}

fn find_rs_files_in_packages<'a>(
    packs: &'a [&Package],
) -> impl Iterator<Item = (PackageId, RsFile)> + 'a {
    packs.iter().flat_map(|pack| {
        find_rs_files_in_package(pack)
            .into_iter()
            .map(move |path| (pack.package_id(), path))
    })
}

fn find_rs_files_in_packages_cargo_metadata<'a>(
    packages: &'a Vec<cargo_metadata::Package>,
) -> impl Iterator<Item = (cargo_metadata::PackageId, RsFile)> + 'a {
    packages.iter().flat_map(|package| {
        find_rs_files_in_package_cargo_metadata(package)
            .into_iter()
            .map(move |rs_file| (package.id.clone(), rs_file))
    })
}
