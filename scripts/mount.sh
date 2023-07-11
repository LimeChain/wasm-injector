#!/bin/bash

dirs=(tests wasm)

for dir in ${dirs[@]}; do
    local_dir="remote_${dir}"
    mkdir -p "${local_dir}"
    nix run nixpkgs#rclone -- mount -v "zt-node:${dir}" "${local_dir}" 1>/dev/null 2>&1 &
    # disown
done
