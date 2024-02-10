config = debug

ifeq (debug, $(config))
run:
	env RUST_LOG=debug RUST_BACKTRACE=1 cargo run
endif
ifeq (release, $(config))
run:
	env RUST_LOG=info RUST_BACKTRACE=1 cargo run --release
endif