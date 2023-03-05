use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const GIT_DIR: &str = ".rgit";

/// Create a new directory.
fn mkdir<P: AsRef<Path>>(path: P) -> io::Result<Option<PathBuf>> {
    let path = path.as_ref();
    if path == Path::new("") {
        return Ok(None);
    }

    match fs::create_dir(path) {
        Ok(()) => {
            let mut rgit_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            rgit_path.push(GIT_DIR);
            Ok(Some(rgit_path))
        },
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(None),
        Err(e) => Err(e)
    }
}

pub fn init() -> io::Result<Option<PathBuf>> {
    mkdir(GIT_DIR)
}
