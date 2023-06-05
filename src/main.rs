use std::{collections::VecDeque, env};
use wasm_injector::injecting::injections;
use wasm_injector::util::{load_module_from_wasm, save_module_to_wasm};

fn main() {
    // Get arguments
    let mut args = env::args().collect::<VecDeque<_>>();

    // Pop $0
    args.pop_front();

    // Path is the first argument
    let path = &args.pop_front().expect("Path");

    // Filename is the second argument
    let file_name = &args.pop_front().expect("Filename");

    // Calculate full path
    let full_path = &format!("{}/{}", path, file_name);

    // Get the module
    let mut module = load_module_from_wasm(full_path).unwrap();

    // "Inject" the module
    injections::inject_stack_overflow(&mut module).unwrap();

    // Save the modified module
    save_module_to_wasm(module, path, file_name).unwrap();
}
