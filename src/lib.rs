//! # WASM Injector
//!
//! This crate provides a library for injecting WebAssembly modules with
//! predefined injections. The injections are primarily meant to be used for testing polkadot parachains hosts.
//! # Example
//! ```
//! use wasm_injector::{load_module_from_wasm, save_module_to_wasm, Injection};
//!
//! let mut module = load_module_from_wasm(source.as_path())?; // supply your own path here
//! let injection = Injection::Noops; // choose your injection
//! injection.inject(&mut module)?; // inject the module
//! save_module_to_wasm(module, destination.as_path(), compressed, hexified)?; // save the module in your destination. You can choose to compress and/or hexify the module.
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
