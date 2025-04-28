mod cli;
mod scanner;
mod git;
mod serialize;

use cli::Cli;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    scanner::run(&args);
}
