all:
	cargo build --release
	cp target/wasm32-unknown-unknown/release/walking_simulator.wasm .
	wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code walking_simulator.wasm -o walking_simulator.wasm
	
	# enable wasm-opt for a tiny reduction but it won't run in the native runtime
	# wasm-opt -Oz --strip-producers --dce walking_simulator.wasm -o walking_simulator.wasm

run: all
	# w4 run-native walking_simulator.wasm
	w4 run --no-qr --no-open walking_simulator.wasm

dev:
	cargo watch -s "make run"
	
bundle: all
	mkdir -p bundle/

	w4 bundle walking_simulator.wasm \
		--title "Walking Simulator" \
		--description "Have Fun!" \
		--html bundle/walking_simulator.html \
		--mac bundle/walking_simulator.bin \
		--linux bundle/walking_simulator.linux \
		--windows bundle/walking_simulator.exe