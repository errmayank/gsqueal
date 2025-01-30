use clap::Parser;
use gsqueal::commands::network;
use gsqueal::models::cli::{Cli, NetworkCommands, RootCommands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(RootCommands::Network { command }) => match command {
            Some(NetworkCommands::Update { repeat_last }) => {
                network::update(repeat_last).await;
            }
            None => {
                panic!("No network subcommand provided. Use --help to see available options.")
            }
        },
        None => {}
    }
}
