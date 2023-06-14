use std::{collections::VecDeque, env};
use wasm_injector::injecting::injections;
use wasm_injector::util::{load_module_from_wasm, save_module_to_wasm};

fn main() -> Result<(), String> {
    // Get arguments
    let mut args = env::args().collect::<VecDeque<_>>();

    // Pop $0
    args.pop_front();

    // Path is the first argument
    let path = &args.pop_front().ok_or("Path")?;

    // Filename is the second argument
    let file_name = &args.pop_front().ok_or("Filename")?;

    // Calculate full path
    let full_path = &format!("{}/{}", path, file_name);

    // Get the module
    let mut module = load_module_from_wasm(full_path)?;

    // "Inject" the module
    injections::inject_infinite_loop(&mut module)?;

    // Save the modified module
    save_module_to_wasm(module, path, file_name)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use sp_maybe_compressed_blob::{decompress, CODE_BLOB_BOMB_LIMIT};
    use wasm_instrument::parity_wasm::{deserialize_buffer, elements::Module};

    #[test]
    fn takovata() {
        let path = "../ZigWasm/wasm";
        let file_name = "injected_rococo-parachain_runtime-v9400.compact.compressed.wasm";

        let full_path = &format!("{}/{}", path, file_name);
        let orig_bytes = &read(full_path).unwrap();
        let decompressed_bytes = decompress(orig_bytes, CODE_BLOB_BOMB_LIMIT).expect("Couldn't decompress");

        let module: Module = deserialize_buffer(&decompressed_bytes).unwrap();
        println!("Original module len: {}", orig_bytes.len());
        if module.code_section().is_none() {
            println!("No code in module!");
            std::process::exit(1);
        }
    }
}
