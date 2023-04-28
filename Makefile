all: b r t

b:
	- cargo build

r:
	- cargo build --release

t:
	- cargo test

lint:
	- cargo fmt --all -- --check
	- cargo clippy -- -D warnings