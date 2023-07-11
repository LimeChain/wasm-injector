use std::collections::VecDeque;
use std::fs::{read, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use itertools::Itertools;
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

pub fn get_file_name(path: &Path) -> Result<&str, String> {
    path.file_name()
        .ok_or(format!("{} is not a file", path.display()))?
        .to_str()
        .ok_or("Couldn't convert filename to string".to_string())
}

pub fn modify_file_name(path: &Path, mapper: impl Fn(&str) -> String) -> Result<PathBuf, String> {
    let file_name = get_file_name(path)?;

    let mut result = PathBuf::from(path);

    result.set_file_name(mapper(file_name));

    Ok(result)
}

// Extract the module from the (maybe hexified) (maybe compressed) WASM bytes
pub fn module_from_blob(blob_bytes: &[u8]) -> Result<Module, String> {
    let mut blob_bytes = blob_bytes.to_vec();

    // Unhexify if needed
    if blob_bytes[0..=1] == *b"0x" {
        blob_bytes = unhexify_bytes(blob_bytes)?;
    }

    // Decompress if needed
    let blob_bytes = decompress(&blob_bytes, CODE_BLOB_BOMB_LIMIT)
        .map_err(|err| format!("Couldn't decompress blob: {}", err))?;

    // Deserialize the modules from the raw bytes
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

pub fn save_module_to_wasm(
    module: Module,
    destination: &Path,
    compressed: bool,
    hexified: bool,
) -> Result<(), String> {
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

pub fn unhexify_bytes(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    bytes
        .iter()
        // Each pair of hex bytes represent a single real byte
        .chunks(2)
        .into_iter()
        // Skip the "0x"
        .skip(1)
        .map(|two_bytes| {
            // Parse each pair into a string ...
            String::from_utf8(two_bytes.cloned().collect_vec())
                .map_err(|err| format!("Could not convert bytes to string: {}", err))
                .and_then(|two_bytes_string| {
                    // ... and_then try to parse that string into a number (byte)
                    u8::from_str_radix(&two_bytes_string, 16)
                        .map_err(|err| format!("Could not parse string: {}", err))
                })
        })
        // sequence :: Vec<Result<u8, String>>
        //          -> Result<Vec<u8>, String>
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{hexify_bytes, unhexify_bytes};

    #[test]
    fn hexification_isomorphism() {
        let bytes = b"0123456789abcdef".to_vec();

        let hexified_bytes = hexify_bytes(bytes.clone());

        let unhexified_bytes = unhexify_bytes(hexified_bytes).unwrap();

        assert_eq!(bytes, unhexified_bytes);
    }
}
