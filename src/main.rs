use clap::{builder::ArgPredicate, Parser, Subcommand, ValueHint};
use std::path::PathBuf;
use wasm_injector::injecting::injections::Injection;
use wasm_injector::util::{load_module_from_wasm, modify_file_name, save_module_to_wasm};

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum Action {
    #[command(about = "Inject invalid instructions into a wasm module")]
    Inject {
        #[arg(required = true, value_name = "injection", value_hint = ValueHint::Other)]
        injection: Injection,

        #[command(flatten)]
        global_opts: GlobalOpts,

        #[arg(
            long,
            value_name = "compressed",
            help = "Compresses the wasm. Can be used with `--hexified`",
            default_value_t = false
        )]
        compressed: bool,

        #[arg(
            long,
            value_name = "hexified",
            help = "Hexifies the wasm. Can be used with `--compressed`",
            default_value_t = false
        )]
        hexified: bool,
    },
    #[command(
        about = "Convert from `hexified` and/or `compressed` to `raw` wasm module and vice versa"
    )]
    Convert {
        #[command(flatten)]
        global_opts: GlobalOpts,

        #[arg(
            long,
            value_name = "raw",
            help = "Saves the file as raw wasm (default). Can not be used with `--compressed` or `--hexified`.",
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
            help = "Compresses the wasm (zstd compression). Can be used with `--hexified`.",
            default_value_t = false
        )]
        compressed: bool,

        #[arg(
            long,
            value_name = "hexified",
            help = "Hexifies the wasm. Can be used with `--compressed`",
            default_value_t = false
        )]
        hexified: bool,
    },
}

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
struct GlobalOpts {
    #[arg(required = true, help = "Wasm source file path. Can be compressed and/or hexified.", value_hint = ValueHint::FilePath)]
    source: PathBuf,

    #[arg(help = "Destination file path (optional). If not specified, the output file will be a prefixed source file name. ", value_hint = ValueHint::FilePath)]
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

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_inject_noops() {
        assert_eq!(
            Cli::try_parse_from(&["test", "inject", "noops", "test.wasm"]).unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::Noops,
                    global_opts: GlobalOpts {
                        source: PathBuf::from("test.wasm"),
                        destination: None
                    },
                    compressed: false,
                    hexified: false
                }
            }
        )
    }

    #[test]
    fn test_inject_heap_overflow() {
        assert_eq!(
            Cli::try_parse_from(&["test", "inject", "heap-overflow", "test.wasm"]).unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::HeapOverflow,
                    global_opts: GlobalOpts {
                        source: PathBuf::from("test.wasm"),
                        destination: None
                    },
                    compressed: false,
                    hexified: false
                }
            }
        )
    }

    #[test]
    fn test_inject_stack_overflow() {
        assert_eq!(
            Cli::try_parse_from(&["test", "inject", "stack-overflow", "test.wasm"]).unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::StackOverflow,
                    global_opts: GlobalOpts {
                        source: PathBuf::from("test.wasm"),
                        destination: None
                    },
                    compressed: false,
                    hexified: false
                }
            }
        )
    }

    #[test]
    fn test_inject_bad_return_value() {
        assert_eq!(
            Cli::try_parse_from(&["test", "inject", "bad-return-value", "test.wasm"]).unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::BadReturnValue,
                    global_opts: GlobalOpts {
                        source: PathBuf::from("test.wasm"),
                        destination: None
                    },
                    compressed: false,
                    hexified: false
                }
            }
        )
    }

    #[test]
    fn test_inject_infinite_loop() {
        assert_eq!(
            Cli::try_parse_from(&["test", "inject", "infinite-loop", "test.wasm"]).unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::InfiniteLoop,
                    global_opts: GlobalOpts {
                        source: PathBuf::from("test.wasm"),
                        destination: None
                    },
                    compressed: false,
                    hexified: false
                }
            }
        )
    }

    #[test]
    fn test_inject_invalid_injection() {
        assert!(Cli::try_parse_from(&["test", "inject", "invalid-injection", "test.wasm"]).is_err())
    }

    #[test]
    fn test_convert() {
        assert_eq!(
            Cli::try_parse_from(&["test", "convert", "test.wasm"]).unwrap(),
            Cli {
                action: Action::Convert {
                    global_opts: GlobalOpts {
                        source: PathBuf::from("test.wasm"),
                        destination: None
                    },
                    raw: true,
                    compressed: false,
                    hexified: false
                }
            }
        )
    }

    #[test]
    fn test_convert_raw_exludes_compressed() {
        assert!(
            Cli::try_parse_from(&["test", "convert", "test.wasm", "--compressed", "--raw"])
                .is_err()
        )
    }

    #[test]
    fn test_convert_raw_exludes_hexified() {
        assert!(
            Cli::try_parse_from(&["test", "convert", "test.wasm", "--hexified", "--raw"]).is_err()
        )
    }
}
