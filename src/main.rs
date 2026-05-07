mod args;
mod logging;
mod metadata;
mod utils;
mod workspace;

use anyhow::{Context, Result, bail};
use args::{Commands, FusionArgs};
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::Parser;
use ignore::WalkBuilder;
use metadata::ProjectMetadata;
use rayon::prelude::*;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};
use workspace::resolve_workspace_manifest;

use crate::logging::init_logging;
use crate::utils::display_path;

const TOC_HEADER_LINES: usize = 2; // "// Table of Contents\n" + "// ==================\n"

fn main() -> Result<()> {
    let Commands::Fusion(args) = Commands::parse();

    let _guard = init_logging(args.verbose || args.info)?;

    args.verify_args()?;

    let file_args = Arc::new(args);

    let start = file_args.info.then(Instant::now);

    // Найти корень воркспейса, поднимаясь вверх по дереву директорий —
    // точно так же, как это делает сам Cargo.
    let resolution = resolve_workspace_manifest(&file_args.manifest_path)?;

    info!("Resolution: {resolution:?}");

    let metadata = if file_args.include_metadata {
        let manifest = cargo_toml::Manifest::from_path(&resolution.manifest_path)?;
        if manifest.package.is_some() {
            Some(ProjectMetadata::from_manifest(&resolution.manifest_path)?)
        } else {
            None
        }
    } else {
        None
    };

    // debug!("Metadata: {metadata:?}");

    let source_files = collect_source_files(&file_args, &resolution)?;

    // info!("Source files: {source_files:?}");

    if source_files.is_empty() {
        error!("No files found to include");
        return Ok(());
    }

    generate_output(&file_args, source_files, metadata, start)
}

fn print_info_summary(file_contents: &[(PathBuf, Vec<u8>)], start: Instant) {
    let elapsed = start.elapsed();
    let sum = file_contents
        .iter()
        .map(|(_, content)| content.lines().count())
        .sum::<usize>();

    error!(
        "Found {} files\nTotal Lines of Code: {sum}\nTime Elapsed: {}.{:03}s",
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
    let mut curr_line = head_len + TOC_HEADER_LINES;

    for (path, content) in file_contents {
        // Используем display() напрямую, он корректно обрабатывает Windows пути
        let path_str = path.display();
        let line_count = content.lines().count();

        writeln!(toc, "// Ln{curr_line:<6} : {path_str}")
            .expect("Writing to string should not fail");

        curr_line += line_count + 2; // +2 для разделителя и пустой строки
    }

    toc.push_str("// ==================\n");
    Some(toc)
}

fn generate_output(
    args: &FusionArgs,
    file_contents: Vec<(PathBuf, Vec<u8>)>,
    metadata: Option<ProjectMetadata>,
    start: Option<Instant>,
) -> Result<()> {
    let head = args.head.as_ref().map(std::fs::read).transpose()?;
    let table_of_contents = if args.table_of_contents {
        generate_table_of_contents(&file_contents, head.as_deref().map_or(0, <[u8]>::len))
            .map(String::into_bytes)
    } else {
        None
    };

    if let Some(start) = start {
        print_info_summary(&file_contents, start);
        return Ok(());
    }

    let mut cursor: Box<dyn Write> = if args.stdout {
        Box::new(BufWriter::new(std::io::stdout()))
    } else {
        Box::new(BufWriter::new(fs::File::create(&args.output)?))
    };

    write_output(
        &mut cursor,
        args,
        file_contents,
        metadata,
        table_of_contents,
    )?;

    Ok(())
}

fn write_output(
    cursor: &mut dyn Write,
    args: &FusionArgs,
    file_contents: Vec<(PathBuf, Vec<u8>)>,
    metadata: Option<ProjectMetadata>,
    table_of_contents: Option<Vec<u8>>,
) -> Result<()> {
    if let Some(head_path) = &args.head {
        let head_content = fs::read(head_path)?;
        cursor.write_all(&head_content)?;
    }

    if let Some(data) = metadata {
        let meta = data.to_string();
        cursor.write_all(meta.as_bytes())?;
    }

    if let Some(toc) = table_of_contents {
        cursor.write_all(&toc)?;
    }

    for (path, contents) in file_contents {
        writeln!(cursor, "{} {}", &args.separator, path.display())?;
        cursor.write_all(&contents)?;
        cursor.write_all(b"\n")?;
    }

    Ok(())
}

fn should_include_by_metadata(
    entry: &ignore::DirEntry,
    smaller_than: Option<&u64>,
    larger_than: Option<&u64>,
    newer_than: Option<&NaiveDateTime>,
    older_than: Option<&NaiveDateTime>,
) -> bool {
    let Ok(metadata) = entry.metadata() else {
        return false;
    };

    // Проверка размера
    if let Some(max_size) = smaller_than
        && metadata.len() > *max_size
    {
        return false;
    }

    if let Some(min_size) = larger_than
        && metadata.len() < *min_size
    {
        return false;
    }

    // Проверка даты
    if let (Some(newer), Some(older)) = (newer_than, older_than) {
        let modified: DateTime<Utc> = match metadata.modified() {
            Ok(m) => m.into(),
            Err(_) => return false,
        };

        if modified < newer.and_utc() || modified > older.and_utc() {
            return false;
        }
    } else if let Some(newer) = newer_than {
        let modified: DateTime<Utc> = match metadata.modified() {
            Ok(m) => m.into(),
            Err(_) => return false,
        };
        if modified < newer.and_utc() {
            return false;
        }
    } else if let Some(older) = older_than {
        let modified: DateTime<Utc> = match metadata.modified() {
            Ok(m) => m.into(),
            Err(_) => return false,
        };
        if modified > older.and_utc() {
            return false;
        }
    }

    true
}

fn filter_path(
    extension: &[String],
    smaller_than: Option<&u64>,
    larger_than: Option<&u64>,
    newer_than: Option<&NaiveDateTime>,
    older_than: Option<&NaiveDateTime>,
    include_lock: bool,
    entry: &ignore::DirEntry,
) -> Option<PathBuf> {
    let path = entry.path();

    // Проверка Cargo.lock
    if !include_lock && path.file_name() == Some(std::ffi::OsStr::new("Cargo.lock")) {
        return None;
    }

    // Проверка расширения
    let has_valid_extension = extension.iter().any(|ext| {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == ext)
    });

    if !has_valid_extension {
        return None;
    }

    // Проверка метаданных
    if should_include_by_metadata(entry, smaller_than, larger_than, newer_than, older_than) {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn collect_search_paths(
    args: &FusionArgs,
    search_root: &Path,   // директория поиска файлов (крейт или воркспейс)
    manifest_path: &Path, // путь к Cargo.toml (всегда корень воркспейса)
    manifest: &cargo_toml::Manifest,
) -> Result<Vec<PathBuf>> {
    let mut search_paths = Vec::new();

    // Add explicitly included paths
    search_paths.extend(
        args.include
            .iter()
            .filter(|&f| {
                let exists = f.is_dir() || f.is_file();
                if !exists {
                    error!("File not found: {}", f.display());
                }
                exists
            })
            .cloned(),
    );

    // Just add the current directory
    search_paths.push(search_root.to_path_buf());
    if args.dependencies {
        let deps = collect_dependencies_with_workspace(manifest, manifest_path, args)?;
        search_paths.extend(deps);
    }

    // Deduplicate search paths
    search_paths.sort();
    search_paths.dedup();

    if search_paths.is_empty() {
        bail!("No search paths found");
    }

    Ok(search_paths)
}

// Новая функция для сбора файлов с помощью walker
fn collect_files_from_paths(search_paths: &[PathBuf], args: &FusionArgs) -> Result<Vec<PathBuf>> {
    // Initialize walker with first path
    let mut walker = WalkBuilder::new(&search_paths[0]);
    for path in search_paths.iter().skip(1) {
        walker.add(path);
    }

    setup_walker(&mut walker, args);

    // Better: use sequential walker + rayon par_iter for filtering
    // ignore's WalkBuilder is already optimized; parallelism shines at the read stage
    let entries: Vec<_> = walker.build().collect();

    let mut source_files: Vec<PathBuf> = entries
        .into_par_iter()
        .filter_map(|result| {
            let entry = result.ok()?;
            filter_path(
                &args.extension,
                args.smaller_than.as_ref(),
                args.larger_than.as_ref(),
                args.newer_than.as_ref(),
                args.older_than.as_ref(),
                args.include_lock,
                &entry,
            )
        })
        .collect();

    if source_files.is_empty() {
        bail!("No files found to include");
    }

    // If there are any directories, get the files from them
    reduce_dir_list(&mut source_files, args);

    if let Some(max_files) = args.max_files
        && source_files.len() > max_files
    {
        error!(
            "Found {} files, but the maximum number of files is set to {}, truncating to fit the desired amount of files",
            source_files.len(),
            max_files
        );
        source_files.truncate(max_files);
    }

    Ok(source_files)
}

fn collect_source_files(
    args: &FusionArgs,
    resolution: &workspace::WorkspaceResolution,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let manifest = cargo_toml::Manifest::from_path(&resolution.manifest_path)
        .context("Failed to parse Cargo.toml")?;

    // В crate-режиме collect_search_paths получает директорию крейта,
    // поэтому поиск файлов будет ограничен только этим крейтом.
    let search_paths = collect_search_paths(
        args,
        &resolution.search_root,
        &resolution.manifest_path,
        &manifest,
    )?;

    let source_files = collect_files_from_paths(&search_paths, args)?;

    // Используем обычную итерацию для чтения, так как rayon может быть избыточен
    // для небольших проектов, и добавляем прогресс-индикатор
    let mut file_contents: Vec<_> = source_files
        .into_iter()
        .filter_map(|path| match fs::read(&path) {
            Ok(content) => Some((path, content)),
            Err(e) => {
                error!("Warning: failed to read {}: {}", path.display(), e);
                None
            }
        })
        .collect();

    // Сортировка через unstable sort для производительности
    file_contents.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    Ok(file_contents)
}

// New function to handle workspace-aware dependency collection
fn collect_dependencies_with_workspace(
    manifest: &cargo_toml::Manifest,
    manifest_path: &Path,
    args: &FusionArgs,
) -> Result<Vec<PathBuf>> {
    let mut deps = Vec::new();
    let manifest_parent = manifest_path
        .parent()
        .context("Cargo.toml has no parent directory")?
        .to_path_buf();

    // Collect dependencies from the current manifest
    for dep in manifest.dependencies.values() {
        if let Some(detail) = dep.detail()
            && let Some(path) = &detail.path
        {
            let dep_path = manifest_parent.join(path);

            // Check if this dependency is a workspace member
            let dep_manifest_path = dep_path.join("Cargo.toml");
            if dep_manifest_path.exists() {
                if let Ok(dep_manifest) = cargo_toml::Manifest::from_path(&dep_manifest_path) {
                    // If it's a workspace member, include its source files
                    deps.push(dep_path.clone());

                    // Recursively collect its dependencies if needed
                    if args.dependencies
                        && let Ok(child_deps) = collect_dependencies_with_workspace(
                            &dep_manifest,
                            &dep_manifest_path,
                            args,
                        )
                    {
                        deps.extend(child_deps);
                    }
                } else {
                    // Regular path dependency
                    deps.push(dep_path);
                }
            } else {
                // Path exists but no Cargo.toml, still add it
                deps.push(dep_path);
            }
        }
    }

    Ok(deps)
}

fn setup_walker(walker: &mut WalkBuilder, args: &FusionArgs) {
    // Строим overrides для исключений
    if !args.exclude.is_empty() {
        let mut overrides = ignore::overrides::OverrideBuilder::new(".");
        for excl in &args.exclude {
            // Префикс '!' исключает паттерн
            let pattern = format!("!{excl}");
            if let Err(e) = overrides.add(&pattern) {
                error!("Warning: invalid exclude pattern '{excl}': {e}");
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

/// Reduces a list of paths to files and/or dirs to a list of dirs to only files.
/// This function avoids iterating over the entire list multiple times by using a single pass
/// to collect directories and then processing them in bulk.
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
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
        .map(ignore::DirEntry::into_path)
        .collect();

    *paths = files.into_iter().chain(new_files).collect();
}
