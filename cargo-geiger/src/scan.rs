use crate::find::{find_unsafe_in_packages, GeigerContext, find_unsafe_in_packages_cargo_metadata, GeigerContextCargoMetadata};
use crate::format::print::PrintConfig;
use crate::format::table::{_create_table_from_text_tree_lines, UNSAFE_COUNTERS_HEADER, create_table_from_text_tree_lines_cargo_metadata};
use crate::format::tree::{TextTreeLine};
use crate::format::{get_kind_group_name, EmojiSymbols, Pattern, SymbolKind};
use crate::graph::{Graph, GraphCargoMetadata};
use crate::rs_file::resolve_rs_file_deps;
use crate::traversal::{walk_dependency_tree, walk_dependency_tree_cargo_metadata};
use crate::Args;

use cargo::core::compiler::CompileMode;
use cargo::core::package::PackageSet;
use cargo::core::shell::Verbosity;
use cargo::core::{Package, PackageId, Workspace};
use cargo::ops::CompileOptions;
use cargo::util::CargoResult;
use cargo::Config;
use cargo::{CliError, CliResult};
use colored::Colorize;
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

pub enum ScanMode {
    // The default scan mode, scan every .rs file.
    Full,

    // An optimization to allow skipping everything except the entry points.
    // This is only useful for the "--forbid-only" mode since that mode only
    // depends on entry point .rs files.
    EntryPointsOnly,
}

pub fn _run_scan_mode_default(
    config: &Config,
    workspace: &Workspace,
    packages: &PackageSet,
    root_pack_id: PackageId,
    graph: &Graph,
    print_config: &PrintConfig,
    args: &Args,
) -> CliResult {
    let mut scan_output_lines = Vec::<String>::new();

    let compile_options = build_compile_options(args, config);
    let rs_files_used =
        resolve_rs_file_deps(&compile_options, &workspace).unwrap();
    if print_config.verbosity == Verbosity::Verbose {
        let mut rs_files_used_lines =
            construct_rs_files_used_lines(&rs_files_used);
        scan_output_lines.append(&mut rs_files_used_lines);
    }
    let mut progress = cargo::util::Progress::new("Scanning", config);
    let emoji_symbols = EmojiSymbols::new(print_config.charset);
    let geiger_context = find_unsafe_in_packages(
        &packages,
        print_config.allow_partial_results,
        print_config.include_tests,
        ScanMode::Full,
        |i, count| -> CargoResult<()> { progress.tick(i, count) },
    );
    progress.clear();
    config.shell().status("Scanning", "done")?;

    let mut output_key_lines =
        construct_scan_mode_default_output_key_lines(&emoji_symbols);
    scan_output_lines.append(&mut output_key_lines);

    let tree_lines = walk_dependency_tree(root_pack_id, &graph, &print_config);
    let (mut table_lines, mut warning_count) =
        _create_table_from_text_tree_lines(
            &geiger_context,
            packages,
            print_config,
            &rs_files_used,
            tree_lines,
        );
    scan_output_lines.append(&mut table_lines);

    for scan_output_line in scan_output_lines {
        println!("{}", scan_output_line);
    }

    _list_files_used_but_not_scanned(
        geiger_context,
        &rs_files_used,
        &mut warning_count,
    );

    if warning_count > 0 {
        Err(CliError::new(
            anyhow::Error::new(FoundWarningsError { warning_count }),
            1,
        ))
    } else {
        Ok(())
    }
}

pub fn run_scan_mode_default_cargo_metadata(
    args: &Args,
    config: &Config,
    graph: &GraphCargoMetadata,
    package_hashmap: &HashMap<cargo_metadata::PackageId, (cargo_metadata::Package, cargo_metadata::DependencyKind)>,
    print_config: &PrintConfig,
    root_package_id: cargo_metadata::PackageId,
    workspace: &Workspace,
) -> CliResult {
    let packages = package_hashmap
        .iter()
        .map(|(_, (p, _))| p.clone())
        .collect::<Vec<cargo_metadata::Package>>();

    let mut scan_output_lines = Vec::<String>::new();
    let compile_options = build_compile_options(args, config);
    let rs_files_used =
        resolve_rs_file_deps(&compile_options, &workspace).unwrap();
    if print_config.verbosity == Verbosity::Verbose {
        let mut rs_files_used_lines =
            construct_rs_files_used_lines(&rs_files_used);
        scan_output_lines.append(&mut rs_files_used_lines);
    }

    let mut progress = cargo::util::Progress::new("Scanning", config);
    let emoji_symbols = EmojiSymbols::new(print_config.charset);

    let geiger_context = find_unsafe_in_packages_cargo_metadata(
        &packages,
        print_config.allow_partial_results,
        print_config.include_tests,
        ScanMode::Full,
        |i, count| -> CargoResult<()> { progress.tick(i, count) },
    );

    progress.clear();
    config.shell().status("Scanning", "done")?;

    let mut output_key_lines =
        construct_scan_mode_default_output_key_lines(&emoji_symbols);
    scan_output_lines.append(&mut output_key_lines);

    let text_tree_lines = walk_dependency_tree_cargo_metadata(
        root_package_id,
        &graph,
        print_config
    );

    let (mut table_lines, mut warning_count) =
        create_table_from_text_tree_lines_cargo_metadata(
            &geiger_context,
            package_hashmap,
            print_config,
            &rs_files_used,
            text_tree_lines,
        );
    scan_output_lines.append(&mut table_lines);

    for scan_output_line in scan_output_lines {
        println!("{}", scan_output_line);
    }

    list_files_used_but_not_scanned_cargo_metadata(
        geiger_context,
        &rs_files_used,
        &mut warning_count,
    );

    if warning_count > 0 {
        Err(CliError::new(
            anyhow::Error::new(FoundWarningsError { warning_count }),
            1,
        ))
    } else {
        Ok(())
    }
}

pub fn run_scan_mode_forbid_only(
    config: &Config,
    packages: &PackageSet,
    root_pack_id: PackageId,
    graph: &Graph,
    print_config: &PrintConfig,
) -> CliResult {
    let mut scan_output_lines = Vec::<String>::new();

    let emoji_symbols = EmojiSymbols::new(print_config.charset);
    let sym_lock = emoji_symbols.emoji(SymbolKind::Lock);
    let sym_qmark = emoji_symbols.emoji(SymbolKind::QuestionMark);

    let mut progress = cargo::util::Progress::new("Scanning", config);
    let geiger_ctx = find_unsafe_in_packages(
        &packages,
        print_config.allow_partial_results,
        print_config.include_tests,
        ScanMode::EntryPointsOnly,
        |i, count| -> CargoResult<()> { progress.tick(i, count) },
    );
    progress.clear();
    config.shell().status("Scanning", "done")?;

    let mut output_key_lines =
        construct_scan_mode_forbid_only_output_key_lines(&emoji_symbols);

    scan_output_lines.append(&mut output_key_lines);

    let tree_lines = walk_dependency_tree(root_pack_id, &graph, &print_config);
    for tree_line in tree_lines {
        match tree_line {
            TextTreeLine::Package { id, tree_vines } => {
                let package = packages.get_one(id).unwrap(); // FIXME
                let name = format_package_name(package, print_config.format);
                let pack_metrics = geiger_ctx.package_id_to_metrics.get(&id);
                let package_forbids_unsafe = match pack_metrics {
                    None => false, // no metrics available, .rs parsing failed?
                    Some(pm) => pm
                        .rs_path_to_metrics
                        .iter()
                        .all(|(_k, v)| v.metrics.forbids_unsafe),
                };
                let (symbol, name) = if package_forbids_unsafe {
                    (&sym_lock, name.green())
                } else {
                    (&sym_qmark, name.red())
                };
                scan_output_lines
                    .push(format!("{} {}{}", symbol, tree_vines, name));
            }
            TextTreeLine::ExtraDepsGroup { kind, tree_vines } => {
                let name = get_kind_group_name(kind);
                if name.is_none() {
                    continue;
                }
                let name = name.unwrap();
                // TODO: Fix the alignment on macOS (others too?)
                scan_output_lines.push(format!("  {}{}", tree_vines, name));
            }
        }
    }

    for scan_output_line in scan_output_lines {
        println!("{}", scan_output_line);
    }

    Ok(())
}

#[derive(Debug)]
struct FoundWarningsError {
    pub warning_count: u64,
}

impl Error for FoundWarningsError {}

/// Forward Display to Debug.
impl fmt::Display for FoundWarningsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Based on code from cargo-bloat. It seems weird that CompileOptions can be
/// constructed without providing all standard cargo options, TODO: Open an issue
/// in cargo?
fn build_compile_options<'a>(
    args: &'a Args,
    config: &'a Config,
) -> CompileOptions {
    let features = args
        .features
        .as_ref()
        .cloned()
        .unwrap_or_else(String::new)
        .split(' ')
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let mut compile_options =
        CompileOptions::new(&config, CompileMode::Check { test: false })
            .unwrap();
    compile_options.features = features;
    compile_options.all_features = args.all_features;
    compile_options.no_default_features = args.no_default_features;

    // TODO: Investigate if this is relevant to cargo-geiger.
    //let mut bins = Vec::new();
    //let mut examples = Vec::new();
    // opt.release = args.release;
    // opt.target = args.target.clone();
    // if let Some(ref name) = args.bin {
    //     bins.push(name.clone());
    // } else if let Some(ref name) = args.example {
    //     examples.push(name.clone());
    // }
    // if args.bin.is_some() || args.example.is_some() {
    //     opt.filter = ops::CompileFilter::new(
    //         false,
    //         bins.clone(), false,
    //         Vec::new(), false,
    //         examples.clone(), false,
    //         Vec::new(), false,
    //         false,
    //     );
    // }

    compile_options
}

fn construct_rs_files_used_lines(
    rs_files_used: &HashSet<PathBuf>,
) -> Vec<String> {
    // Print all .rs files found through the .d files, in sorted order.
    let mut paths = rs_files_used
        .iter()
        .map(std::borrow::ToOwned::to_owned)
        .collect::<Vec<PathBuf>>();

    paths.sort();

    paths
        .iter()
        .map(|p| format!("Used by build (sorted): {}", p.display()))
        .collect::<Vec<String>>()
}

fn construct_scan_mode_default_output_key_lines(
    emoji_symbols: &EmojiSymbols,
) -> Vec<String> {
    let mut output_key_lines = Vec::<String>::new();

    output_key_lines.push(String::new());
    output_key_lines.push(String::from("Metric output format: x/y"));
    output_key_lines
        .push(String::from("    x = unsafe code used by the build"));
    output_key_lines
        .push(String::from("    y = total unsafe code found in the crate"));
    output_key_lines.push(String::new());

    output_key_lines.push(String::from("Symbols: "));
    let forbids = "No `unsafe` usage found, declares #![forbid(unsafe_code)]";
    let unknown = "No `unsafe` usage found, missing #![forbid(unsafe_code)]";
    let guilty = "`unsafe` usage found";

    let shift_sequence = if emoji_symbols.will_output_emoji() {
        "\r\x1B[7C" // The radiation icon's Unicode width is 2,
                    // but by most terminals it seems to be rendered at width 1.
    } else {
        ""
    };

    output_key_lines.push(format!(
        "    {: <2} = {}",
        emoji_symbols.emoji(SymbolKind::Lock),
        forbids
    ));

    output_key_lines.push(format!(
        "    {: <2} = {}",
        emoji_symbols.emoji(SymbolKind::QuestionMark),
        unknown
    ));

    output_key_lines.push(format!(
        "    {: <2}{} = {}",
        emoji_symbols.emoji(SymbolKind::Rads),
        shift_sequence,
        guilty
    ));

    output_key_lines.push(String::new());

    output_key_lines.push(format!(
        "{}",
        UNSAFE_COUNTERS_HEADER
            .iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join(" ")
            .bold()
    ));

    output_key_lines.push(String::new());

    output_key_lines
}

fn construct_scan_mode_forbid_only_output_key_lines(
    emoji_symbols: &EmojiSymbols,
) -> Vec<String> {
    let mut output_key_lines = Vec::<String>::new();

    output_key_lines.push(String::new());
    output_key_lines.push(String::from("Symbols: "));

    let forbids = "All entry point .rs files declare #![forbid(unsafe_code)].";
    let unknown = "This crate may use unsafe code.";

    output_key_lines.push(format!(
        "    {: <2} = {}",
        emoji_symbols.emoji(SymbolKind::Lock),
        forbids
    ));

    output_key_lines.push(format!(
        "    {: <2} = {}",
        emoji_symbols.emoji(SymbolKind::QuestionMark),
        unknown
    ));

    output_key_lines.push(String::new());

    output_key_lines
}

fn format_package_name(package: &Package, pattern: &Pattern) -> String {
    format!(
        "{}",
        pattern.display(&package.package_id(), package.manifest().metadata())
    )
}

fn _list_files_used_but_not_scanned(
    geiger_context: GeigerContext,
    rs_files_used: &HashSet<PathBuf>,
    warning_count: &mut u64,
) {
    let scanned_files = geiger_context
        .package_id_to_metrics
        .iter()
        .flat_map(|(_k, v)| v.rs_path_to_metrics.keys())
        .collect::<HashSet<&PathBuf>>();
    let used_but_not_scanned =
        rs_files_used.iter().filter(|p| !scanned_files.contains(p));
    for path in used_but_not_scanned {
        eprintln!(
            "WARNING: Dependency file was never scanned: {}",
            path.display()
        );
        *warning_count += 1;
    }
}

fn list_files_used_but_not_scanned_cargo_metadata(
    geiger_context: GeigerContextCargoMetadata,
    rs_files_used: &HashSet<PathBuf>,
    warning_count: &mut u64,
) {
    let scanned_files = geiger_context
        .package_id_to_metrics
        .iter()
        .flat_map(|(_k, v)| v.rs_path_to_metrics.keys())
        .collect::<HashSet<&PathBuf>>();
    let used_but_not_scanned =
        rs_files_used.iter().filter(|p| !scanned_files.contains(p));
    for path in used_but_not_scanned {
        eprintln!(
            "WARNING: Dependency file was never scanned: {}",
            path.display()
        );
        *warning_count += 1;
    }
}

#[cfg(test)]
mod scan_tests {
    use super::*;

    use crate::format::Charset;

    use cargo::util::important_paths;

    #[test]
    fn build_compile_options_test() {
        let args_all_features = rand::random();
        let args_features = Some(String::from("unit test features"));
        let args_no_default_features = rand::random();

        let args = Args {
            all: false,
            all_deps: false,
            all_features: args_all_features,
            all_targets: false,
            build_deps: false,
            charset: Charset::Utf8,
            color: None,
            dev_deps: false,
            features: args_features,
            forbid_only: false,
            format: "".to_string(),
            frozen: false,
            help: false,
            include_tests: false,
            invert: false,
            locked: false,
            manifest_path: None,
            no_default_features: args_no_default_features,
            no_indent: false,
            offline: false,
            package: None,
            prefix_depth: false,
            quiet: None,
            target: None,
            unstable_flags: vec![],
            verbose: 0,
            version: false,
        };

        let config = Config::default().unwrap();

        let compile_options = build_compile_options(&args, &config);

        assert_eq!(compile_options.all_features, args_all_features);
        assert_eq!(compile_options.features, vec!["unit", "test", "features"]);
        assert_eq!(
            compile_options.no_default_features,
            args_no_default_features
        );
    }

    #[test]
    fn construct_rs_files_used_lines_test() {
        let mut rs_files_used = HashSet::<PathBuf>::new();

        rs_files_used.insert(PathBuf::from("b/path.rs"));
        rs_files_used.insert(PathBuf::from("a/path.rs"));
        rs_files_used.insert(PathBuf::from("c/path.rs"));

        let rs_files_used_lines = construct_rs_files_used_lines(&rs_files_used);

        assert_eq!(
            rs_files_used_lines,
            vec![
                String::from("Used by build (sorted): a/path.rs"),
                String::from("Used by build (sorted): b/path.rs"),
                String::from("Used by build (sorted): c/path.rs"),
            ]
        );
    }

    #[test]
    fn construct_scan_mode_default_output_key_lines_test() {
        let emoji_symbols = EmojiSymbols::new(Charset::Utf8);
        let output_key_lines =
            construct_scan_mode_default_output_key_lines(&emoji_symbols);

        assert_eq!(output_key_lines.len(), 12);
    }

    #[test]
    fn construct_scan_mode_forbid_only_output_key_lines_test() {
        let emoji_symbols = EmojiSymbols::new(Charset::Utf8);
        let output_key_lines =
            construct_scan_mode_forbid_only_output_key_lines(&emoji_symbols);

        assert_eq!(output_key_lines.len(), 5);
    }

    #[test]
    fn format_package_name_test() {
        let pattern = Pattern::try_build("{p}").unwrap();

        let config = Config::default().unwrap();
        let workspace = Workspace::new(
            &important_paths::find_root_manifest_for_wd(config.cwd()).unwrap(),
            &config,
        )
        .unwrap();

        let package = workspace.current().unwrap();

        let formatted_package_name = format_package_name(&package, &pattern);

        assert_eq!(formatted_package_name, "cargo-geiger 0.10.2");
    }
}
