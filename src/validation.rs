use crate::args::FusionArgs;
use anyhow::{Result, bail};

pub fn validate_args(args: &FusionArgs) -> Result<()> {
    // Проверка размеров
    if let (Some(max), Some(min)) = (args.smaller_than, args.larger_than)
        && max < min
    {
        bail!(
            "`smaller_than` ({}) cannot be less than `larger_than` ({})",
            max,
            min
        );
    }

    // Проверка дат
    if let (Some(min_date), Some(max_date)) = (args.newer_than, args.older_than)
        && min_date > max_date
    {
        bail!(
            "`newer_than` ({}) cannot be later than `older_than` ({})",
            min_date,
            max_date
        );
    }

    // Проверка существования manifest
    if !args.manifest_path.exists() {
        bail!("Cargo.toml not found at {}", args.manifest_path.display());
    }

    Ok(())
}
