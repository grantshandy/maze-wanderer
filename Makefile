all:
	cargo build --release
	cp target/wasm32-unknown-unknown/release/raycaster.wasm .
	wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code raycaster.wasm -o raycaster.wasm
	
	# enable wasm-opt for a tiny reduction but it won't run in the native runtime
	# wasm-opt -Oz --strip-producers --dce raycaster.wasm -o raycaster.wasm

run: all
	# w4 run-native raycaster.wasm
	w4 run --no-qr --no-open raycaster.wasm

dev:
	cargo watch -s "make run"