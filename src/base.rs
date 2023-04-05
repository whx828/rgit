use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crate::data;

pub fn write_tree() -> String {
    let rgit_path = PathBuf::from("./src");
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

            let e = format!("{} {} {:?}", rgit_type, oid, entry.file_name());
            entries.push(e);
        }

        for e in &entries {
            tree.push_str(e);
            tree.push('\n');
        }
    }

    data::hash_object(&tree, "tree")
}

fn is_dot_path(path: &Path) -> bool {
    format!("{path:?}").split('/').last().unwrap().as_bytes()[0] == b'.'
}
