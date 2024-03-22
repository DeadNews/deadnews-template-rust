.PHONY: all clean default build install checks pc test

default: build

build:
	cargo build

install:
	pre-commit install

checks: pc test

pc:
	pre-commit run -a

test:
	cargo test --all-features --workspace

fmt:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

doc:
	cargo doc --no-deps --document-private-items --all-features --workspace --examples
