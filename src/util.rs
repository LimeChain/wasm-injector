use std::collections::VecDeque;
use std::fs::{read, OpenOptions};
use std::io::Write;

use sp_maybe_compressed_blob::{compress, decompress, CODE_BLOB_BOMB_LIMIT};
use wasm_instrument::parity_wasm::serialize;
use wasm_instrument::parity_wasm::{deserialize_buffer, elements::Module};

pub fn save(
    path: &str,
    file_name: &str,
    bytes: &[u8],
    name_modifiers: Vec<&str>,
    ext_modifiers: Vec<&str>,
) -> Result<(), String> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!(
            "{}/{}{}{}",
            path,
            name_modifiers
                .iter()
                .map(|s| format!("{}_", s.to_lowercase()))
                .collect::<String>(),
            file_name,
            ext_modifiers
                .iter()
                .map(|s| format!(".{}", s.to_lowercase()))
                .collect::<String>()
        ))
        .map(|mut file| {
            file.write_all(bytes).expect("Could not write to file");
            println!(
                "Wrote {} wasm to file",
                name_modifiers
                    .iter()
                    .map(|s| str::to_uppercase(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        })
        .map_err(|err| format!("Could not open file: {}", err))
}

// Extract the module from the (maybe compressed) WASM bytes
pub fn module_from_blob(blob_bytes: &[u8]) -> Result<Module, String> {
    let blob_bytes = decompress(blob_bytes, CODE_BLOB_BOMB_LIMIT)
        .map_err(|err| format!("Couldn't decompress blob: {}", err))?;

    deserialize_buffer(blob_bytes.as_ref())
        .map_err(|err| format!("Could not deserialize blob: {}", err))
}

pub fn blob_from_module(module: Module) -> Result<Vec<u8>, String> {
    serialize(module).map_err(|err| format!("Could not serialize module: {}", err))
}

pub fn load_module_from_wasm(path: &str) -> Result<Module, String> {
    // Read bytes
    let orig_bytes = &read(path).map_err(|err| format!("Could not read wasm blob: {}", err))?;

    // Deserialize module
    let module: Module = module_from_blob(orig_bytes).unwrap();

    // Return module
    Ok(module)
}

pub fn save_module_to_wasm(module: Module, path: &str, file_name: &str) -> Result<(), String> {
    // Serialize injected module
    let injected_bytes = blob_from_module(module)?;

    save(
        path,
        file_name,
        &injected_bytes,
        vec!["injected"],
        vec![],
    ).unwrap();

    // Compress serialized bytes
    let compressed_bytes = compress(&injected_bytes, CODE_BLOB_BOMB_LIMIT).unwrap();

    // Hexify compressed bytes
    let hexified_bytes = hexify_bytes(compressed_bytes);

    save(
        path,
        file_name,
        &hexified_bytes,
        vec!["hexified", "injected"],
        vec!["hex"],
    )
}

pub fn hexify_bytes(bytes: Vec<u8>) -> Vec<u8> {
    let mut hexified_bytes = bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
        .bytes()
        .collect::<VecDeque<_>>();
    hexified_bytes.push_front(b'x');
    hexified_bytes.push_front(b'0');

    hexified_bytes.into()
}
