use anyhow::Result;
use std::path::Path;

pub fn decompile(_path: &Path) -> Result<String> {
    Ok(String::from("Hello, Decompiler!"))
}
