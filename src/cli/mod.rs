use clap::Parser;
#[derive(Parser, Debug)]
pub struct Cli {
    /// Name of the person to greet
    #[arg(short, long, default_value = "")]
    pub url: String,
}
