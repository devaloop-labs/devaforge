use crate::{builder::bank as bank_builder, utils::version::get_version};
use clap::{Parser, Subcommand};
use std::env;
use tokio::io;

mod addon;
mod builder;
mod utils;

#[derive(Parser)]
#[command(name = "devaforge")]
#[command(version = get_version())]
#[command(author = "Devaloop")]
#[command(about = "A tool to create and build banks/plugins/presets/templates for Devalang")]
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
    
    /// Delete a generated bank
    Delete {
        /// Bank identifier: <author>.<name>
        id: String,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let cwd: String = env::current_dir()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get current dir: {}", e),
            )
        })?
        .into_os_string()
        .into_string()
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Current directory contains invalid UTF-8",
            )
        })?;

    match cli.command {
        Commands::Bank { command } => match command {
            BankCommands::Create {} => {
                if let Err(e) = addon::bank::prompt::prompt_bank_addon(&cwd).await {
                    eprintln!("Error creating bank: {}", e);
                }

                println!();
                println!("Bank created successfully.");
                println!();

                Ok(())
            }
            BankCommands::Build { path } => {
                match path {
                    Some(p) => {
                        if let Err(e) = bank_builder::build_bank(&p, &cwd) {
                            eprintln!("Error building bank: {}", e);
                        }
                    }
                    None => {
                        if let Err(e) = bank_builder::build_all_banks(&cwd) {
                            eprintln!("Error building banks: {}", e);
                        }
                    }
                }
                Ok(())
            }
            BankCommands::List {} => {
                if let Err(e) = addon::bank::manage::list_banks(&cwd) {
                    eprintln!("Error listing banks: {}", e);
                }
                Ok(())
            }
            BankCommands::Version { id, bump } => {
                if let Err(e) = addon::bank::manage::bump_version(&cwd, &id, &bump) {
                    eprintln!("Error bumping version: {}", e);
                }
                Ok(())
            }
            BankCommands::Delete { id } => {
                if let Err(e) = addon::bank::manage::delete_bank(&cwd, &id) {
                    eprintln!("Error deleting bank: {}", e);
                }
                Ok(())
            }
        },
    }
}
