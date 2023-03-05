use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub const GIT_DIR: &str = ".rgit";

/// Create a new directory.
fn mkdir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();

    fs::create_dir(path)?;
    Ok(())
}

/// Create a new file.
fn mkfile<P: AsRef<Path>>(path: P, data: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;

    Ok(())
}

pub fn init() -> io::Result<()> {
    mkdir(GIT_DIR)?;

    let object = format!("{GIT_DIR}/objects");
    mkdir(object)
}

pub fn hash_object(data: &str) -> String {
    let oid = sha1_smol::Sha1::from(data).digest().to_string();
    let path = format!("{GIT_DIR}/objects/{oid}");
    mkfile(path, data).expect("create fail");

    oid
}
