.PHONY: help build run test lint clean bump install

help:
	@echo "Available commands:"
	@echo "  make build   - Build the project (cargo build)"
	@echo "  make run     - Run the project (cargo run)"
	@echo "  make test    - Run tests (cargo test)"
	@echo "  make lint    - Run clippy linter (cargo clippy)"
	@echo "  make clean   - Clean build artifacts (cargo clean)"
	@echo "  make bump    - Bump version with cog (cog bump --auto)"
	@echo "  make install - Build and install binary to system bin directory"

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

install:
	@echo "Detecting operating system..."
	@if [ "$(OS)" = "Windows_NT" ]; then \
		echo "Installing for Windows..."; \
		cargo build --release; \
		mkdir -p "$(USERPROFILE)/bin"; \
		cp target/release/hackertuah.exe "$(USERPROFILE)/bin/"; \
		echo "Checking PATH configuration..."; \
		if ! echo "$$PATH" | grep -q "$(USERPROFILE)/bin"; then \
			echo "Adding $(USERPROFILE)/bin to PATH..."; \
			echo 'export PATH="$(USERPROFILE)/bin:$$PATH"' >> "$(USERPROFILE)/.bashrc"; \
			echo 'export PATH="$(USERPROFILE)/bin:$$PATH"' >> "$(USERPROFILE)/.zshrc" 2>/dev/null || true; \
			echo "PATH updated. Please restart your terminal or run: source ~/.bashrc"; \
		else \
			echo "$(USERPROFILE)/bin is already in PATH"; \
		fi; \
	elif [ "$(shell uname)" = "Darwin" ]; then \
		echo "Installing for macOS..."; \
		mkdir -p ~/bin; \
		cargo build --release; \
		cp target/release/hackertuah ~/bin/; \
		echo "Checking PATH configuration..."; \
		if ! echo "$$PATH" | grep -q "~/bin\|$$HOME/bin"; then \
			echo "Adding ~/bin to PATH..."; \
			echo 'export PATH="$$HOME/bin:$$PATH"' >> ~/.zshrc; \
			echo 'export PATH="$$HOME/bin:$$PATH"' >> ~/.bash_profile 2>/dev/null || true; \
			echo "PATH updated. Please restart your terminal or run: source ~/.zshrc"; \
		else \
			echo "~/bin is already in PATH"; \
		fi; \
	else \
		echo "Installing for Linux..."; \
		mkdir -p ~/bin; \
		cargo build --release; \
		cp target/release/hackertuah ~/bin/; \
		echo "Checking PATH configuration..."; \
		if ! echo "$$PATH" | grep -q "~/bin\|$$HOME/bin"; then \
			echo "Adding ~/bin to PATH..."; \
			echo 'export PATH="$$HOME/bin:$$PATH"' >> ~/.bashrc; \
			echo 'export PATH="$$HOME/bin:$$PATH"' >> ~/.zshrc 2>/dev/null || true; \
			echo "PATH updated. Please restart your terminal or run: source ~/.bashrc"; \
		else \
			echo "~/bin is already in PATH"; \
		fi; \
	fi 