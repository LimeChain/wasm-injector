use sp_maybe_compressed_blob::{compress, CODE_BLOB_BOMB_LIMIT};
use std::fs::read;
use std::{collections::VecDeque, env};
use wasm_injector::injector::{blob_from_module, module_from_blob, ModuleMapper};
use wasm_injector::util::{hexify_bytes, save};
use wasm_instrument::parity_wasm::elements::{Instruction, Instructions, Module, FuncBody};

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
    let full_path = format!("{}/{}", path, file_name);

    // Read bytes
    let orig_bytes = &read(full_path).unwrap();

    // Deserialize module
    let mut module: Module = module_from_blob(orig_bytes).unwrap();

    // "Inject" the module
    module
        .map_function("validate_block", |func_body: &mut FuncBody| {
            *func_body.code_mut() = Instructions::new(vec![
                // Last value on the stack gets returned
                Instruction::I64Const(123456789),
                Instruction::End,
            ]);
        })
        .unwrap();

    // Serialize injected module
    let injected_bytes = blob_from_module(module).unwrap();

    // Compress serialized bytes
    let compressed_bytes = compress(&injected_bytes, CODE_BLOB_BOMB_LIMIT).unwrap();

    // Hexify compressed bytes
    let hexified_bytes = hexify_bytes(compressed_bytes);

    // Write final bytes
    save(
        path,
        file_name,
        &hexified_bytes,
        vec!["hexified", "compressed", "injected"],
        vec!["hex"],
    );
}
