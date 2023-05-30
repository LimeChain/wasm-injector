#![allow(unused)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::mem;
use parity_wasm::{
    builder,
    elements::{self, Instruction, Instructions, Type},
};

use crate::maybe_compressed_blob::{compress, decompress};
use std::fs::{self, read};
use std::io::Write;
pub use wasm_instrument;
use wasm_instrument::parity_wasm::{deserialize_buffer, elements::Module, serialize};

/// Generate a new global that will be used for tracking a timer
fn generate_timer_global(module: &mut elements::Module) -> u32 {
    let global_entry = builder::global()
        .value_type()
        .i32()
        .mutable()
        .init_expr(Instruction::I32Const(0))
        .build();

    // Try to find an existing global section.
    for section in module.sections_mut() {
        if let elements::Section::Global(gs) = section {
            gs.entries_mut().push(global_entry);
            return (gs.entries().len() as u32) - 1;
        }
    }

    // Existing section not found, create one!
    module.sections_mut().push(elements::Section::Global(
        elements::GlobalSection::with_entries(vec![global_entry]),
    ));
    0
}

pub fn inject_single(path: &str, file_name: &str) {
    let full_path = &format!("{}/{}", path, file_name);
    let orig_bytes = &read(full_path).unwrap();
    let decompressed_bytes = decompress(orig_bytes, 10_000_000).expect("Couldn't decompress");

    let orig_module: Module = deserialize_buffer(decompressed_bytes.as_ref()).unwrap();
    println!("Original module len: {}", orig_bytes.len());

    let injected_module =
        wasm_instrument::inject_stack_limiter(orig_module, 1024).expect("Couldn't inject limiter");

    let injected_bytes = serialize(injected_module).unwrap();
    match fs::OpenOptions::new()
        .create(true) // To create a new file
        .write(true)
        .open(format!("{}/injected_{}", path, file_name))
    {
        Ok(mut file) => {
            file.write_all(&injected_bytes)
                .expect("Couldn't write to file");
            println!("Wrote stack-injected wasm to file")
        }
        Err(e) => println!("Error: {}", e),
    }

    let compressed_bytes = compress(&injected_bytes, 100_000_000).unwrap();

    println!("Injected module len: {}", injected_bytes.len());
    println!("Compressed module len: {}", compressed_bytes.len());

    match fs::OpenOptions::new()
        .create(true) // To create a new file
        .write(true)
        .open(format!("{}/injected_compressed_{}", path, file_name))
    {
        Ok(mut file) => {
            file.write_all(&compressed_bytes)
                .expect("Couldn't write to file");
            println!("Wrote stack-injected wasm to file")
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn decompress_takovata(path: &str, file_name: &str) {
    let full_path = &format!("{}/{}", path, file_name);
    let orig_bytes = &read(full_path).unwrap();
    let decompressed_bytes = decompress(orig_bytes, 10_000_000).expect("Couldn't decompress");
    let _ = std::fs::write(format!("{}/decompressed_{}", path, file_name), decompressed_bytes);
}

fn show_information(path: &str, file_name: &str) {
    let full_path = &format!("{}/{}", path, file_name);
    let orig_bytes = &read(full_path).unwrap();
    let decompressed_bytes = decompress(orig_bytes, 10_000_000).expect("Couldn't decompress");

    let module: Module = deserialize_buffer(&decompressed_bytes).unwrap();
    println!("Original module len: {}", orig_bytes.len());
    if module.code_section().is_none() {
        println!("no code in module!");
        std::process::exit(1);
    }

    module
        .parse_names()
        .expect("Couldn't parse names")
        .export_section()
        .expect("Empty names section")
        .entries()
        .iter()
        .for_each(|entry| {
            println!("entry: {:?}", entry);
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn inject_stack_limiter() {
        use super::*;

        let path = "wasm";
        let file_name = "rococo-parachain_runtime-v9400.compact.compressed.wasm";
        inject_single(path, file_name);
    }

    #[test]
    fn show_wasm_information() {
        use super::*;

        let path = "wasm";
        let file_name = "rococo-parachain_runtime-v9381.compact.compressed.wasm";
        show_information(path, file_name);
    }

    #[test]
    fn takovata() {
        use super::*;

        let path = "wasm";
        let file_name = "rococo-parachain_runtime-v9381.compact.compressed.wasm";
        decompress_takovata(path, file_name);
    }
}

