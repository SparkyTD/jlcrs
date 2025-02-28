use clap::{Parser, Subcommand};

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Import a component from JLCPCB based on its LCSC code (e.g. C35879)
    Import {
        code: String,

        /// Allow updating existing components
        #[arg(short, long)]
        update: bool,

        /// Set a custom name for the library
        #[arg(short, long, default_value = "JLCPCB_Components")]
        name: String,

        /// Set a custom name for the library
        #[arg(short, long, default_value = "Components downloaded and converted directly from JLCPCB")]
        description: String,

        /// Root directory for the library (relative to project)
        #[arg(short, long)]
        root: Option<String>,
    }
}