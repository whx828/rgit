mod data;

use std::fs::File;
use std::io::{Read, stdout, Write};
use std::path::PathBuf;
use clap::{Parser, Subcommand};

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
    HashObject {
        #[arg(short, long)]
        filename: String,
    },
    CatFile {
        #[arg(short, long)]
        object: String,
    }
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    match &cli.command {
        Some(Commands::Init) => {
            if let Ok(_) = data::init() {
                let mut rgit_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                rgit_path.push(data::GIT_DIR);
                println!("Initialized empty rgit repository in {:#?}", rgit_path);
            }
        },
        Some(Commands::HashObject { filename}) => {
            let mut file = File::open(filename).expect("input file not exist");
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let oid = data::hash_object(&contents, "blob");
            println!("{oid}");
        },
        Some(Commands::CatFile { object }) => {
            let out_str = data::get_object(object, Some("blob"));
            stdout().flush().unwrap();
            print!("{out_str}");
        }
        None => {}
    }
}
