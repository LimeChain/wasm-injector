use std::{env, collections::VecDeque};
use wasm_stack_injector::injector::sed_validate_block;

fn main() {
    // Get arguments
    let mut args = env::args().collect::<VecDeque<_>>();

    // Pop $0
    args.pop_front();

    // Path is the first argument
    let path = &args.pop_front().expect("Path");
    // Filename is the second argument
    let file_name = &args.pop_front().expect("Filename");

    // "Inject" the wasm
    sed_validate_block(path, file_name);
}
