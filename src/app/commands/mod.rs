use crate::app::cli::{Cli, Commands};
use crate::core::errors::Result;

pub async fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::List(_args)) => {
            println!("pdown list - available in Phase 2");
            Ok(())
        }
        Some(Commands::Status(_args)) => {
            println!("pdown status - available in Phase 3");
            Ok(())
        }
        Some(Commands::Test(_args)) => {
            println!("pdown test - available in Phase 5");
            Ok(())
        }
        Some(Commands::Use(args)) => {
            println!("pdown use {} - available in Phase 4", args.mirror_id);
            Ok(())
        }
        Some(Commands::Reset(_args)) => {
            println!("pdown reset - available in Phase 4");
            Ok(())
        }
        Some(Commands::Doctor(_args)) => {
            println!("pdown doctor - available in Phase 6");
            Ok(())
        }
        Some(Commands::Mirror(_args)) => {
            println!("pdown mirror - available in Phase 2");
            Ok(())
        }
        Some(Commands::Registry(_args)) => {
            println!("pdown registry - available in Phase 2");
            Ok(())
        }
        Some(Commands::Profile(_args)) => {
            println!("pdown profile - available in Phase 6");
            Ok(())
        }
        Some(Commands::Config(_args)) => {
            println!("pdown config - available in Phase 6");
            Ok(())
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
            println!();
            Ok(())
        }
    }
}
