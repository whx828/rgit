use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crate::data;

pub fn write_tree() -> String {
    let rgit_path = PathBuf::from("./test");
    visit_dirs(&rgit_path)
}

// one possible implementation of walking a directory only visiting files
// https://doc.rust-lang.org/std/fs/fn.read_dir.html
fn visit_dirs(dir: &Path) -> String {
    let mut entries = vec![];
    let mut rgit_type;
    let mut oid;
    let mut tree = String::from("");

    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if is_dot_path(&path) {
                continue;
            }

            if path.is_dir() {
                rgit_type = "tree";
                oid = visit_dirs(&path);
            } else {
                let mut file = File::open(path).expect("input file not exist");
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                rgit_type = "blob";
                oid = data::hash_object(&contents, rgit_type);
            }

            let e = format!(
                "{} {} {}",
                rgit_type,
                oid,
                entry.file_name().into_string().unwrap()
            );
            entries.push(e);
        }

        for e in &entries {
            tree.push_str(e);
            tree.push('\n');
        }
    }

    data::hash_object(&tree, "tree")
}

fn empty_current_directory(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry.unwrap().path();

            if is_dot_path(&path) {
                continue;
            }

            if path.is_dir() {
                empty_current_directory(&path)?;
            } else if path.is_file() {
                fs::remove_file(path)?;
            }
        }

        fs::remove_dir(dir)?;
    }

    // } else if dir.is_file() {
    //     fs::remove_file(dir)?;
    // }

    Ok(())
}

fn iter_tree_entries(oid: &str, base_path: &str) -> (HashSet<String>, HashMap<String, String>) {
    let mut dirs = HashSet::new();
    let mut files = HashMap::new();

    let content = data::get_object(oid, Some("tree"));
    for entry in content.lines() {
        let mut e = entry.split_whitespace().collect::<Vec<&str>>();

        let name = e.pop().unwrap();
        let mut path = base_path.to_string();
        path.push_str(name);

        let oid = e.pop().unwrap();
        let rgit_type = e.pop().unwrap();

        if rgit_type == "blob" {
            files.insert(path, oid.to_string());
        } else if rgit_type == "tree" {
            dirs.insert(path.to_string());

            let (subdirs, subfiles) = iter_tree_entries(oid, &format!("{path}/"));
            for p in subdirs {
                dirs.insert(p);
            }
            for (p, o) in subfiles {
                files.insert(p, o);
            }
        } else {
            panic!("Unknown entry")
        }
    }

    (dirs, files)
}

pub fn read_tree(tree: &str) {
    let path = PathBuf::from("./test");
    empty_current_directory(&path).unwrap();

    let (dirs, files) = iter_tree_entries(tree, "./test/");
    data::mkdir("./test").unwrap();

    let mut dirs = dirs.into_iter().collect::<Vec<String>>();
    dirs.sort();
    for p in dirs {
        data::mkdir(&p).unwrap();
    }

    for (p, o) in files {
        data::mkfile(&p, data::get_object(&o, None).as_bytes()).unwrap();
    }
}

fn is_dot_path(path: &Path) -> bool {
    format!("{path:?}").split('/').last().unwrap().as_bytes()[0] == b'.'
}
