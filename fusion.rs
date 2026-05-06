// Project: cargo-fusion (v0.1.0)
// Description: A Cargo subcommand that fuses a Rust workspace into a single file
// Authors: Serbe
// License: MIT
// Repository: https://github.com/serbe/fusion

// README
// ======
// # Cargo Fusion
// 
// Cargo Fusion is a Rust tool that generates a single file containing all the source code of a Rust project, primarily designed for piping source code into Large Language Models (LLMs).
// 
// ![Rust](https://img.shields.io/badge/language-Rust-orange.svg)
// ![License](https://img.shields.io/badge/license-MIT-blue.svg)
// 
// ## About this fork
// 
// This is a fork of [cargo-onefile](https://github.com/exotik850/cargo-onefile) by @Exotik850.
// 
// ## Table of Contents
// 
// - [Cargo Fusion](#cargo-fusion)
//   - [Table of Contents](#table-of-contents)
//   - [Installation](#installation)
//   - [Usage](#usage)
//   - [Features](#features)
//   - [Configuration](#configuration)
//   - [Contributing](#contributing)
//   - [License](#license)
//   - [Support](#support)
// 
// ## Installation
// 
// To install Cargo Fusion, you need to have Rust and Cargo installed on your system. If you don't have them installed, follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).
// 
// Once Rust is installed, you can install Cargo Fusion using the following command:
// 
// ```sh
// cargo install cargo-fusion
// ```
// 
// ## Usage
// 
// To use Cargo Fusion, navigate to your Rust project directory and run:
// 
// ```sh
// cargo fusion
// ```
// 
// This will generate a single file containing all the source code of your project. By default, the output file will be named `fusion.rs` in the current directory.
// 
// For more options, you can use the `--help` flag:
// 
// ```sh
// cargo fusion --help
// ```
// 
// ## Features
// 
// 1. **Single File Generation**: Combines all source files into a single file for easy sharing or analysis.
// 2. **Flexible Output**: Supports writing to a file or stdout, with customizable output paths.
// 3. **Dependency Inclusion**: Option to include project dependencies in the output.
// 4. **Customizable Filtering**: Allows filtering files based on size, modification date, and file extensions.
// 5. **Performance Metrics**: Includes an info mode to measure performance and provide statistics on the processed files.
// 
// ## Configuration
// 
// Cargo Fusion offers various configuration options:
// 
// - `--stdout`: Output to stdout instead of a file.
// - `--table-of-contents`: Include a table of contents at the top of the output.
// - `-o, --output <PATH>`: Specify the output file path.
// - `-p, --manifest-path <PATH>`: Specify the path to the Cargo.toml file.
// - `--head <PATH>`: Prepend contents of a header file to the output.
// - `--depth <DEPTH>`: Set the maximum depth to search for files.
// - `--skip-gitignore <BOOL>`: Choose whether to skip gitignored files.
// - `-d, --dependencies`: Include project dependencies in the output.
// - `--separator <STRING>`: Set the separator shown between files.
// - `--newer-than <DATETIME>`: Exclude files older than the specified datetime.
// - `--older-than <DATETIME>`: Exclude files newer than the specified datetime.
// - `--smaller-than <SIZE>`: Exclude files larger than the specified size in bytes.
// - `--larger-than <SIZE>`: Exclude files smaller than the specified size in bytes.
// - `--max-files <NUMBER>`: Set the maximum number of files to include.
// - `-E, --extension <EXTENSION>`: Include files with the specified extension(s).
// - `-e, --exclude <FILE>`: Exclude specified files from the output.
// 
// For a complete list of options, use the `--help` flag.
// 
// ## Contributing
// 
// Contributions to Cargo Fusion are welcome! Please follow these steps to contribute:
// 
// 1. Fork the repository.
// 2. Create a new branch for your feature or bug fix.
// 3. Write your code and tests.
// 4. Ensure all tests pass by running `cargo test`.
// 5. Submit a pull request with a clear description of your changes.
// 
// Please adhere to the existing code style and include appropriate tests for new features.
// 
// ## License
// 
// This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
// 
// ## Support
// 
// For support, questions, or feedback, please [open an issue](https://github.com/serbe/cargo-fusion/issues) on the GitHub repository.
// ======
// \\?\D:\github\cargo-fusion\src\args.rs
use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub enum Commands {
    #[command(name = "fusion")]
    #[command(author, version, about)]
    Fusion(FusionArgs),
}

#[derive(Parser, Debug, Clone)]
#[command(name = "Cargo Fusion")]
#[command(
    about = "Generate a single file that contains all the source code of a Rust project.\nMainly intended to pipe source code into an LLM."
)]
#[command(version, long_about = None)]
pub struct FusionArgs {
    /// Output to stdout instead of a file.
    /// If this flag is set, the `output` option is ignored.
    ///
    /// Example:
    ///   cargo fusion --stdout
    #[arg(long)]
    pub stdout: bool,

    /// Include a table of contents at the top of the output.
    ///
    /// Example:
    ///   cargo fusion --table-of-contents
    #[arg(long, action)]
    pub table_of_contents: bool,

    /// Path to the output file.
    ///
    /// Example:
    ///   cargo fusion -o ./output/combined.rs
    #[arg(short, long, default_value = "./fusion.rs")]
    pub output: PathBuf,

    /// Path to a `Cargo.toml` file.
    /// If not provided, looks for `Cargo.toml` in the current directory.
    ///
    /// Example:
    ///   cargo fusion -p ./path/to/Cargo.toml
    #[arg(short = 'p', long, default_value = "./Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Path to a header file prepended to the output.
    ///
    /// Example:
    ///   cargo fusion --head ./header.txt
    #[arg(long)]
    pub head: Option<PathBuf>,

    /// Maximum depth to search for files.
    ///
    /// Example:
    ///   cargo fusion --depth 5
    #[arg(long)]
    pub depth: Option<usize>,

    /// Skip gitignored files (enabled by default).
    ///
    /// Example:
    ///   cargo fusion --skip-gitignore false
    #[arg(long, default_value_t = true)]
    pub skip_gitignore: bool,

    /// Measure performance and print statistics without writing output.
    #[arg(short = 'I', long, action)]
    pub info: bool,

    /// Include project dependencies in the output.
    ///
    /// WARNING: This will significantly increase the output size.
    #[arg(short, long, action)]
    pub dependencies: bool,

    /// Separator shown between files.
    ///
    /// Example:
    ///   cargo fusion --separator "// File: "
    #[arg(long, default_value = "//")]
    pub separator: String,

    /// Exclude files modified before the specified datetime.
    ///
    /// Format: "YYYY-MM-DD HH:MM:SS"
    ///
    /// Example:
    ///   cargo fusion --newer-than "2021-01-01 00:00:00"
    #[arg(long)]
    pub newer_than: Option<NaiveDateTime>,

    /// Exclude files modified after the specified datetime.
    ///
    /// Format: "YYYY-MM-DD HH:MM:SS"
    ///
    /// Example:
    ///   cargo fusion --older-than "2024-01-01 00:00:00"
    #[arg(long)]
    pub older_than: Option<NaiveDateTime>,

    /// Exclude files larger than the specified size in bytes.
    ///
    /// Example:
    ///   cargo fusion --smaller-than 1000000
    #[arg(long)]
    pub smaller_than: Option<u64>,

    /// Exclude files smaller than the specified size in bytes.
    ///
    /// Example:
    ///   cargo fusion --larger-than 1000
    #[arg(long)]
    pub larger_than: Option<u64>,

    /// Maximum number of files to include.
    ///
    /// Example:
    ///   cargo fusion --max-files 100
    #[arg(long)]
    pub max_files: Option<usize>,

    /// Additional paths (files or directories) to include.
    ///
    /// Example:
    ///   cargo fusion --include "file1.rs" --include "util/components"
    #[arg(short, long)]
    pub include: Vec<PathBuf>,

    /// File extensions to include (defaults to "rs").
    ///
    /// Example:
    ///   cargo fusion --extension toml
    #[arg(short = 'E', long, default_values_t = ["rs".to_string()])]
    pub extension: Vec<String>,

    /// Files to exclude from the output.
    ///
    /// Example:
    ///   cargo fusion --exclude "file1.rs" --exclude "file2.rs"
    #[arg(short, long)]
    pub exclude: Vec<String>,

    /// Include project metadata at the top of the output.
    #[arg(long, default_value_t = true)]
    pub include_metadata: bool,

    /// Include the `Cargo.lock` file in the output.
    #[arg(long)]
    pub include_lock: bool,
}

// \\?\D:\github\cargo-fusion\src\filter.rs
use chrono::{DateTime, NaiveDateTime, Utc};
use std::path::PathBuf;

use crate::args::FusionArgs;

/// Encapsulates all file-filtering logic in one place,
/// avoiding the long parameter list that `filter_path` previously had.
pub struct FileFilter<'a> {
    extension: &'a [String],
    smaller_than: Option<u64>,
    larger_than: Option<u64>,
    newer_than: Option<NaiveDateTime>,
    older_than: Option<NaiveDateTime>,
    include_lock: bool,
}

impl<'a> FileFilter<'a> {
    pub fn from_args(args: &'a FusionArgs) -> Self {
        Self {
            extension: &args.extension,
            smaller_than: args.smaller_than,
            larger_than: args.larger_than,
            newer_than: args.newer_than,
            older_than: args.older_than,
            include_lock: args.include_lock,
        }
    }

    /// Returns `Some(path)` if the entry passes all filters, `None` otherwise.
    pub fn apply(&self, entry: &ignore::DirEntry) -> Option<PathBuf> {
        let path = entry.path();

        if !self.include_lock && path.file_name() == Some(std::ffi::OsStr::new("Cargo.lock")) {
            return None;
        }

        let has_valid_extension = self.extension.iter().any(|ext| {
            path.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == ext)
        });

        if !has_valid_extension {
            return None;
        }

        if !self.passes_metadata_filter(entry) {
            return None;
        }

        Some(path.to_path_buf())
    }

    fn passes_metadata_filter(&self, entry: &ignore::DirEntry) -> bool {
        let needs_check = self.smaller_than.is_some()
            || self.larger_than.is_some()
            || self.newer_than.is_some()
            || self.older_than.is_some();

        if !needs_check {
            return true;
        }

        let Ok(meta) = entry.metadata() else {
            eprintln!(
                "Warning: could not read metadata for {}",
                entry.path().display()
            );
            return false;
        };

        if let Some(max_size) = self.smaller_than
            && meta.len() > max_size
        {
            return false;
        }

        if let Some(min_size) = self.larger_than
            && meta.len() < min_size
        {
            return false;
        }

        if self.newer_than.is_some() || self.older_than.is_some() {
            let modified: DateTime<Utc> = match meta.modified() {
                Ok(m) => m.into(),
                Err(_) => return false,
            };

            if let Some(newer) = self.newer_than
                && modified < newer.and_utc()
            {
                return false;
            }

            if let Some(older) = self.older_than
                && modified > older.and_utc()
            {
                return false;
            }
        }

        true
    }
}

// \\?\D:\github\cargo-fusion\src\main.rs
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

// \\?\D:\github\cargo-fusion\src\metadata.rs
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Project metadata included at the top of the generated file.
#[derive(Debug)]
pub struct ProjectMetadata {
    name: String,
    version: String,
    description: Option<String>,
    readme: Option<String>,
    repository: Option<String>,
    authors: Vec<String>,
    license: Option<String>,
}

impl ProjectMetadata {
    pub fn from_manifest(manifest_path: &Path) -> Result<Self> {
        let manifest =
            cargo_toml::Manifest::from_path(manifest_path).context("Failed to read Cargo.toml")?;

        let package = manifest
            .package
            .context("No [package] section found in Cargo.toml")?;

        let readme = Self::read_readme(manifest_path, &package);

        Ok(Self {
            name: package.name.clone(),
            version: package.version().to_string(),
            description: package.description().map(ToString::to_string),
            readme,
            repository: package.repository().map(ToString::to_string),
            authors: package.authors().to_vec(),
            license: package.license().map(ToString::to_string),
        })
    }

    fn read_readme(manifest_path: &Path, package: &cargo_toml::Package) -> Option<String> {
        let parent = manifest_path.parent()?;

        // Prefer the readme path declared in Cargo.toml.
        if let Some(readme_path) = package.readme().as_path() {
            let full = parent.join(readme_path);
            if let Ok(text) = fs::read_to_string(&full) {
                return Some(text);
            }
        }

        // Fall back to common README filenames.
        ["README.md", "README", "Readme.md", "README.txt"]
            .iter()
            .find_map(|name| fs::read_to_string(parent.join(name)).ok())
    }
}

impl std::fmt::Display for ProjectMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "// Project: {} (v{})", self.name, self.version)?;

        if let Some(desc) = &self.description {
            writeln!(f, "// Description: {desc}")?;
        }
        if !self.authors.is_empty() {
            writeln!(f, "// Authors: {}", self.authors.join(", "))?;
        }
        if let Some(license) = &self.license {
            writeln!(f, "// License: {license}")?;
        }
        if let Some(repo) = &self.repository {
            writeln!(f, "// Repository: {repo}")?;
        }
        if let Some(readme) = &self.readme {
            writeln!(f, "\n// README\n// ======")?;
            for line in readme.lines() {
                writeln!(f, "// {line}")?;
            }
            writeln!(f, "// ======")?;
        }

        Ok(())
    }
}

// \\?\D:\github\cargo-fusion\src\workspace.rs
//! Resolves the correct manifest and source scope for cargo-fusion.
//!
//! Two modes, detected automatically:
//!
//! 1. **Workspace mode** – the given `Cargo.toml` owns a `[workspace]` section.
//!    Parse and search the entire workspace tree.
//!
//! 2. **Crate mode** – the given `Cargo.toml` is a workspace member (no `[workspace]`).
//!    Walk up to find the workspace root so `cargo_toml` can resolve inherited fields,
//!    but restrict the file search to the **crate directory only**.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct WorkspaceResolution {
    /// Always the workspace-root `Cargo.toml` (or the original path if no workspace was found).
    pub manifest_path: PathBuf,

    /// Directory used as the search root for source files.
    ///
    /// - Crate mode  → the crate directory (e.g. `crates/my-crate`)
    /// - Workspace mode → the workspace root directory
    pub search_root: PathBuf,

    pub is_crate_mode: bool,
}

pub fn resolve_workspace_manifest(manifest_path: &Path) -> Result<WorkspaceResolution> {
    let canonical = manifest_path
        .canonicalize()
        .with_context(|| format!("cannot resolve: {}", manifest_path.display()))?;

    let crate_dir = canonical
        .parent()
        .context("Cargo.toml has no parent directory")?
        .to_path_buf();

    // If this manifest already declares [workspace], we are at the root.
    if manifest_declares_workspace(&canonical)? {
        return Ok(WorkspaceResolution {
            manifest_path: canonical,
            search_root: crate_dir,
            is_crate_mode: false,
        });
    }

    // Otherwise walk up to find the workspace root that lists this crate as a member.
    let mut current = crate_dir.parent();
    while let Some(dir) = current {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() && workspace_lists_member(&candidate, &crate_dir)? {
            eprintln!(
                "info: workspace root at {}, collecting only crate at {}",
                candidate.display(),
                crate_dir.display(),
            );
            return Ok(WorkspaceResolution {
                manifest_path: candidate,
                search_root: crate_dir,
                is_crate_mode: true,
            });
        }
        current = dir.parent();
    }

    // No workspace found — treat the manifest as standalone.
    Ok(WorkspaceResolution {
        manifest_path: canonical,
        search_root: crate_dir,
        is_crate_mode: false,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the manifest file contains a `[workspace]` table.
///
/// We use `cargo_toml` for parsing instead of scanning raw text, which avoids
/// false positives from comments like `# [workspace]`.
fn manifest_declares_workspace(path: &Path) -> Result<bool> {
    let manifest = cargo_toml::Manifest::from_path(path)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(manifest.workspace.is_some())
}

/// Returns `true` when `candidate_manifest` is a workspace root that lists
/// `crate_dir` among its members.
fn workspace_lists_member(candidate_manifest: &Path, crate_dir: &Path) -> Result<bool> {
    let manifest = cargo_toml::Manifest::from_path(candidate_manifest)
        .with_context(|| format!("failed to parse {}", candidate_manifest.display()))?;

    let Some(workspace) = &manifest.workspace else {
        return Ok(false);
    };

    let workspace_dir = candidate_manifest
        .parent()
        .context("workspace Cargo.toml has no parent")?;

    let members = match &workspace.members {
        members if members.is_empty() => {
            // Virtual workspace with no explicit members covers everything.
            return Ok(true);
        }
        members => members,
    };

    for pattern in members {
        let glob_pattern = workspace_dir.join(pattern);
        if glob_matches(&glob_pattern, crate_dir, workspace_dir) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Checks whether `pattern_path` (which may contain `*`) matches `crate_dir`.
fn glob_matches(pattern_path: &Path, crate_dir: &Path, workspace_dir: &Path) -> bool {
    let Some(pattern_str) = pattern_path.to_str() else {
        return false;
    };

    if !pattern_str.contains('*') {
        // Exact path — canonicalize both sides for a reliable comparison.
        return pattern_path
            .canonicalize()
            .is_ok_and(|cp| crate_dir == cp || crate_dir.starts_with(&cp));
    }

    // Glob: compare relative paths with forward slashes so the logic is
    // platform-independent.
    let rel = match crate_dir.strip_prefix(workspace_dir) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => return false,
    };

    let ws_str = workspace_dir.to_string_lossy().replace('\\', "/");
    let pattern_rel = pattern_str
        .replace('\\', "/")
        .trim_start_matches(ws_str.as_str())
        .trim_start_matches('/')
        .to_string();

    simple_glob_matches(&pattern_rel, &rel)
}

/// Minimal single-`*` glob matching for workspace member patterns like `crates/*`.
///
/// A `*` matches any sequence of characters that does **not** contain `/`,
/// which mirrors Cargo's own behaviour for workspace member globs.
fn simple_glob_matches(pattern: &str, input: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == input,
        Some((prefix, suffix)) => {
            if !input.starts_with(prefix) {
                return false;
            }
            let rest = &input[prefix.len()..];
            // `*` must not span directory boundaries.
            let segment = match rest.split_once('/') {
                Some((seg, _)) => seg,
                None => rest,
            };
            suffix.is_empty() || segment.ends_with(suffix.trim_start_matches('/'))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::simple_glob_matches;

    #[test]
    fn glob_exact_segment() {
        assert!(simple_glob_matches("crates/*", "crates/foo"));
    }

    #[test]
    fn glob_does_not_cross_slash() {
        assert!(!simple_glob_matches("crates/*", "crates/foo/bar"));
    }

    #[test]
    fn glob_no_wildcard() {
        assert!(simple_glob_matches("crates/foo", "crates/foo"));
        assert!(!simple_glob_matches("crates/foo", "crates/bar"));
    }

    #[test]
    fn glob_suffix() {
        assert!(simple_glob_matches("*-core", "my-core"));
        assert!(!simple_glob_matches("*-core", "my-core/sub"));
    }
}

