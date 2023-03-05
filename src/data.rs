use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

pub const GIT_DIR: &str = ".rgit";

/// Create a new directory.
fn mkdir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();

    fs::create_dir(path)?;
    Ok(())
}

/// Create a new file.
fn mkfile<P: AsRef<Path>>(path: P, data: &Vec<u8>) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;

    Ok(())
}

pub fn init() -> io::Result<()> {
    mkdir(GIT_DIR)?;

    let object = format!("{GIT_DIR}/objects");
    mkdir(object)
}

pub fn hash_object(data: &str, type_obj: &str) -> String {
    let mut obj = type_obj.as_bytes().to_owned();
    obj.push(b'\x00');
    obj.append(&mut data.as_bytes().to_owned());

    let oid = sha1_smol::Sha1::from(obj.clone()).digest().to_string();
    let path = format!("{GIT_DIR}/objects/{oid}");
    mkfile(path, &obj).expect("create failed");

    oid
}

pub fn get_object(oid: &str, expected: Option<&str>) -> String {
    let path = format!("{GIT_DIR}/objects/{oid}");
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut cont: Vec<_> = contents.split(b'\x00' as char).collect();
    let content = cont.pop().unwrap();
    let type_obj= cont.pop().unwrap();

    if !expected.is_none() {
        assert_eq!(type_obj, expected.unwrap());
    }

    content.to_string()
}
