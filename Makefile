.PHONY: build test

default: build

build:
	cargo build

pc-install:
	pre-commit install

checks: pc-run test

pc-run:
	pre-commit run -a

test:
	cargo test --all-features --workspace

fmt:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

doc:
	cargo doc --no-deps --document-private-items --all-features --workspace --examples

docker: compose-up

compose-up:
	docker compose up --build

compose-down:
	docker compose down
