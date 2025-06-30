.PHONY: all clean default run build install checks lint pc test release

default: checks

run:
	cargo run

build:
	cargo build

goreleaser:
	goreleaser --clean --snapshot --skip=publish

install:
	pre-commit install

update:
	cargo update

checks: pc lint test
pc:
	pre-commit run -a
lint:
	cargo fmt --all --check
	cargo clippy --all-targets --all-features -- -D warnings
test:
	cargo test --all-features

test-cov:
	cargo llvm-cov --ignore-filename-regex 'test.rs'

doc:
	cargo doc --no-deps --document-private-items --all-features --examples

bumped:
	git cliff --bumped-version

# make release TAG=$(git cliff --bumped-version)-alpha.0
release: checks
	git cliff -o CHANGELOG.md --tag $(TAG)
	pre-commit run --files CHANGELOG.md || pre-commit run --files CHANGELOG.md
	git add CHANGELOG.md
	git commit -m "chore(release): prepare for $(TAG)"
	git push
	git tag -a $(TAG) -m "chore(release): $(TAG)"
	git push origin $(TAG)
