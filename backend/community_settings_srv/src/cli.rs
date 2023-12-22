use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Root folder to store contributed setting files
    #[arg(short, long, default_value = "./community_settings")]
    pub folder: std::path::PathBuf,

    /// Server port
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Log file location
    #[arg(short, long, default_value = "/tmp/powertools_community_settings_srv.log")]
    pub log: std::path::PathBuf,
}

impl Cli {
    pub fn get() -> Self {
        Self::parse()
    }
}
