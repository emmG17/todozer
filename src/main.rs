mod cli;
mod scanner;

use cli::Cli;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    println!("Path: {}", args.path);
    scanner::run(&args);
}
