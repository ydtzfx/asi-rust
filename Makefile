.PHONY: build release test lint fmt clean run css deploy

# Build debug
build:
	cargo build

# Build release
release:
	cargo build --release

# Run all tests
test:
	cargo test --workspace

# Run a single test (usage: make test-one TEST=test_name)
test-one:
	cargo test -- $(TEST) --nocapture

# Lint with clippy
lint:
	cargo clippy --all-targets -- -D warnings

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt --check

# Build Tailwind CSS
css:
	npx @tailwindcss/cli -i styles/input.css -o static/styles.css --minify

# Clean build artifacts
clean:
	cargo clean

# Run the server (loads .env)
run:
	export $$(grep -v '^#' .env | xargs) && ./target/release/asi-server

# Full CI pipeline locally
ci: fmt-check lint test release
	@echo "CI pipeline passed"

# Deploy to Vercel (requires VERCEL_TOKEN)
deploy:
	npx vercel deploy --prod
