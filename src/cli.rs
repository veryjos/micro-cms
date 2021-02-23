use clap::Clap;

/// CLI arguments for an instance of micro-cms.
#[derive(Clap, Debug)]
#[clap(name="micro-cms")]
pub struct CliArgs {
    /// Path to content folder.
    #[clap(short, long)]
    pub content_path: String,

    /// Binding address.
    #[clap(short, long, default_value = "0.0.0.0")]
    pub address: String,

    /// Binding port.
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
}

impl CliArgs {
    /// Parses CLI arguments and returns an instance of [CliArgs].
    pub fn from_cli() -> CliArgs {
        CliArgs::parse()
    }
}
