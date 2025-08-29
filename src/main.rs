use crate::build_addon::{build_addon, build_all_addons};
use clap::{Parser, Subcommand};
use std::env;
use tokio::io;

mod bank;
mod build_addon;
mod utils;

#[derive(Parser)]
#[command(name = "devaforge")]
#[command(author = "Devaloop")]
#[command(about = "A tool to create and build banks/plugins/presets for Devalang")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage Banks
    Bank {
        #[command(subcommand)]
        command: BankCommands,
    },
}

#[derive(Subcommand)]
enum BankCommands {
    /// Scaffold a new bank
    Create {},

    /// Build banks
    Build {
        /// Relative path OR alias bank.<bankId>. Leave empty to build all.
        path: Option<String>,
        #[arg(short, long, default_value_t = false)]
        release: bool,
    },

    /// List available banks
    List {},

    /// Bump bank version
    Version {
        /// Bank identifier: <author>.<name>
        id: String,
        /// Bump type: major | minor | patch
        bump: String,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let cwd: String = env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    match cli.command {
        Commands::Bank { command } => match command {
            BankCommands::Create {} => {
                if let Err(e) = bank::prompt::prompt_bank_addon(&cwd).await {
                    eprintln!("Error creating bank: {}", e);
                }
                Ok(())
            }
            BankCommands::Build { path, release } => {
                match path {
                    Some(p) => {
                        if let Err(e) = build_addon(&p, &release, &cwd) {
                            eprintln!("Error building bank: {}", e);
                        }
                    }
                    None => {
                        if let Err(e) = build_all_addons(&release, &cwd) {
                            eprintln!("Error building banks: {}", e);
                        }
                    }
                }
                Ok(())
            }
            BankCommands::List {} => {
                if let Err(e) = bank::manage::list_banks(&cwd) {
                    eprintln!("Error listing banks: {}", e);
                }
                Ok(())
            }
            BankCommands::Version { id, bump } => {
                if let Err(e) = bank::manage::bump_version(&cwd, &id, &bump) {
                    eprintln!("Error bumping version: {}", e);
                }
                Ok(())
            }
        },
    }
}
