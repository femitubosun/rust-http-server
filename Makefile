.PHONY: build run clean test

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

clean:
	cargo clean
