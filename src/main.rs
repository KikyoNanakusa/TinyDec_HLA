mod util;
mod core;
mod disassemble;
mod symbol;
mod instruction;
mod block;
mod block_store;
mod edge_label;
mod condition;
mod cfg_construction;
mod visualize;
mod region; 
mod region_arena;
mod cfg_structuring;
mod raw_loop;
mod codegen;

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
