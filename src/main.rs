use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;
use wasm_injector::injecting::injections::Injection;
use wasm_injector::util::{load_module_from_wasm, modify_file_name, save_module_to_wasm};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[group()]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    Inject {
        #[arg(required = true, value_name = "injection", value_hint = ValueHint::Other)]
        injection: Injection,

        #[arg(
            long,
            value_name = "compressed",
            help = "Compresses the wasm. Can be combined with `hexified`",
            default_value_t = false
        )]
        compressed: bool,

        #[arg(
            long,
            value_name = "hexified",
            help = "Hexifies the wasm. Can be combined with `compressed`",
            default_value_t = false
        )]
        hexified: bool,
        #[command(flatten)]
        global_opts: GlobalOpts,
    },
    Decode {
        #[command(flatten)]
        global_opts: GlobalOpts,
    },
}

#[derive(Parser, Debug, Clone)]
struct GlobalOpts {
    #[arg(required = true, help = "Wasm source file path. Can be compressed and/or hexified.", value_hint = ValueHint::FilePath)]
    source: PathBuf,

    #[arg(global = true, help = "Destination file path (optional)", value_hint = ValueHint::FilePath)]
    destination: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let Args { action } = Args::parse();

    let calculate_default_destination_file_name = |file_name: &str| {
        let mut file_name = String::from(file_name);

        match &action {
            Action::Inject {
                compressed,
                hexified,
                ..
            } => {
                file_name = format!("injected_{}", file_name);
                if *compressed {
                    file_name = format!("compressed_{}", file_name);
                }
                if *hexified {
                    file_name = format!("hexified_{}.hex", file_name);
                }
            }
            Action::Decode { .. } => {
                file_name = format!("decoded_{}.wasm", file_name);
            }
        }

        file_name
    };

    let global_opts = match &action {
        Action::Inject { global_opts, .. } | Action::Decode { global_opts } => global_opts.clone(),
    };

    let destination = match global_opts.destination {
        // Creates a new file with the destination as name
        Some(destination_file) => destination_file,

        // There is no destination:
        // put it next to the source, surrounding it with the appropriate modifiers
        None => modify_file_name(
            global_opts.source.as_path(),
            calculate_default_destination_file_name,
        )?,
    };

    // // Get the module
    let mut module = load_module_from_wasm(global_opts.source.as_path())?;

    match action {
        Action::Inject {
            injection,
            compressed,
            hexified,
            global_opts: _,
        } => {
            // "Inject" the module
            injection.inject(&mut module)?;

            // Save the modified module
            save_module_to_wasm(module, destination.as_path(), compressed, hexified)?;
        }
        Action::Decode { .. } => {
            // Save the modified module
            save_module_to_wasm(module, destination.as_path(), false, false)?;
        }
    }

    Ok(())
}
