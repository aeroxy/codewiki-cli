.PHONY: build check run clean bump-patch bump-minor bump-major update-formula

LINUX_TARGET = x86_64-unknown-linux-gnu
LINUX_OUT    = target/$(LINUX_TARGET)/release

## Build the full project (frontend + backend, debug)
build:
	cargo build

## Release build
release:
	cargo build --release
	zip -j target/release/codewiki_macos_arm64.zip target/release/codewiki
	@echo ""
	@echo "All platform zips ready:"
	@echo "  target/release/codewiki_macos_arm64.zip"

## Type-check without producing a binary
check:
	cargo check

## Run the CLI
run:
	cargo run -- structure facebook/react

## Remove build artifacts
clean:
	cargo clean

## Bump the patch version (0.1.0 → 0.1.1) and update all version references
bump-patch:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	patch=$$(echo $$old | cut -d. -f3); \
	new="$$major.$$minor.$$((patch+1))"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' "s/version \"$$old\"/version \"$$new\"/" Formula/codewiki-cli.rb; \
	echo "$$old → $$new"

## Bump the minor version (0.1.1 → 0.2.0) and update all version references
bump-minor:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	new="$$major.$$((minor+1)).0"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' "s/version \"$$old\"/version \"$$new\"/" Formula/codewiki-cli.rb; \
	echo "$$old → $$new"

## Bump the major version (0.1.1 → 1.0.0) and update all version references
bump-major:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	new="$$((major+1)).0.0"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' "s/version \"$$old\"/version \"$$new\"/" Formula/codewiki-cli.rb; \
	echo "$$old → $$new"

## Update Formula/codewiki-cli.rb SHA256 from local release zips
## (run after release-all, before upload)
##   make update-formula
update-formula:
	@mac_zip="target/release/codewiki_macos_arm64.zip"; \
	echo "Computing macOS ARM SHA256 …"; \
	mac_sha=$$(shasum -a 256 "$$mac_zip" | cut -d' ' -f1); \
	echo "macOS ARM SHA256: $$mac_sha"; \
	sed -i '' "/codewiki_macos_arm64\.zip/{n; s/sha256.*/sha256 \"$$mac_sha\"/;}" Formula/codewiki-cli.rb; \
	echo "Formula/codewiki-cli.rb updated"
