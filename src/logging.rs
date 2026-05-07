use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_logging(verbose: bool) -> Result<()> {
    let env_filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("cargo_fusion=debug,info"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("cargo_fusion=warn,info"))
    };

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stderr); // warnings/errors в stderr

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    subscriber.init();
    Ok(())
}
