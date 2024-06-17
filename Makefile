install:
	rustup override set nightly
	rustup toolchain list
	rustup target add x86_64-unknown-linux-musl

build:
	cargo build --release --target x86_64-unknown-linux-musl
