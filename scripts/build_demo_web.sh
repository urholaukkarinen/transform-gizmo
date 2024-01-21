#!/usr/bin/env bash

set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

WASM_PATH="docs/egui-gizmo-demo_bg.wasm"

pushd "$script_path/../demo"
trunk build --config Trunk.toml --release
popd

wasm-opt "$WASM_PATH" -O2 --fast-math -o "$WASM_PATH"