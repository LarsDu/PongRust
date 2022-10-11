
run:
	cargo run

prereqs:
	cargo install -f wasm-bindgen-cli
	cargo install wasm-server-runner

install_wasm: prereqs
	rustup target install wasm32-unknown-unknown


run_wasm: setup_wasm
	# Run a minimal server with the game compiled into WASM
	cargo run --release --target wasm32-unknown-unknown

watch_wasm:
	cargo watch -cx "run --release --target wasm32-unknown-unknown"

build_wasm: install_wasm
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/pong_rust.wasm 

