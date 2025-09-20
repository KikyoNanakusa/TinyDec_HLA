mod util;
mod core;
mod disassemble;
mod section_data;
mod symbol;
mod instruction;


fn main() {
    env_logger::init();
    util::clear_dot_files();

    let args = util::parse_arguments();
    let path = std::path::Path::new(&args[1]);
    match core::decompile(path) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
