.PHONY: all
all: target/release/v2c-rust-hyper

target/release/v2c-rust-hyper: Cargo.toml src/*.rs
	cargo build --release

.PHONY: run
run: all
	cargo run --release
