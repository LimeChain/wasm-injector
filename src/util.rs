use std::collections::VecDeque;
use std::fs::{read, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use sp_maybe_compressed_blob::{compress, decompress, CODE_BLOB_BOMB_LIMIT};
use wasm_instrument::parity_wasm::serialize;
use wasm_instrument::parity_wasm::{deserialize_buffer, elements::Module};

pub fn save(path: &Path, bytes: &[u8]) -> Result<(), String> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .map(|mut file| {
            file.write_all(bytes).expect("Could not write to file");
            println!("Wrote to {}", path.display());
        })
        .map_err(|err| format!("Could not open file: {}", err))
}

pub fn modify_file_name(path: &Path, mapper: impl Fn(&str) -> String) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .ok_or(format!("{} is not a file", path.display()))?
        .to_str()
        .ok_or("Couldn't convert filename to string".to_string())?;

    let mut result = PathBuf::from(path);

    result.set_file_name(mapper(file_name));

    Ok(result)
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

pub fn load_module_from_wasm(path: &Path) -> Result<Module, String> {
    // Read bytes
    let orig_bytes = &read(path).map_err(|err| format!("Could not read wasm blob: {}", err))?;

    // Deserialize module
    let module: Module = module_from_blob(orig_bytes).unwrap();

    // Return module
    Ok(module)
}

pub fn save_module_to_wasm(module: Module, destination: &Path, compressed: bool, hexified: bool) -> Result<(), String> {
    // Serialize injected module
    let mut bytes = blob_from_module(module)?;

    // Compress serialized bytes
    if compressed {
        bytes = compress(&bytes, CODE_BLOB_BOMB_LIMIT).ok_or("Bomb bomb")?;
    }

    // Hexify compressed bytes
    if hexified {
        bytes = hexify_bytes(bytes);
    }

    // Save proccessed bytes
    save(
        destination,
        // Injection, compression and hexification
        &bytes,
    )?;

    Ok(())
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
