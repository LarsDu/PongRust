
run:
	cargo run

setup_wasm:
	rustup target install wasm32-unknown-unknown
	cargo install -f wasm-bindgen-cli

run_wasm: setup_wasm
	# Run a minimal server with the game compiled into WASM
	cargo run --target wasm32-unknown-unknown

wasm: setup_wasm
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir ./out_wasm/ --target web ./target/
