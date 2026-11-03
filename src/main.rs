use clap::Parser;
use pdown::app::cli::Cli;
use pdown::app::commands;
use pdown::infrastructure::logging;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    logging::init_logging(cli.verbose, cli.quiet);

    if let Err(err) = commands::execute(cli).await {
        eprintln!("Error: {err}");
        std::process::exit(err.exit_code());
    }
}
