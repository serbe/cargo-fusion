use chrono::NaiveDateTime;
use clap::Parser;
use std::path::PathBuf;

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
    about = "Generate a single file containing all source code of a Rust project.\nMainly intended to pipe source code into an LLM."
)]
pub struct FusionArgs {
    #[arg(long)]
    pub stdout: bool,

    #[arg(long, action)]
    pub table_of_contents: bool,

    #[arg(short, long, default_value = "./cargo-fusion-output.rs")]
    pub output: PathBuf,

    #[arg(short = 'p', long, default_value = "./Cargo.toml")]
    pub manifest_path: PathBuf,

    #[arg(long)]
    pub head: Option<PathBuf>,

    #[arg(long)]
    pub depth: Option<usize>,

    #[arg(long, default_value_t = true)]
    pub skip_gitignore: bool,

    #[arg(short = 'I', long, action)]
    pub info: bool,

    #[arg(short, long, action)]
    pub dependencies: bool,

    #[arg(long, default_value = "//")]
    pub separator: String,

    #[arg(long)]
    pub newer_than: Option<NaiveDateTime>,

    #[arg(long)]
    pub older_than: Option<NaiveDateTime>,

    #[arg(long)]
    pub smaller_than: Option<u64>,

    #[arg(long)]
    pub larger_than: Option<u64>,

    #[arg(long)]
    pub max_files: Option<usize>,

    #[arg(short, long)]
    pub include: Vec<PathBuf>,

    #[arg(short = 'E', long, default_values_t = ["rs".to_string()])]
    pub extension: Vec<String>,

    #[arg(short, long)]
    pub exclude: Vec<String>,

    #[arg(long, default_value_t = true)]
    pub include_metadata: bool,

    #[arg(long)]
    pub include_lock: bool,
}
