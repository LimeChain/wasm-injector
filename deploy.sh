#!/bin/sh

dir="${1}"
file="${2}"

cargo run -- "${dir}" "${file}"
scp "${dir}/hexified_injected_${file}.hex" zt-node:wasm
