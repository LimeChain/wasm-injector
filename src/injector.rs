use crate::maybe_compressed_blob::{compress, decompress};
use parity_wasm::elements;
use std::collections::VecDeque;
use std::fs::{self, read};
use std::io::Write;
use wasm_instrument::parity_wasm::{
    deserialize_buffer,
    elements::{Instruction, Instructions, Module},
    serialize,
};

pub const BLOB_SIZE_LIMIT: usize = 100_000_000;

fn save(
    path: &str,
    file_name: &str,
    bytes: &[u8],
    name_modifiers: Vec<&str>,
    ext_modifiers: Vec<&str>,
) {
    match fs::OpenOptions::new()
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
        )) {
        Ok(mut file) => {
            file.write_all(bytes).expect("Couldn't write to file");
            println!(
                "Wrote {} wasm to file",
                name_modifiers
                    .iter()
                    .map(|s| str::to_uppercase(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        Err(e) => println!("Error: {}", e),
    };
}

pub fn sed_validate_block(path: &str, file_name: &str) -> Option<()> {
    let full_path = &format!("{}/{}", path, file_name);
    let orig_bytes = &read(full_path).unwrap();
    let decompressed_bytes = decompress(orig_bytes, BLOB_SIZE_LIMIT).expect("Couldn't decompress");

    // Extract the modules from the WASM bytes
    let mut module: Module = deserialize_buffer(decompressed_bytes.as_ref()).unwrap();
    println!("Original module len: {}", orig_bytes.len());

    // Find the `validate_block` function index
    let validate_block_index = module
        .export_section()?
        .entries()
        .iter()
        .find_map(|export| match export.internal() {
            elements::Internal::Function(index) if export.field() == "validate_block" => {
                Some(index)
            }
            _ => None,
        })?
        .to_owned();

    // Extract the `validate_block` instructions
    let verify_block_body = module
        .code_section_mut()?
        .bodies_mut()
        .get_mut(validate_block_index as usize)?
        .code_mut()
        .elements_mut();

    // Prepare new instructions
    // TODO: parametrize on the instructions
    let return_value: i64 = 123456789;
    let new_instructions = Instructions::new(vec![
        // Last value on the stack gets returned
        Instruction::I64Const(return_value),
        Instruction::End,
    ]);

    // Substisture the new instructions
    verify_block_body.clear();
    verify_block_body.extend(new_instructions.elements().to_vec());

    let injected_bytes = serialize(module).unwrap();
    println!("Injected module len: {}", injected_bytes.len());

    save(path, file_name, &injected_bytes, vec!["injected"], vec![]);

    let compressed_bytes = compress(&injected_bytes, BLOB_SIZE_LIMIT).unwrap();
    println!("Compressed injected module len: {}", compressed_bytes.len());

    save(
        path,
        file_name,
        &compressed_bytes,
        vec!["compressed", "injected"],
        vec![],
    );

    let mut hexified_bytes = compressed_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
        .bytes()
        .collect::<VecDeque<_>>();
    hexified_bytes.push_front(b'x');
    hexified_bytes.push_front(b'0');

    let hexified_bytes = hexified_bytes.into_iter().collect::<Vec<_>>();
    println!(
        "Hexified compressed injected module len: {}",
        hexified_bytes.len()
    );

    save(
        path,
        file_name,
        &hexified_bytes,
        vec!["hexified", "compressed", "injected"],
        vec!["hex"],
    );

    Some(())
}
