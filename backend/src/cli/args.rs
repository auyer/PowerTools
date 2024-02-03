use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// TCP port for front-end client connection
    #[arg(long)]
    pub port: Option<u16>,

    /// Override log file location
    #[arg(long)]
    pub log: Option<std::path::PathBuf>,

    /// Force verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Specail operation to perform
    #[command(subcommand)]
    pub op: Option<Operation>,
}

impl Args {
    pub fn load() -> Self {
        Self::parse()
    }

    pub fn is_default(&self) -> bool {
        self.port.is_none() && self.log.is_none() && !self.verbose && self.op.is_none()
    }
}

#[derive(Subcommand, Debug)]
pub enum Operation {
    /// Dump useful system information for adding new device support
    DumpSys,
    /// Remove all files created by PowerTools, not including $HOME/homebrew/plugins/PowerTools/
    Clean,
}
