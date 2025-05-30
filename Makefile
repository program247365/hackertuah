.PHONY: help build run test lint clean bump

help:
	@echo "Available commands:"
	@echo "  make build   - Build the project (cargo build)"
	@echo "  make run     - Run the project (cargo run)"
	@echo "  make test    - Run tests (cargo test)"
	@echo "  make lint    - Run clippy linter (cargo clippy)"
	@echo "  make clean   - Clean build artifacts (cargo clean)"
	@echo "  make bump    - Bump version with cog (cog bump --auto)"

build:
	cargo build

run:
	cargo run

test:
	cargo test

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean

bump:
	cog bump --auto 