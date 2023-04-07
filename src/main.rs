mod base;
mod data;

use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
    Log {
        oid: Option<String>,
    },
    Checkout {
        #[arg(short, long)]
        oid: String,
    },
    Tag {
        name: String,
        oid: Option<String>,
    },
    K,
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
            let object = base::get_oid(object);
            let out_str = data::get_object(&object, None);
            stdout().flush().unwrap();
            print!("{out_str}");
        }
        Some(Commands::WriteTree) => {
            let oid = base::write_tree();
            println!("{oid}");
        }
        Some(Commands::ReadTree { tree }) => {
            let tree = base::get_oid(tree);
            base::read_tree(&tree)
        }
        Some(Commands::Commit { message }) => {
            let commit_oid = base::commit(message);
            println!("{commit_oid}");
        }
        Some(Commands::Log { oid }) => match oid {
            Some(oid) => {
                let oid = base::get_oid(oid);
                base::get_commit(&oid);
            }
            None => {
                let oid = data::get_ref("HEAD").unwrap();
                base::get_commit(&oid);
            }
        },
        Some(Commands::Checkout { oid }) => {
            let oid = base::get_oid(oid);
            base::checkout(&oid);
        }
        Some(Commands::Tag { name, oid }) => match oid {
            Some(oid) => {
                let oid = base::get_oid(oid);
                base::create_tag(name, &oid);
            }
            None => {
                let oid = data::get_ref("HEAD").unwrap();
                base::create_tag(name, &oid);
            }
        },
        Some(Commands::K) => {
            let mut sides = HashSet::new();
            let mut dot = String::from("digraph commits {\n");
            let entries = data::iter_refs();

            for (f, oids) in entries {
                for oid in oids.windows(2) {
                    let arrow_oid = format!("  {} -> {}\n", oid[0], oid[1]);
                    if sides.insert(arrow_oid.clone()) {
                        dot.push_str(&arrow_oid);
                    }
                }
                let f_arrow = format!("  {f} -> {}\n", oids[0]);
                dot.push_str(&f_arrow);
                let f_note = format!("  {f} [shape=note]\n");
                dot.push_str(&f_note);
            }
            dot.push('}');

            let mut child = Command::new("dot")
                .arg("-Tjpeg")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to spawn child process");

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(dot.as_bytes()).unwrap();
            }

            Command::new("open")
                .args(["-a", "Preview.app", "-f"])
                .stdin(Stdio::from(child.stdout.unwrap()))
                .spawn()
                .unwrap();
        }
        None => {}
    }
}
