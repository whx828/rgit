mod base;
mod data;

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::PathBuf;

// 本地仓库

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
    },
    WriteTree,
    ReadTree {
        #[arg(short, long)]
        tree: String,
    },
    Commit {
        #[arg(short, long)]
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    match &cli.command {
        Some(Commands::Init) => {
            if File::open(data::GIT_DIR).is_ok() {
                println!("Already initialized rgit repository! Please don't again.");
                return;
            }

            if data::init().is_ok() {
                let mut rgit_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                rgit_path.push(data::GIT_DIR);
                println!("Initialized empty rgit repository in {:#?}", rgit_path);
            }
        }
        Some(Commands::HashObject { filename }) => {
            let mut file = File::open(filename).expect("input file not exist");
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let oid = data::hash_object(&contents, "blob");
            println!("{oid}");
        }
        Some(Commands::CatFile { object }) => {
            // let out_str = data::get_object(object, Some("blob"));
            let out_str = data::get_object(object, None);
            stdout().flush().unwrap();
            print!("{out_str}");
        }
        Some(Commands::WriteTree) => {
            let oid = base::write_tree();
            println!("{oid}");
        }
        Some(Commands::ReadTree { tree }) => base::read_tree(tree),
        Some(Commands::Commit { message }) => {
            let commit_oid = base::commit(message);
            println!("{commit_oid}");
        }
        None => {}
    }
}
