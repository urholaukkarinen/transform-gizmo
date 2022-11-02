cargo build -p egui-gizmo-demo --target wasm32-unknown-unknown --release
cp ../target/wasm32-unknown-unknown/release/egui-gizmo-demo.wasm ../docs/demo.wasm
