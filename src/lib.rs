//! # WASM Injector
//!
//! This crate provides a library for injecting WebAssembly modules with
//! predefined injections. The injections are primarily meant to be used for testing polkadot parachains hosts.
//! # Example
//! ```
//! use std::path::Path;
//! use wasm_injector::{Injection, load_module_from_wasm, save_module_to_wasm};
//!
//! # fn main() -> Result<(), String> {
//! let source = Path::new("samples/example.wasm");
//! let destination = Path::new("samples/injected_example.wasm");
//! let compressed = false;
//! let hexified = true;
//!
//! let mut module = load_module_from_wasm(source)?; // supply your own path here
//! let injection = Injection::StackOverflow; // choose your injection
//! injection.inject(&mut module, "validate_block", None)?; // inject the instruction into the specified wasm export function
//!
//! save_module_to_wasm(module, destination, compressed, hexified)?; // save the module in your destination. You can choose to compress and/or hexify the module.
//!     
//! # std::fs::remove_file(destination);
//! # Ok(())
//! # }
//! ```

pub mod injecting;
pub mod stack_limiter;
pub mod util;

pub use self::injecting::injections::Injection;
pub use self::util::blob_from_module;
pub use self::util::hexify_bytes;
pub use self::util::load_module_from_wasm;
pub use self::util::module_from_blob;
pub use self::util::save_module_to_wasm;
pub use self::util::unhexify_bytes;
