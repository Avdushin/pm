.PHONY: Default build

Default:
	cargo run -q

build:
	cargo build --release
	cp target/release/pm .
