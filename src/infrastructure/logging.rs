use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize application logging with an optional verbosity flag.
pub fn init_logging(verbose: bool, quiet: bool) {
    if quiet {
        return;
    }

    let default_filter = if verbose {
        "pdown=debug,info"
    } else {
        "pdown=info,warn"
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).compact())
        .try_init();
}
