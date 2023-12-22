trunk build --release --filehash false --dist ../docs
wasm-opt -Oz ../docs/egui-gizmo-demo_bg.wasm --output ../docs/egui-gizmo-demo_bg.wasm
