use clap::{Parser, ValueHint};
use std::path::PathBuf;
use wasm_injector::injecting::injections::Injection;
use wasm_injector::util::{
    get_file_name, load_module_from_wasm, modify_file_name, save_module_to_wasm,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1, required = true, value_name = "injection")]
    injection: Injection,

    #[arg(index = 2, required = true, value_name = "(hexified) (compressed) wasm source file", value_hint = ValueHint::FilePath)]
    source: PathBuf,

    #[arg(index = 3, value_name = "destination path (directory/file)", value_hint = ValueHint::FilePath)]
    destination: Option<PathBuf>,

    #[arg(long, value_name = "compressed", default_value_t = false)]
    compressed: bool,

    #[arg(long, value_name = "hexified", default_value_t = false)]
    hexified: bool,
}

fn main() -> Result<(), String> {
    let Args {
        injection,
        source,
        destination,
        compressed,
        hexified,
    } = Args::parse();

    let calculate_default_destination_file_name = |file_name: &str| {
        let mut file_name = String::from(file_name);

        if injection != Injection::Nothing {
            file_name = format!("injected_{}", file_name);
        }
        if compressed {
            file_name = format!("compressed_{}", file_name);
        }
        if hexified {
            file_name = format!("hexified_{}.hex", file_name);
        }

        file_name
    };

    let destination = match destination {
        // Destination is a directory:
        // use the source filename and surround it with the appropriate modifiers
        Some(destination_directory) if destination_directory.is_dir() => destination_directory
            .join(calculate_default_destination_file_name(get_file_name(
                source.as_path(),
            )?)),

        // Destination is a file:
        // use directly
        Some(destination_file) if destination_file.is_file() => destination_file,

        // Destination is something else:
        // error out
        Some(_) => panic!("Destination should either be a directory or a file"),

        // There is no destination:
        // put it next to the source, surrounding it with the appropriate modifiers
        None => modify_file_name(source.as_path(), calculate_default_destination_file_name)?,
    };

    // Get the module
    let mut module = load_module_from_wasm(source.as_path())?;

    // "Inject" the module
    injection.inject(&mut module)?;

    // Save the modified module
    save_module_to_wasm(module, destination.as_path(), compressed, hexified)?;

    Ok(())
}
