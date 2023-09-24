build:
	cargo build

checks: test fmt clippy

test:
	cargo test --all-features --workspace

fmt:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

doc:
	cargo doc --no-deps --document-private-items --all-features --workspace --examples
