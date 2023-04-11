# wasm

## build(gen wasm) build the example for the wasm target, creating a binary

cargo build --release --target wasm32-unknown-unknown

## bind(js) wasm-bindgen-cli is used to create javascript bindings to this wasm file, which can be loaded using in html

wasm-bindgen --out-name mcrs --out-dir wasm/target --target web target/wasm32-unknown-unknown/release/mcrs.wasm

## run

basic-http-server ./wasm/target
