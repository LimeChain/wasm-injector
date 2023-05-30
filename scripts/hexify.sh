#!/usr/bin/env bash

src="${1}"
dest="${2}"

xxd -p "${src}" | tr -d "\n" | sed "s/^/0x/" > "${dest}"
