use std::{fs, path::PathBuf};

use anyhow::Ok;

fn get_token_file_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push(".data");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("token");
    path
}

pub fn persist_token(token: &str) -> anyhow::Result<()> {
    let _ = fs::write(get_token_file_path(), token)?;
    Ok(())
}

pub fn read_token() -> Option<String> {
    fs::read_to_string(get_token_file_path()).ok()
}

pub fn delete_token() -> anyhow::Result<()> {
    let _ = fs::remove_file(get_token_file_path());
    Ok(())
}
