config = debug

ifeq (debug, $(config))
run:
	env RUST_LOG=trace cargo run
endif
ifeq (release, $(config))
run:
	env RUST_LOG=info cargo run --release
endif