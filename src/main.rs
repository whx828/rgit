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
        commit: String,
    },
    Tag {
        name: String,
        oid: Option<String>,
    },
    K,
    Branch {
        name: Option<String>,
        start_point: Option<String>,
    },
    Status,
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

            if base::init().is_ok() {
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
                let oid = data::get_ref("HEAD", true).value.unwrap();
                base::get_commit(&oid);
            }
        },
        Some(Commands::Checkout { commit }) => {
            base::checkout(commit);
        }
        Some(Commands::Tag { name, oid }) => match oid {
            Some(oid) => {
                let oid = base::get_oid(oid);
                base::create_tag(name, &oid);
            }
            None => {
                let oid = data::get_ref("HEAD", true).value.unwrap();
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
        Some(Commands::Branch { name, start_point }) => match name {
            Some(name) => match start_point {
                Some(sp) => {
                    let oid = base::get_oid(sp);
                    base::create_branch(name, &oid);
                    println!("Branch {name} created at {:?}", &sp[0..10]);
                }
                None => {
                    let oid = data::get_ref("HEAD", true).value.unwrap();
                    base::create_branch(name, &oid);
                    println!("Branch {name} created at {:?}", &oid[0..10]);
                }
            },
            None => match base::get_status_name() {
                Some(branch_name) => {
                    for name in base::iter_branch_names() {
                        if name == branch_name {
                            println!("*{branch_name}");
                        } else {
                            println!(" {name}");
                        }
                    }
                }
                None => {
                    let head = base::get_oid("@");
                    println!("HEAD detached at{}", &head[0..10]);
                }
            },
        },
        Some(Commands::Status) => {
            let head = base::get_oid("@");
            let branch = base::get_status_name();
            match branch {
                Some(branch_name) => println!("On branch {}", branch_name),
                None => println!("HEAD detached at{}", &head[0..10]),
            }
        }
        None => {}
    }
}
