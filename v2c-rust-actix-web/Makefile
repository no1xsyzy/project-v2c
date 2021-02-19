.PHONY: all
all: target/release/v2c-rust-actix-web

target/release/v2c-rust-actix-web: Cargo.toml src/*.rs
	cargo build --release

.PHONY: run
run: all
	cargo run --release
