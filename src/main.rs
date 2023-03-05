mod data;

use clap::{Parser, Subcommand};
use crate::data::init;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// creates a new empty repository
    Init,
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    match &cli.command {
        Some(Commands::Init) => {
            if let Some(rgit_path) = init().unwrap() {
                println!("Initialized empty rgit repository in {:?}", rgit_path);
            } else {
                println!(".rgit folder already exists! Please not init again!")
            }
        },
        None => {}
    }
}
