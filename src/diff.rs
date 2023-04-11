use crate::{base, data};
use std::{io::Write, process::Command};
use tempfile::NamedTempFile;

pub fn compare_trees(oid: &str) -> Vec<(String, String)> {
    let binding = data::get_object(&oid, None);
    let parent_oid = binding.lines().nth(1).unwrap().split(' ').nth(1).unwrap();

    let binding = data::get_object(parent_oid, None);
    let parent_tree = binding.lines().next().unwrap().split(' ').nth(1).unwrap();

    let parent_file_content = data::get_object(parent_tree, None);

    let binding = data::get_object(&oid, None);
    let child_tree = binding.lines().next().unwrap().split(' ').nth(1).unwrap();

    let now_file_content = data::get_object(child_tree, None);

    diff_trees(parent_file_content, now_file_content)
}

pub fn diff_trees(parent: String, child: String) -> Vec<(String, String)> {
    // oid
    let parent = parent
        .lines()
        .map(|x| x.split(' ').map(|x| x.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();

    let child = child
        .lines()
        .map(|x| x.split(' ').map(|x| x.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();

    let mut diffs = vec![];

    for c in child.clone() {
        for p in parent.clone() {
            if p[0] == c[0] && p[2] == c[2] {
                if p[1] != c[1] {
                    if p[0] == String::from("blob") {
                        diffs.push((p[1].clone(), c[1].clone()));
                        break;
                    } else if p[0] == String::from("tree") {
                        let parent_file_content = data::get_object(&p[1], None);
                        let now_file_content = data::get_object(&c[1], None);

                        for i in diff_trees(parent_file_content, now_file_content) {
                            diffs.push(i);
                        }
                        break;
                    }
                }
            }
        }
    }

    for p in parent.clone() {
        if find_remove(p.clone(), child.clone()) {
            if p[0] == String::from("blob") {
                println!("remove file {}", p[2]);
            } else if p[0] == String::from("tree") {
                println!("remove folder {}", p[2]);
            }
        }
    }

    for c in child.clone() {
        if find_add(parent.clone(), c.clone()) {
            if c[0] == String::from("blob") {
                println!("add new file {}", c[2]);
            } else if c[0] == String::from("tree") {
                println!("add new folder {}", c[2]);
            }
        }
    }

    diffs
}

fn iter_changed_files(parent: String, child: String) {
    let parent = parent
        .lines()
        .map(|x| x.split(' ').map(|x| x.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();

    let child = child
        .lines()
        .map(|x| x.split(' ').map(|x| x.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();

    for c in child.clone() {
        for p in parent.clone() {
            if p[0] == c[0] && p[2] == c[2] {
                if p[1] != c[1] {
                    if p[0] == String::from("blob") {
                        println!("modified file {}", p[2]);
                        break;
                    } else if p[0] == String::from("tree") {
                        let parent_file_content = data::get_object(&p[1], None);
                        let now_file_content = data::get_object(&c[1], None);

                        println!("modified folder {}", p[2]);
                        diff_trees(parent_file_content, now_file_content);
                        break;
                    }
                }
            }
        }
    }

    for p in parent.clone() {
        if find_remove(p.clone(), child.clone()) {
            if p[0] == String::from("blob") {
                println!("remove file {}", p[2]);
            } else if p[0] == String::from("tree") {
                println!("remove folder {}", p[2]);
            }
        }
    }

    for c in child.clone() {
        if find_add(parent.clone(), c.clone()) {
            if c[0] == String::from("blob") {
                println!("add new file {}", c[2]);
            } else if c[0] == String::from("tree") {
                println!("add new folder {}", c[2]);
            }
        }
    }
}

fn find_remove(parent: Vec<String>, child: Vec<Vec<String>>) -> bool {
    for c in child {
        if parent[0] == c[0] {
            if parent[2] == c[2] {
                return false;
            }
        }
    }

    true
}

fn find_add(parent: Vec<Vec<String>>, child: Vec<String>) -> bool {
    for p in parent {
        if p[0] == child[0] {
            if p[2] == child[2] {
                return false;
            }
        }
    }

    true
}

pub fn get_working_tree_diff(oid: &str) -> Vec<(String, String)> {
    let now_tree = base::write_tree();
    let now_uncommit_file_content = data::get_object(&now_tree, Some("tree"));

    let binding = data::get_object(&oid, None);
    let child_tree = binding.lines().next().unwrap().split(' ').nth(1).unwrap();

    let now_commit_file_content = data::get_object(child_tree, None);

    diff_trees(now_commit_file_content, now_uncommit_file_content)
}

pub fn get_working_tree_diff_simple_version(oid: &str) {
    let now_tree = base::write_tree();
    let now_uncommit_file_content = data::get_object(&now_tree, Some("tree"));

    let binding = data::get_object(&oid, None);
    let child_tree = binding.lines().next().unwrap().split(' ').nth(1).unwrap();

    let now_commit_file_content = data::get_object(child_tree, None);

    iter_changed_files(now_commit_file_content, now_uncommit_file_content);
}

pub fn merge(tree1_oid: &str, tree2_oid: &str) -> String {
    let binding = data::get_object(tree1_oid, None);
    let child_tree = binding.lines().next().unwrap().split(' ').nth(1).unwrap();

    let tree1_content = data::get_object(child_tree, None);

    let binding = data::get_object(tree2_oid, None);
    let child_tree = binding.lines().next().unwrap().split(' ').nth(1).unwrap();

    let tree2_content = data::get_object(child_tree, None);
    merge_diff_trees(tree1_content, tree2_content)
}

fn merge_diff_trees(parent: String, child: String) -> String {
    let parent = parent
        .lines()
        .map(|x| x.split(' ').map(|x| x.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();

    let child = child
        .lines()
        .map(|x| x.split(' ').map(|x| x.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();

    let mut entries = vec![];
    let mut rgit_type = String::from("");
    let mut oid = String::from("");
    let mut filename = String::from("");
    let mut tree = String::from("");

    for c in child.clone() {
        for p in parent.clone() {
            if p[0] == c[0] && p[2] == c[2] {
                if p[1] != c[1] {
                    if p[0] == String::from("blob") {
                        rgit_type = p[0].clone();

                        let new_file_content = merge_blobs(&p[1], &c[1]);
                        oid = data::hash_object(&new_file_content, "blob");
                        filename = p[2].clone();
                    } else if p[0] == String::from("tree") {
                        rgit_type = p[0].clone();
                        let parent_file_content = data::get_object(&p[1], None);
                        let now_file_content = data::get_object(&c[1], None);

                        oid = merge_diff_trees(parent_file_content, now_file_content);
                        filename = p[2].clone();
                    }

                    let e = format!("{} {} {}", rgit_type, oid, filename);
                    entries.push(e);
                    break;
                } else {
                    let e = format!("{} {} {}", p[0], p[1], p[2]);
                    entries.push(e);
                    break;
                }
            }
        }
    }

    for e in &entries {
        tree.push_str(e);
        tree.push('\n');
    }

    data::hash_object(&tree, "tree")
}

fn merge_blobs(blob1_oid: &str, blob2_oid: &str) -> String {
    let arg1 = format!("{}", data::get_object(blob1_oid, Some("blob")));
    let arg2 = format!("{}", data::get_object(blob2_oid, Some("blob")));

    let mut file1 = NamedTempFile::new().unwrap();
    file1.write_all(arg1.as_bytes()).unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    file2.write_all(arg2.as_bytes()).unwrap();

    let output = Command::new("diff")
        .args([
            "-DHEAD",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to spawn child process");

    String::from_utf8(output.stdout).unwrap()
}
