use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: RootCommands,
}

#[derive(Subcommand)]
pub enum RootCommands {
    /// Update CloudSQL instance's authorized networks
    Network {
        #[command(subcommand)]
        command: Option<NetworkCommands>,
    },
}

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Updates the authorized network with your current IP address in /24 CIDR notation
    Update {
        /// Repeats the last update operation using your current IP
        #[arg(short, long)]
        repeat_last: bool,
    },
}
