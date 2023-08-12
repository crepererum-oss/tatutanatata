//! Logging setup.
use anyhow::Result;
use clap::Parser;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Logging CLI config.
#[derive(Debug, Parser)]
pub struct LoggingCLIConfig {
    /// Log verbosity.
    #[clap(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
    )]
    log_verbose_count: u8,
}

/// Setup process-wide logging.
pub fn setup_logging(config: LoggingCLIConfig) -> Result<()> {
    LogTracer::init()?;

    let base_filter = match config.log_verbose_count {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_new(format!("{base_filter},hyper=info"))?;

    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
