use clap::Parser;

/// Finds TODOs in code and outputs them as JSON
#[derive(Parser)]
#[command(name = "todozer")]
#[command(version = "0.1")]
#[command(about = "Scan source files and extract TODOs", long_about = None)]

pub struct Cli {
    /// The path to the source code to scan
    #[arg(short, long, default_value = ".")]
    pub path: String,
}
