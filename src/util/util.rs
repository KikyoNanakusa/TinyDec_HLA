use std::fs;

/// Delete all `*.dot` and `*.png` files in the current directory, if present.
/// Best-effort: ignores errors and continues.
pub fn clear_dot_files() {
    if let Ok(dir) = fs::read_dir("images") {
        for entry in dir.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "dot" || ext == "png" {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }
}

/// Parse CLI arguments and return them as a `Vec<String>`.
/// Exits with a usage message if the number of args is not exactly 2 (program + file path).
pub fn parse_arguments() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
     eprintln!("Usage: {} <file>", args[0]);
     std::process::exit(1);
    }
    args
}

// Split the operands into a vector of strings.
// If the operands are not provided, return an empty vector.
// If the operands are provided, split them by commas and trim the whitespace.
// If the operands are empty, return an empty vector.
#[allow(dead_code)]
pub fn split_operands(ops: &Option<String>) -> Vec<String> {
    ops.as_ref()
        .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
        .unwrap_or_else(|| vec![])
}
