all:
	cargo build --release
	cp target/wasm32-unknown-unknown/release/maze_wanderer.wasm .
	wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code maze_wanderer.wasm -o maze_wanderer.wasm
	
	# enable wasm-opt for a tiny reduction but it won't run in the native runtime
	# wasm-opt -Oz --strip-producers --dce maze_wanderer.wasm -o maze_wanderer.wasm

run: all
	# w4 run-native maze_wanderer.wasm
	w4 run --no-qr --no-open maze_wanderer.wasm

dev:
	cargo watch -s "make run"
	
bundle: all
	mkdir -p bundle/

	w4 bundle maze_wanderer.wasm \
		--title "Maze Wanderer" \
		--description "Have Fun!" \
		--html bundle/maze_wanderer.html \
		--mac bundle/maze_wanderer.bin \
		--linux bundle/maze_wanderer.linux \
		--windows bundle/maze_wanderer.exe