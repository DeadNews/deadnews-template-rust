.PHONY: all clean default build install checks lint pc test

default: build

build:
	cargo build

goreleaser:
	goreleaser --clean --snapshot --skip=publish

install:
	pre-commit install

update:
	cargo update

checks: pc lint test
lint: fmt clippy
pc:
	pre-commit run -a
fmt:
	cargo fmt --all --check
clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings
test:
	cargo test --all-features --workspace

doc:
	cargo doc --no-deps --document-private-items --all-features --workspace --examples

bumped:
	git cliff --bumped-version

# make release-tag_name
# make release-$(git cliff --bumped-version)-alpha.0
release-%: checks
	git cliff -o CHANGELOG.md --tag $*
	pre-commit run --files CHANGELOG.md || pre-commit run --files CHANGELOG.md
	git add CHANGELOG.md
	git commit -m "chore(release): prepare for $*"
	git push
	git tag -a $* -m "chore(release): $*"
	git push origin $*
	git tag --verify $*
