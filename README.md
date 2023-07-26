  # wasm-injector

  `wasm-injector` is a Rust-based command line utility to manipulate WebAssembly (wasm) modules. It allows you to inject code, compress, and hexify a given wasm module. This utility is especially useful when you need to modify wasm files for testing, optimization, or debugging.

  ## Prerequisites

  To use this utility, you need to have Rust installed on your system. If you don't have Rust installed, you can follow the official instructions [here](https://www.rust-lang.org/tools/install).

  ## Installation

  Clone the repository and build the project with `cargo`, Rust's package manager:

  ```sh
  git clone https://github.com/your_github_username/wasm-injector.git
  cd wasm-injector
  cargo build --release
  ```

  This will create an executable in the `target/release` directory.

  ## Usage
  - This project is developed to inject instructions in WASM modules or polkadot conformance testing. You can download a sample WASM file from [here](https://github.com/paritytech/cumulus/releases/tag/parachains-v9430).
  - The script will automatically unhexify and decompress WASM modules passed in as input
  - The general format to run the `wasm-injector` is as follows:

  ```sh
  ./wasm-injector [INJECTION] [SOURCE] [DESTINATION] [--compressed] [--hexified]
  ```

  - `[INJECTION]` (required): The type of injection to apply to the wasm module. Possible values ```possible values: [nothing, infinite-loop, jibberish-return-value, stack-overflow, noops, heap-overflow]```

  - `[SOURCE]` (required): A path to the wasm source file. If the wasm module is compressed or hexified, indicate this in the path name as "(hexified)" and/or "(compressed)".

  - `[DESTINATION]` (optional): A path where you want the output wasm file to be saved. If this argument is not provided, the modified wasm file will be saved alongside the source file with a predefined file name.

  - `--compressed` (optional): If this flag is provided, the output file will be compressed.

  - `--hexified` (optional): If this flag is provided, the output file will be hexified (converted to a hexadecimal representation).

## Examples

  To inject code into a wasm file, compress and hexify it, you can run:

  ```sh
  ./wasm-injector noops my_wasm_file.wasm --compressed --hexified
  ```

  To specify a custom destination path, you can run:

  ```sh
  ./wasm-injector noops my_wasm_file.wasm my_destination_directory/my_new_file.wasm
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
