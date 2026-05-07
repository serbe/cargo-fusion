use std::path::PathBuf;

use anyhow::{Result, bail};
use chrono::NaiveDateTime;
use clap::Parser;
use tracing::info;

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
    about = "Generate a single file that contains all the source code of a Rust project.
Mainly intended to pipe source code into an LLM."
)]
#[command(version, long_about=None)]
pub struct FusionArgs {
    /// Output to stdout instead of a file.
    /// If this flag is set, the `output` option is ignored.
    ///
    /// Example:
    ///   cargo fusion --stdout
    #[arg(long)]
    pub stdout: bool,

    /// Include a table of contents at the top of the output.
    /// This will list all the files included in the output.
    ///
    /// Example:
    ///  cargo fusion --table-of-contents
    #[arg(long, action)]
    pub table_of_contents: bool,

    /// Optional path to the output file.
    ///
    /// Example:
    ///   cargo fusion -o ./output/combined.rs
    #[arg(short, long, default_value = "./fusion.rs")]
    pub output: PathBuf,

    /// Optional path to a `Cargo.toml` file.
    /// If not provided, the command will look for a `Cargo.toml` file in the current directory.
    ///
    /// Example:
    ///   cargo fusion -p ./path/to/Cargo.toml
    #[arg(short = 'p', long, default_value = "./Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Optional path to a header file.
    /// The contents of this file will be prepended to the output.
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

    /// Skip gitignored files.
    /// Enabled by default.
    ///
    /// Example:
    ///   cargo fusion --skip-gitignore false
    #[arg(long, default_value_t = true)]
    pub skip_gitignore: bool,

    /// Info mode.
    /// This flag is used to measure the performance of the command, as well as the number of files found and the number of lines of code.
    /// It will not write to a file or stdout.
    #[arg(short = 'I', long, action)]
    pub info: bool,

    /// Add the dependencies of the project to the output.
    ///
    /// WARNING: This will increase the size of the output significantly.
    #[arg(short, long, action)]
    pub dependencies: bool,

    /// The separator shown between files.
    ///
    /// Example:
    ///   cargo fusion --separator "// File: "
    #[arg(long, default_value = "//")]
    pub separator: String,

    /// Exclude files older than the specified datetime.
    ///
    /// Format: "YYYY-MM-DD HH:MM:SS"
    ///
    /// Will not work if `newer_than` is also set and is older than `older_than`.
    ///
    /// Example:
    ///  cargo fusion --older-than "2021-01-01 00:00:00"
    #[arg(long)]
    pub newer_than: Option<NaiveDateTime>,
    /// Exclude files newer than the specified datetime.
    ///
    /// Format: "YYYY-MM-DD HH:MM:SS"
    ///
    /// Will not work if `older_than` is also set and is newer than `newer_than`.
    ///
    /// Example:
    ///   cargo fusion --newer-than "2021-01-01 00:00:00"
    #[arg(long)]
    pub older_than: Option<NaiveDateTime>,

    /// Exclude files larger than the specified size in bytes.
    ///
    /// Will not work if `smaller_than` is also set and is larger than `larger_than`.
    ///
    /// Example:
    ///  cargo fusion --larger-than 1000000
    #[arg(long)]
    pub smaller_than: Option<u64>,

    /// Exclude files smaller than the specified size in bytes.
    ///
    /// Will not work if `larger_than` is also set and is smaller than `smaller_than`.
    ///
    /// Example:
    ///   cargo fusion --smaller-than 1000
    #[arg(long)]
    pub larger_than: Option<u64>,

    /// Max number of files to include in the output.
    /// If the number of files found exceeds this value, the command will ignore the rest of the files found past this number.
    ///
    /// Example:
    ///  cargo fusion --max-files 100
    #[arg(long)]
    pub max_files: Option<usize>,

    /// Add a path to include in the output
    ///
    /// If the path is a directory, all files in the directory will be included.
    ///
    /// Example:
    /// cargo fusion --include "file1.rs" --include "util/components"
    #[arg(short, long)]
    pub include: Vec<PathBuf>,

    /// Include files with the specified extension.
    /// Defaults to "rs".
    ///
    /// Example:
    ///  cargo fusion --extension toml
    #[arg(short = 'E', long, default_values_t=["rs".to_string()])]
    pub extension: Vec<String>,

    /// Exclude the specified files from the output.
    /// Accepts multiple values.
    ///
    /// Example:
    ///   cargo fusion --exclude "file1.rs" --exclude "file2.rs"
    #[arg(short, long)]
    pub exclude: Vec<String>,

    /// Include project metadata at the top of the output.
    #[arg(long, default_value_t = true)]
    pub include_metadata: bool,

    /// Include the `Cargo.lock` file in the output
    ///
    /// This is generally not wanted
    #[arg(long, default_value_t = false)]
    pub include_lock: bool,

    /// Example:
    ///   cargo fusion --verbose
    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long)]
    pub quiet: bool,
}

impl FusionArgs {
    pub(crate) fn verify_args(&self) -> Result<()> {
        info!("Arguments: {self:?}");

        // Используем zip для параллельной проверки
        if let (Some(smaller), Some(larger)) = (self.smaller_than, self.larger_than)
            && smaller > larger
        {
            bail!("`smaller_than` ({smaller}) cannot be larger than `larger_than` ({larger})");
        }

        if let (Some(newer), Some(older)) = (self.newer_than, self.older_than)
            && newer > older
        {
            bail!("`newer_than` ({newer}) cannot be older than `older_than` ({older})");
        }

        if !self.manifest_path.exists() {
            bail!("Cargo.toml not found at {}", self.manifest_path.display());
        }

        Ok(())
    }
}
