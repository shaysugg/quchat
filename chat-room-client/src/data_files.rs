use std::path::{Path, PathBuf};

pub fn create_dir_if_needed() -> Result<(), std::io::Error> {
    let mut path = std::env::current_dir()?;
    path.push(".data");
    if path.exists() {
        Ok(())
    } else {
        std::fs::create_dir_all(&path)
    }
}

pub fn path() -> PathBuf {
    Path::new("./.data").to_path_buf()
}
