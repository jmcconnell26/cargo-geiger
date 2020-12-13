use crate::format::emoji_symbols::EmojiSymbols;
use crate::format::table::{
    create_table_from_text_tree_lines, TableParameters, UNSAFE_COUNTERS_HEADER,
};
use crate::format::{Charset, SymbolKind};
use crate::graph::Graph;
use crate::mapping::CargoMetadataParameters;
use crate::tree::traversal::walk_dependency_tree;

use super::super::{
    construct_rs_files_used_lines, list_files_used_but_not_scanned,
    ScanDetails, ScanParameters, ScanResult,
};
use super::scan;

use cargo::core::shell::Verbosity;
use cargo::core::Workspace;
use cargo::CliError;
use cargo_metadata::PackageId;
use colored::Colorize;

pub fn scan_to_table(
    cargo_metadata_parameters: &CargoMetadataParameters,
    graph: &Graph,
    root_package_id: PackageId,
    scan_parameters: &ScanParameters,
    workspace: &Workspace,
) -> Result<ScanResult, CliError> {
    let mut combined_scan_output_lines = Vec::<String>::new();

    let ScanDetails {
        rs_files_used,
        geiger_context,
    } = scan(cargo_metadata_parameters, scan_parameters, workspace)?;

    if scan_parameters.print_config.verbosity == Verbosity::Verbose {
        let mut rs_files_used_lines =
            construct_rs_files_used_lines(&rs_files_used);
        combined_scan_output_lines.append(&mut rs_files_used_lines);
    }

    let emoji_symbols = EmojiSymbols::new(scan_parameters.print_config.charset);
    let mut output_key_lines = construct_key_lines(
        scan_parameters.print_config.charset,
        &emoji_symbols,
    );
    combined_scan_output_lines.append(&mut output_key_lines);

    let text_tree_lines = walk_dependency_tree(
        cargo_metadata_parameters,
        &graph,
        &scan_parameters.print_config,
        root_package_id,
    );
    let table_parameters = TableParameters {
        geiger_context: &geiger_context,
        print_config: &scan_parameters.print_config,
        rs_files_used: &rs_files_used,
    };

    let ScanResult {
        mut scan_output_lines,
        mut warning_count,
    } = create_table_from_text_tree_lines(
        cargo_metadata_parameters,
        &table_parameters,
        text_tree_lines,
    );
    combined_scan_output_lines.append(&mut scan_output_lines);

    let used_but_not_scanned =
        list_files_used_but_not_scanned(&geiger_context, &rs_files_used);
    warning_count += used_but_not_scanned.len() as u64;
    for path in &used_but_not_scanned {
        eprintln!(
            "WARNING: Dependency file was never scanned: {}",
            path.display()
        );
    }

    Ok(ScanResult {
        scan_output_lines: combined_scan_output_lines,
        warning_count,
    })
}

fn construct_key_lines(
    charset: Charset,
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

    let symbol_kinds_to_string_values = vec![
        (SymbolKind::Lock, "", forbids),
        (SymbolKind::QuestionMark, "", unknown),
        (SymbolKind::Rads, shift_sequence, guilty),
    ];

    for (symbol_kind, shift_sequence, string_values) in
        symbol_kinds_to_string_values
    {
        output_key_lines.push(format!(
            "    {: <2}{} = {}",
            emoji_symbols.emoji(symbol_kind),
            shift_sequence,
            string_values
        ));
    }

    output_key_lines.push(String::new());

    let key = UNSAFE_COUNTERS_HEADER
        .iter()
        .map(|s| s.to_owned())
        .collect::<Vec<_>>()
        .join(" ");

    match charset {
        Charset::GitHubMarkdown => output_key_lines.push(key),
        _ => output_key_lines.push(key.bold().to_string()),
    }

    output_key_lines.push(String::new());

    output_key_lines
}
