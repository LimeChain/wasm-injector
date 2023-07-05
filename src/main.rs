use clap::{Parser, ValueHint};
use std::path::PathBuf;
use wasm_injector::injecting::injections::Injection;
use wasm_injector::util::{load_module_from_wasm, modify_file_name, save_module_to_wasm};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1, required = true, value_name = "injection")]
    injection: Injection,

    #[arg(index = 2, required = true, value_name = "wasm source file", value_hint = ValueHint::FilePath)]
    source: PathBuf,

    #[arg(index = 3, value_name = "destination file", value_hint = ValueHint::FilePath)]
    destination: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let Args {
        injection,
        source,
        destination,
    } = Args::parse();

    let default_destination = modify_file_name(source.as_path(), |file_name| {
        format!("hexified_{}.hex", file_name)
    })?;

    let destination = destination.unwrap_or(default_destination);

    // Get the module
    let mut module = load_module_from_wasm(source.as_path())?;

    // "Inject" the module
    injection.inject(&mut module)?;

    // Save the modified module
    save_module_to_wasm(module, destination.as_path(), None)?;

    Ok(())
}
