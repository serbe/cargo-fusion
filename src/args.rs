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
