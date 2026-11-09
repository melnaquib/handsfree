.PHONY: help build test quality-gate dev deploy clean

help:
	@echo "Available commands:"
	@echo "  make build         - Build all servers"
	@echo "  make test          - Run all tests"
	@echo "  make quality-gate  - Run format, clippy, and tests"
	@echo "  make dev           - Start development server with hot reload"
	@echo "  make deploy        - Deploy to production"
	@echo "  make clean         - Clean build artifacts"

build:
	cargo build --release

test:
	cargo test --all-features

quality-gate:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test --all-features

dev:
	@echo "Starting development server..."
	@echo "Use: cargo pmcp dev --server <name>"

deploy:
	@echo "Deploying to production..."
	@echo "Use: cargo pmcp deploy --server <name> --target lambda"

clean:
	cargo clean
