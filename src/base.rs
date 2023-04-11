use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use core::str;
use hex::FromHex;
use tempfile::NamedTempFile;

use crate::data;
use crate::data::RefValue;
use crate::data::GIT_DIR;
use crate::diff;

pub fn init() -> io::Result<()> {
    data::init()?;

    let value = RefValue {
        symbolic: true,
        value: Some(String::from("refs/heads/master")),
    };
    data::set_ref("HEAD", value, false);

    Ok(())
}

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

pub fn commit(message: &str) -> String {
    let mut commit = "tree ".to_string();
    commit.push_str(&write_tree());
    commit.push('\n');

    if let Some(head) = data::get_ref("HEAD", true).value {
        commit.push_str("parent ");
        commit.push_str(&head);
        commit.push('\n');
    }

    commit.push('\n');
    commit.push_str(message);
    commit.push('\n');

    let oid = data::hash_object(&commit, "commit");
    let tmp = RefValue::new(Some(oid.clone()));
    data::set_ref("HEAD", tmp, true);

    oid
}

pub fn checkout(name: &str) {
    let oid = get_oid(name);
    let commit = data::get_object(&oid, Some("commit"));

    let tree = commit.lines().collect::<Vec<&str>>()[0]
        .split_whitespace()
        .nth(1)
        .unwrap();

    read_tree(tree);

    let tmp = if is_branch(name) {
        let value = format!("refs/heads/{name}");
        data::RefValue {
            symbolic: true,
            value: Some(value),
        }
    } else {
        data::RefValue::new(Some(oid))
    };

    data::set_ref("HEAD", tmp, false);
}

fn is_branch(name: &str) -> bool {
    let rgit_ref = format!("refs/heads/{name}");
    data::get_ref(&rgit_ref, false).value.is_some()
}

pub fn iter_branch_names() -> Vec<String> {
    let path = format!("{GIT_DIR}/refs/heads/");

    fs::read_dir(path)
        .unwrap()
        .map(|res| res.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<String>>()
}

fn iter_branch_contents() -> Vec<(String, String)> {
    let mut contents = vec![];
    for name in iter_branch_names() {
        let path = format!("{GIT_DIR}/refs/heads/{name}");
        let mut file = File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        contents.push((name, content));
    }

    contents
}

pub fn get_status_name() -> Option<String> {
    let value = data::get_ref("HEAD", false);
    if !value.symbolic {
        return None;
    }

    let head = value.value.unwrap();
    assert!(head.starts_with("ref: refs/heads/"));

    Some(head.split("ref: refs/heads/").last().unwrap().to_string())
}

pub fn reset(oid: &str) {
    let value = RefValue::new(Some(oid.to_string()));
    data::set_ref("HEAD", value, true);
}

pub fn read_tree_merged(tree1: &str, tree2: &str) {
    let tree1_oid = get_oid(tree1);
    let tree2_oid = get_oid(tree2);
    let tree_oid = diff::merge(&tree1_oid, &tree2_oid);

    let mut commit = "tree ".to_string();
    commit.push_str(&tree_oid);
    commit.push('\n');

    commit.push_str("parent ");
    commit.push_str(&tree1_oid);
    commit.push('\n');

    commit.push_str("parent ");
    commit.push_str(&tree2_oid);
    commit.push('\n');

    commit.push('\n');
    commit.push_str("merge message");
    commit.push('\n');

    let oid = data::hash_object(&commit, "commit");
    let tmp = RefValue::new(Some(oid.clone()));
    data::set_ref(tree1, tmp, true);

    println!("{commit}");
    read_tree(&tree_oid);
}

pub fn create_tag(name: &str, oid: &str) {
    let tmp = RefValue::new(Some(oid.to_string()));
    data::set_ref(&format!("refs/tags/{name}"), tmp, true);
}

pub fn create_branch(name: &str, oid: &str) {
    let tmp = RefValue::new(Some(oid.to_string()));
    data::set_ref(&format!("refs/heads/{name}"), tmp, true);
}

pub fn print_commit(modi_contents: &Vec<(String, String)>) {
    for (i, j) in modi_contents {
        let arg1 = format!("{}", data::get_object(i, None));
        let arg2 = format!("{}", data::get_object(j, None));

        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(arg1.as_bytes()).unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(arg2.as_bytes()).unwrap();

        let mut child = Command::new("diff")
            .args([
                "--text",
                "--unified",
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ])
            .spawn()
            .expect("failed to spawn child process");

        child.wait().expect("failed to wait for child process");
    }
}

pub fn get_commit(oid: &str) {
    let branch_oids = iter_branch_contents();
    let mut refs = String::new();

    for (br, br_oid) in branch_oids {
        if br_oid == oid.to_string() {
            let b = format!("<- {br} ");
            refs.push_str(&b);
        }
    }

    let commit = data::get_object(oid, Some("commit"));
    println!("commit {oid} {refs}");

    let mut lines = commit.lines().collect::<Vec<&str>>();
    let message = lines.pop().unwrap();
    println!("    {message}\n");

    if let Some(parent_oid) = lines[1].split_whitespace().nth(1) {
        get_commit(parent_oid)
    }

    if lines.len() > 3 {
        println!("another parent ----------");
        if let Some(parent_oid) = lines[2].split_whitespace().nth(1) {
            get_commit(parent_oid)
        }
    }
}

pub fn get_oid(mut name: &str) -> String {
    if name == "@" {
        name = "HEAD";
    }

    let refs_to_try = vec![
        format!("{name}"),
        format!("refs/{name}"),
        format!("refs/tags/{name}"),
        format!("refs/heads/{name}"),
    ];

    for r in refs_to_try {
        if let Some(r) = data::get_ref(&r, true).value {
            return r;
        }
    }

    if let Some(oid) = data::get_ref(name, true).value {
        return oid;
    }

    if <[u8; 20]>::from_hex(name).is_ok() {
        return name.to_string();
    }

    unreachable!()
}

fn is_dot_path(path: &Path) -> bool {
    format!("{path:?}").split('/').last().unwrap().as_bytes()[0] == b'.'
}
