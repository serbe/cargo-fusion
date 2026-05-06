#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod args;
mod filter;
mod metadata;
mod workspace;

use anyhow::{Context, Result, bail};
use args::{Commands, FusionArgs};
use clap::Parser;
use filter::FileFilter;
use ignore::WalkBuilder;
use metadata::ProjectMetadata;
use rayon::prelude::*;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use workspace::resolve_workspace_manifest;

const TOC_HEADER_LINES: usize = 2;

fn main() -> Result<()> {
    let Commands::Fusion(args) = Commands::parse();

    verify_args(&args)?;

    let start = args.info.then(Instant::now);

    let resolution = resolve_workspace_manifest(&args.manifest_path)?;

    let metadata = if args.include_metadata {
        let manifest = cargo_toml::Manifest::from_path(&resolution.manifest_path)?;
        if manifest.package.is_some() {
            Some(ProjectMetadata::from_manifest(&resolution.manifest_path)?)
        } else {
            None
        }
    } else {
        None
    };

    let source_files = collect_source_files(&args, &resolution)?;

    if source_files.is_empty() {
        eprintln!("No files found to include");
        return Ok(());
    }

    generate_output(&args, source_files, metadata, start)
}

fn verify_args(args: &FusionArgs) -> Result<()> {
    if let (Some(smaller), Some(larger)) = (args.smaller_than, args.larger_than)
        && smaller > larger
    {
        bail!("`smaller_than` ({smaller}) cannot be larger than `larger_than` ({larger})");
    }

    if let (Some(newer), Some(older)) = (args.newer_than, args.older_than)
        && newer > older
    {
        bail!("`newer_than` ({newer}) cannot be older than `older_than` ({older})");
    }

    if !args.manifest_path.exists() {
        bail!("Cargo.toml not found at {}", args.manifest_path.display());
    }

    Ok(())
}

fn generate_output(
    args: &FusionArgs,
    file_contents: Vec<(PathBuf, Vec<u8>)>,
    metadata: Option<ProjectMetadata>,
    start: Option<Instant>,
) -> Result<()> {
    if let Some(start) = start {
        print_info_summary(&file_contents, start);
        return Ok(());
    }

    let head = args.head.as_ref().map(std::fs::read).transpose()?;
    let table_of_contents = args
        .table_of_contents
        .then(|| generate_table_of_contents(&file_contents, head.as_deref().map_or(0, <[u8]>::len)))
        .flatten()
        .map(String::into_bytes);

    let mut writer: Box<dyn Write> = if args.stdout {
        Box::new(BufWriter::new(std::io::stdout()))
    } else {
        Box::new(BufWriter::new(fs::File::create(&args.output)?))
    };

    write_output(
        &mut writer,
        args,
        file_contents,
        metadata,
        table_of_contents,
    )
}

fn write_output(
    writer: &mut dyn Write,
    args: &FusionArgs,
    file_contents: Vec<(PathBuf, Vec<u8>)>,
    metadata: Option<ProjectMetadata>,
    table_of_contents: Option<Vec<u8>>,
) -> Result<()> {
    if let Some(head_path) = &args.head {
        writer.write_all(&fs::read(head_path)?)?;
    }

    if let Some(data) = metadata {
        writer.write_all(data.to_string().as_bytes())?;
    }

    if let Some(toc) = table_of_contents {
        writer.write_all(&toc)?;
    }

    for (path, contents) in file_contents {
        writeln!(writer, "{} {}", &args.separator, path.display())?;
        writer.write_all(&contents)?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

fn print_info_summary(file_contents: &[(PathBuf, Vec<u8>)], start: Instant) {
    let elapsed = start.elapsed();
    let total_lines: usize = file_contents
        .iter()
        .map(|(_, content)| content.lines().count())
        .sum();

    eprintln!(
        "Found {} files\nTotal Lines of Code: {total_lines}\nTime Elapsed: {}.{:03}s",
        file_contents.len(),
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
}

fn generate_table_of_contents(
    file_contents: &[(PathBuf, Vec<u8>)],
    head_len: usize,
) -> Option<String> {
    if file_contents.is_empty() {
        return None;
    }

    let mut toc = String::from("// Table of Contents\n// ==================\n");
    // head_len is in bytes, not lines — count lines in head separately if needed.
    // For now we track line offset from the header block onward.
    let mut curr_line = head_len + TOC_HEADER_LINES;

    for (path, content) in file_contents {
        let line_count = content.lines().count();
        writeln!(toc, "// Ln{curr_line:<6} : {}", path.display())
            .expect("writing to String is infallible");
        // +1 for the separator line written before each file's content
        curr_line += line_count + 1;
    }

    toc.push_str("// ==================\n");
    Some(toc)
}

fn collect_source_files(
    args: &FusionArgs,
    resolution: &workspace::WorkspaceResolution,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let manifest = cargo_toml::Manifest::from_path(&resolution.manifest_path)
        .context("Failed to parse Cargo.toml")?;

    let search_paths = collect_search_paths(
        args,
        &resolution.search_root,
        &resolution.manifest_path,
        &manifest,
    )?;

    let source_files = collect_files_from_paths(&search_paths, args)?;

    let (file_contents, errors): (Vec<_>, Vec<_>) = source_files
        .into_iter()
        .map(|path| {
            fs::read(&path)
                .map(|content| (path.clone(), content))
                .map_err(|e| (path, e))
        })
        .partition(Result::is_ok);

    for err in errors {
        let (path, e) = err.unwrap_err();
        eprintln!("Warning: failed to read {}: {e}", path.display());
    }

    let mut result: Vec<_> = file_contents.into_iter().map(Result::unwrap).collect();

    result.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    Ok(result)
}

fn collect_search_paths(
    args: &FusionArgs,
    search_root: &Path,
    manifest_path: &Path,
    manifest: &cargo_toml::Manifest,
) -> Result<Vec<PathBuf>> {
    let mut search_paths: Vec<PathBuf> = args
        .include
        .iter()
        .filter(|f| {
            let exists = f.is_dir() || f.is_file();
            if !exists {
                eprintln!("File not found: {}", f.display());
            }
            exists
        })
        .cloned()
        .collect();

    search_paths.push(search_root.to_path_buf());

    if args.dependencies {
        let deps = collect_dependencies_with_workspace(manifest, manifest_path, args)?;
        search_paths.extend(deps);
    }

    search_paths.sort();
    search_paths.dedup();

    if search_paths.is_empty() {
        bail!("No search paths found");
    }

    Ok(search_paths)
}

fn collect_files_from_paths(search_paths: &[PathBuf], args: &FusionArgs) -> Result<Vec<PathBuf>> {
    let mut walker = WalkBuilder::new(&search_paths[0]);
    for path in search_paths.iter().skip(1) {
        walker.add(path);
    }
    setup_walker(&mut walker, args);

    let filter = FileFilter::from_args(args);

    let entries: Vec<_> = walker.build().collect();
    let mut source_files: Vec<PathBuf> = entries
        .into_par_iter()
        .filter_map(|result| filter.apply(&result.ok()?))
        .collect();

    if source_files.is_empty() {
        bail!("No files found to include");
    }

    reduce_dir_list(&mut source_files, args);

    if let Some(max_files) = args.max_files
        && source_files.len() > max_files
    {
        eprintln!(
            "Found {} files, truncating to {max_files}",
            source_files.len()
        );
        source_files.truncate(max_files);
    }

    Ok(source_files)
}

fn collect_dependencies_with_workspace(
    manifest: &cargo_toml::Manifest,
    manifest_path: &Path,
    args: &FusionArgs,
) -> Result<Vec<PathBuf>> {
    let manifest_dir = manifest_path
        .parent()
        .context("Cargo.toml has no parent directory")?;

    let mut deps = Vec::new();

    for dep in manifest.dependencies.values() {
        let Some(detail) = dep.detail() else { continue };
        let Some(rel_path) = &detail.path else {
            continue;
        };

        let dep_path = manifest_dir.join(rel_path);
        let dep_manifest = dep_path.join("Cargo.toml");

        deps.push(dep_path.clone());

        if dep_manifest.exists()
            && args.dependencies
            && let Ok(child_manifest) = cargo_toml::Manifest::from_path(&dep_manifest)
            && let Ok(child_deps) =
                collect_dependencies_with_workspace(&child_manifest, &dep_manifest, args)
        {
            deps.extend(child_deps);
        }
    }

    Ok(deps)
}

fn setup_walker(walker: &mut WalkBuilder, args: &FusionArgs) {
    if !args.exclude.is_empty() {
        let mut overrides = ignore::overrides::OverrideBuilder::new(".");
        for excl in &args.exclude {
            let pattern = format!("!{excl}");
            if let Err(e) = overrides.add(&pattern) {
                eprintln!("Warning: invalid exclude pattern '{excl}': {e}");
            }
        }
        if let Ok(built) = overrides.build() {
            walker.overrides(built);
        }
    }

    walker
        .max_depth(args.depth)
        .standard_filters(args.skip_gitignore)
        .follow_links(false)
        .same_file_system(true);
}

fn reduce_dir_list(paths: &mut Vec<PathBuf>, args: &FusionArgs) {
    let (dirs, files): (Vec<_>, Vec<_>) = paths.drain(..).partition(|p| p.is_dir());

    if dirs.is_empty() {
        *paths = files;
        return;
    }

    let mut walker = WalkBuilder::new(&dirs[0]);
    for dir in dirs.iter().skip(1) {
        walker.add(dir);
    }
    setup_walker(&mut walker, args);

    let new_files: Vec<_> = walker
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .map(ignore::DirEntry::into_path)
        .collect();

    *paths = files.into_iter().chain(new_files).collect();
}
