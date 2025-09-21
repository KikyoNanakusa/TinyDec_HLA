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
