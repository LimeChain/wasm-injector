use clap::{
    builder::ArgPredicate, error::ErrorKind, CommandFactory, Parser, Subcommand, ValueHint,
};
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
        #[arg(value_enum, required = true, requires_if("noops", "size"), value_name = "injection",value_hint = ValueHint::Other)]
        injection: Injection,

        #[arg(required = true, value_name = "function", help = "The name of the exported function to be injected with the instructions", value_hint = ValueHint::Other)]
        function: String,

        #[arg(
            long,
            value_name = "size", 
            help = "The number of noops to be injected in MB (1 NOP = 1 byte)", 
            value_hint = ValueHint::Other
        )]
        size: Option<i16>,

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
            help = "Saves the file as raw wasm (default). Can not be used with `--compressed` or `--hexified`",
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
            help = "Compresses the wasm (zstd compression). Can be used with `--hexified`",
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
    #[arg(required = true, value_name = "source", help = "Wasm source file path. Can be compressed and/or hexified", value_hint = ValueHint::FilePath)]
    source: PathBuf,

    #[arg(value_name = "destination", help = "Destination file path (optional). If not specified, the output file will be a prefixed source file name", value_hint = ValueHint::FilePath)]
    destination: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let Cli { action } = Cli::parse();

    if let Action::Inject {
        injection, size, ..
    } = &action
    {
        match injection {
            Injection::Noops => {}
            _ => {
                if size.is_some() {
                    let mut cmd = Cli::command();
                    cmd.error(
                        ErrorKind::ArgumentConflict,
                        "The `size` argument is only valid for the `noops` injection".to_string(),
                    )
                    .exit();
                }
            }
        }
    }

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

    if let Action::Inject {
        injection,
        function,
        size,
        ..
    } = action
    {
        // Inject the module
        injection.inject(&mut module, &function, size)?;
    }

    save_module_to_wasm(module, destination.as_path(), compressed, hexified)?;

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    const FUNCTION_NAME: &str = "validate_block";

    #[test]
    fn test_invalid_subcommand() {
        let result = Cli::try_parse_from(&["test", "invalid"]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::InvalidSubcommand
        )
    }

    #[test]
    fn function_name_is_required() {
        assert!(Cli::try_parse_from(&["test", "inject", "noops", "test.wasm"]).is_err())
    }

    #[test]
    fn test_inject_noops() {
        assert_eq!(
            Cli::try_parse_from(&[
                "test",
                "inject",
                "noops",
                "--size",
                "20",
                FUNCTION_NAME,
                "test.wasm"
            ])
            .unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::Noops,
                    size: Some(20),
                    function: FUNCTION_NAME.to_string(),
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
    fn test_inject_noops_requires_size_arg() {
        let result = Cli::try_parse_from(&["test", "inject", "noops", FUNCTION_NAME, "test.wasm"]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        )
    }

    #[test]
    fn test_inject_heap_overflow() {
        assert_eq!(
            Cli::try_parse_from(&[
                "test",
                "inject",
                "heap-overflow",
                FUNCTION_NAME,
                "test.wasm"
            ])
            .unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::HeapOverflow,
                    function: FUNCTION_NAME.to_string(),
                    size: None,
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
            Cli::try_parse_from(&[
                "test",
                "inject",
                "stack-overflow",
                FUNCTION_NAME,
                "test.wasm"
            ])
            .unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::StackOverflow,
                    function: FUNCTION_NAME.to_string(),
                    size: None,
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
            Cli::try_parse_from(&[
                "test",
                "inject",
                "bad-return-value",
                FUNCTION_NAME,
                "test.wasm"
            ])
            .unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::BadReturnValue,
                    function: FUNCTION_NAME.to_string(),
                    size: None,
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
            Cli::try_parse_from(&[
                "test",
                "inject",
                "infinite-loop",
                FUNCTION_NAME,
                "test.wasm"
            ])
            .unwrap(),
            Cli {
                action: Action::Inject {
                    injection: Injection::InfiniteLoop,
                    function: FUNCTION_NAME.to_string(),
                    size: None,
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
        let result = Cli::try_parse_from(&[
            "test",
            "inject",
            "invalid-injection",
            &FUNCTION_NAME,
            "test.wasm",
        ]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::InvalidValue
        )
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
        let result =
            Cli::try_parse_from(&["test", "convert", "test.wasm", "--compressed", "--raw"]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::ArgumentConflict
        );
    }

    #[test]
    fn test_convert_raw_exludes_hexified() {
        let result = Cli::try_parse_from(&["test", "convert", "test.wasm", "--hexified", "--raw"]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::ArgumentConflict
        );
    }
}
