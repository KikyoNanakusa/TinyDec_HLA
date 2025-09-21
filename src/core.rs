use std::path::Path;
use anyhow::Result;

pub fn decompile(_path: &Path) -> Result<String>{
	Ok(String::from("Hello, Decompiler!"))
}