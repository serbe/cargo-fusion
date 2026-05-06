mod args;
mod collector;
mod filter;
mod fs_utils;
mod manifest_utils;
mod metadata;
mod toc;
mod validation;
mod workspace;
mod writer;

use anyhow::Result;
use args::{Commands, FusionArgs};
use clap::Parser;
use collector::FileCollector;
use metadata::ProjectMetadata;
use std::{path::PathBuf, time::Instant};
use toc::TableOfContents;
use validation::validate_args;
use workspace::resolve_workspace_manifest;
use writer::write_output;

use crate::{fs_utils::read_file, workspace::WorkspaceResolution};

fn main() -> Result<()> {
    let Commands::Fusion(args) = Commands::parse();
    validate_args(&args)?;

    let start_time = args.info.then(Instant::now);
    let resolution = resolve_workspace_manifest(&args.manifest_path)?;
    let metadata = load_metadata_if_needed(&args, &resolution)?;
    let files = FileCollector::collect_files(&args, &resolution)?;

    if files.is_empty() {
        eprintln!("No files found to include");
        return Ok(());
    }

    if args.info {
        print_stats(&files, start_time.unwrap());
        return Ok(());
    }

    let toc = create_table_of_contents(&args, &files);
    write_output(&args, files, metadata, toc)
}

fn load_metadata_if_needed(
    args: &FusionArgs,
    resolution: &WorkspaceResolution,
) -> Result<Option<ProjectMetadata>> {
    if !args.include_metadata {
        return Ok(None);
    }

    ProjectMetadata::from_manifest(&resolution.manifest_path)
}

fn create_table_of_contents(args: &FusionArgs, files: &[(PathBuf, Vec<u8>)]) -> Option<Vec<u8>> {
    if !args.table_of_contents {
        return None;
    }

    let head_content = args.head.as_ref().and_then(|p| read_file(p).ok());
    let header_lines = TableOfContents::count_header_lines(head_content.as_deref());
    TableOfContents::generate(files, header_lines)
}

fn print_stats(files: &[(PathBuf, Vec<u8>)], start: Instant) {
    let elapsed = start.elapsed();
    let total_lines: usize = files
        .iter()
        .map(|(_, content)| content.iter().filter(|&&b| b == b'\n').count())
        .sum();

    eprintln!(
        "Found {} files\nTotal Lines of Code: {}\nTime Elapsed: {}.{:03}s",
        files.len(),
        total_lines,
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
}
