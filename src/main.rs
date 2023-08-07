use clap::{builder::ArgPredicate, Parser, Subcommand, ValueHint};
use std::path::PathBuf;
use wasm_injector::injecting::injections::Injection;
use wasm_injector::util::{load_module_from_wasm, modify_file_name, save_module_to_wasm};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    #[command(about = "Injects a wasm module into another wasm module")]
    Inject {
        #[arg(required = true, value_name = "injection", value_hint = ValueHint::Other)]
        injection: Injection,

        #[command(flatten)]
        global_opts: GlobalOpts,

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
    },
    #[command(
        about = "Convert a hexified and/or compressed wasm module back to raw or hexify and/or compress a raw wasm module"
    )]
    Convert {
        #[command(flatten)]
        global_opts: GlobalOpts,

        #[arg(
            long,
            value_name = "raw",
            help = "Saves the file as raw wasm (default). Can not be combined with `compressed` or `hexified`.",
            default_value_t = true,
            default_value_ifs = [
                ("compressed", ArgPredicate::IsPresent, "false"),
                ("hexified", ArgPredicate::IsPresent, "false")
            ],
            conflicts_with_all = ["hexified", "compressed"]
        )]
        raw: bool,

        #[arg(
            long,
            value_name = "compressed",
            help = "Compresses the wasm (zstd compression). Can be combined with `hexified`.",
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
    },
}

#[derive(Parser, Debug, Clone)]
struct ConvertOpts {}

#[derive(Parser, Debug, Clone)]
struct GlobalOpts {
    #[arg(required = true, help = "Wasm source file path. Can be compressed and/or hexified.", value_hint = ValueHint::FilePath)]
    source: PathBuf,

    #[arg(help = "Destination file path (optional)", value_hint = ValueHint::FilePath)]
    destination: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let Cli { action } = Cli::parse();

    let calculate_default_destination_file_name = |file_name: &str| {
        let mut file_name = String::from(file_name);

        match &action {
            Action::Inject {
                injection,
                hexified,
                compressed,
                ..
            } => {
                file_name = format!("{}-{}.wasm", injection, file_name);
                if *compressed {
                    file_name = format!("compressed-{}", file_name);
                }
                if *hexified {
                    file_name = format!("hexified-{}.hex", file_name);
                }
            }
            Action::Convert {
                raw,
                compressed,
                hexified,
                ..
            } => {
                println!(
                    "raw: {}, compressed: {}, hexified: {}",
                    raw, compressed, hexified
                );
                if *raw {
                    file_name = format!("raw-{}.wasm", file_name);
                }
                if *compressed {
                    file_name = format!("compressed-{}", file_name);
                }
                if *hexified {
                    file_name = format!("hexified-{}.hex", file_name);
                }
            }
        }

        file_name
    };

    let (global_opts, hexified, compressed) = match &action {
        Action::Inject {
            global_opts,
            hexified,
            compressed,
            ..
        }
        | Action::Convert {
            global_opts,
            hexified,
            compressed,
            ..
        } => (global_opts.clone(), *hexified, *compressed),
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

    // Get the module
    let mut module = load_module_from_wasm(global_opts.source.as_path())?;

    if let Action::Inject { injection, .. } = action {
        // Inject the module
        injection.inject(&mut module)?;
    }

    save_module_to_wasm(module, destination.as_path(), compressed, hexified)?;

    Ok(())
}
