use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

pub const GIT_DIR: &str = ".rgit";

pub fn mkdir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();

    fs::create_dir(path)?;
    Ok(())
}

pub fn mkfile<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;

    Ok(())
}

pub fn init() -> io::Result<()> {
    mkdir(GIT_DIR)?;

    let object = format!("{GIT_DIR}/objects");
    mkdir(object)
}

pub fn set_head(oid: &str) {
    let path = format!("{GIT_DIR}/HEAD");
    mkfile(path, oid.as_bytes()).unwrap();
}

pub fn get_head() -> Option<String> {
    let path = format!("{GIT_DIR}/HEAD");
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    Some(contents)
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

    let mut cont = contents
        .split(b'\x00' as char)
        .take(2)
        .collect::<Vec<&str>>();
    let content = cont.pop().unwrap();
    let type_obj = cont.pop().unwrap();

    if let Some(expected_type) = expected {
        assert_eq!(expected_type, type_obj);
    }

    content.to_string()
}
