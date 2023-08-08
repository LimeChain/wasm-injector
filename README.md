# wasm_injector

`wasm_injector` is a Rust-based command line utility to manipulate WebAssembly (wasm) modules. It allows you to inject code, compress, and hexify a given wasm module. This utility is especially useful when you need to modify wasm files for testing, optimization, or debugging.

## Prerequisites

To use this utility, you need to have Rust installed on your system. If you don't have Rust installed, you can follow the official instructions [here](https://www.rust-lang.org/tools/install).

## Installation

Clone the repository and build the project with `cargo`, Rust's package manager:

```sh
git clone https://github.com/LimeChain/parachain-conformance-dev.git
cd parachain-conformance-dev
cargo build --release
```

This will create an executable in the `target/release` directory.

## Usage
- This project is developed to inject instructions in WASM modules or polkadot conformance testing. You can download a sample WASM file from [here](https://github.com/paritytech/cumulus/releases/tag/parachains-v9430).
- The script will automatically unhexify and decompress WASM modules passed in as input
- The general format to run the `wasm_injector` is as follows:

```sh
Usage: wasm_injector <COMMAND>

Commands:
  inject   Inject invalid instructions into a wasm module
  convert  Convert from `hexified` and/or `compressed` to `raw` wasm module and vice versa
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Inject:
```sh
Inject invalid instructions into a wasm module

Usage: wasm_injector inject [OPTIONS] <injection> <function> <source> [destination]

Arguments:
  <injection>    [possible values: infinite-loop, bad-return-value, stack-overflow, noops, heap-overflow]
  <function>     The name of the exported function to be injected
  <source>       Wasm source file path. Can be compressed and/or hexified.
  [destination]  Destination file path (optional). If not specified, the output file will be a prefixed source file name. 

Options:
      --compressed  Compresses the wasm. Can be used with `--hexified`
      --hexified    Hexifies the wasm. Can be used with `--compressed`
  -h, --help        Print help
```

### Convert:
```sh
Convert from `hexified` and/or `compressed` to `raw` wasm module and vice versa

Usage: wasm_injector convert [OPTIONS] <source> [destination]

Arguments:
  <source>       Wasm source file path. Can be compressed and/or hexified.
  [destination]  Destination file path (optional). If not specified, the output file will be a prefixed source file name. 

Options:
      --raw         Saves the file as raw wasm (default). Can not be used with `--compressed` or `--hexified`.
      --compressed  Compresses the wasm (zstd compression). Can be used with `--hexified`.
      --hexified    Hexifies the wasm. Can be used with `--compressed`
  -h, --help        Print help
```

## Examples

### Inject:
To inject code into a wasm file, compress and hexify it, you can run:

```sh
./wasm_injector inject noops validate_block my_wasm_file.wasm --compressed --hexified
```

To specify a custom destination path, you can run:

```sh
./wasm_injector inject noops validate_block my_wasm_file.wasm my_destination_directory/injected_new_file.wasm
```

### Convert:

#### From Compressed and/or Hexified Wasm to Raw (default):
```sh
./wasm_injector convert --raw compressed_and_hexified_wasm_file.wasm.hex raw_wasm_file.wasm
```

or

```sh
./wasm_injector convert compressed_and_hexified_wasm_file.wasm.hex raw_wasm_file.wasm
```

#### From Raw Wasm to Compressed and/or Hexified:

```sh
./wasm_injector convert --hexified raw_wasm_file.wasm heified_wasm_file.wasm.hex
```

```sh
./wasm_injector convert  --compressed --hexified raw_wasm_file.wasm compressed_and_hexified_wasm_file.wasm.hex
```

To run the WASM you need Linux environment with [Zombienet](https://github.com/paritytech/zombienet)
For each test you need to specify the path to the WASM module in the corresponding `.toml` file before running the test

```sh
zombienet -p native test ./tests/0001-parachains-pvf-compilation-time-bad.zndsl
```

## Contributing

Please feel free to contribute to the project. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under [TODO]. See the LICENSE file for details.
