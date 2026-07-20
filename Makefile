.PHONY: help build run test lint format verify clean bump install publish bump-formula smoke-test

BINARY   := hackertuah
TAP_REPO := https://github.com/program247365/homebrew-tap.git
TAP_DIR  := /tmp/homebrew-tap-update
VERSION  := $(shell cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json;print(json.load(sys.stdin)['packages'][0]['version'])")

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make \033[36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

build: ## Build debug binary
	cargo build

run: ## Run the TUI
	cargo run

test: ## Run tests
	cargo test

lint: ## Run clippy with warnings as errors
	cargo clippy -- -D warnings

format: ## Run cargo fmt
	cargo fmt

verify: format lint build test ## Format, lint, build, and test (use after any change)

clean: ## Remove build artifacts
	cargo clean

bump: ## Bump version with cog (cog bump --auto; pre-bump hooks sync Cargo.toml)
	cog bump --auto

install: ## Build release binary and install to ~/bin
	mkdir -p ~/bin
	cargo build --release
	cp target/release/$(BINARY) ~/bin/$(BINARY)

# ── Homebrew release workflow ──────────────────────────────────────────────────
# Usage:
#   make publish       — bump version, push, create GitHub release, update formula
#   make bump-formula  — update Homebrew tap formula to current version only
#   make smoke-test    — verify the published formula installs and runs cleanly

publish: ## Bump version, push, create GitHub release, update Homebrew formula
	cog bump --auto
	@TAG=$$(git describe --tags --abbrev=0); \
	VERSION=$${TAG#v}; \
	echo "Publishing $$TAG..."; \
	git push origin main && \
	git push origin "$$TAG" && \
	gh release create "$$TAG" \
		--repo program247365/hackertuah \
		--title "$$TAG" \
		--generate-notes && \
	$(MAKE) bump-formula TAG=$$TAG VERSION=$$VERSION

bump-formula: ## Update Homebrew tap formula to current version (requires VERSION and TAG)
	$(eval TAG ?= $(shell git describe --tags --abbrev=0))
	$(eval VERSION ?= $(shell echo "$(TAG)" | sed 's/^v//'))
	$(eval SHA256 := $(shell curl -sL "https://github.com/program247365/hackertuah/archive/refs/tags/$(TAG).tar.gz" | shasum -a 256 | awk '{print $$1}'))
	@echo "Updating formula: v$(VERSION) sha256=$(SHA256)"
	rm -rf $(TAP_DIR)
	git clone $(TAP_REPO) $(TAP_DIR)
	python3 -c "\
v='$(VERSION)'; s='$(SHA256)'; \
content = '''class Hackertuah < Formula\n\
  desc \"Terminal UI for browsing Hacker News\"\n\
  homepage \"https://github.com/program247365/hackertuah\"\n\
  url \"https://github.com/program247365/hackertuah/archive/refs/tags/v{v}.tar.gz\"\n\
  sha256 \"{s}\"\n\
  license \"MIT\"\n\
  head \"https://github.com/program247365/hackertuah.git\", branch: \"main\"\n\
\n\
  depends_on \"rust\" => :build\n\
\n\
  def install\n\
    system \"cargo\", \"install\", *std_cargo_args\n\
  end\n\
\n\
  test do\n\
    assert_predicate bin/\"hackertuah\", :exist?\n\
  end\n\
end\n\
'''.format(v=v, s=s); \
open('$(TAP_DIR)/Formula/hackertuah.rb', 'w').write(content)"
	cd $(TAP_DIR) && git add Formula/hackertuah.rb && \
		git commit -m "Update hackertuah to v$(VERSION)" && \
		git push origin main
	rm -rf $(TAP_DIR)
	@echo "Done. Install with: brew tap program247365/tap && brew install hackertuah"

smoke-test: ## Verify the published formula installs and runs cleanly
	@echo "Smoke-testing brew install for v$(VERSION)..."
	@brew tap program247365/tap >/dev/null 2>&1 || true
	@echo "==> Refreshing tap..."
	@brew update --quiet
	@echo "==> Asserting tap formula version matches Cargo.toml..."
	@TAP_VERSION=$$(brew cat program247365/tap/$(BINARY) | sed -n 's|.*tags/v\(.*\)\.tar\.gz.*|\1|p'); \
	  if [ "$$TAP_VERSION" != "$(VERSION)" ]; then \
	    echo "FAIL: tap has v$$TAP_VERSION but Cargo.toml is v$(VERSION) — run 'make bump-formula' first"; \
	    exit 1; \
	  fi; \
	  echo "  tap version: $$TAP_VERSION"
	@echo "==> Installing (builds from source)..."
	@if brew list --versions program247365/tap/$(BINARY) >/dev/null 2>&1; then \
	  brew reinstall program247365/tap/$(BINARY); \
	else \
	  brew install program247365/tap/$(BINARY); \
	fi
	@echo "==> Verifying binary..."
	@BIN="$$(brew --prefix)/bin/$(BINARY)"; \
	  INSTALLED=$$("$$BIN" --version | awk '{print $$2}'); \
	  if [ "$$INSTALLED" != "$(VERSION)" ]; then \
	    echo "FAIL: $$BIN --version reports v$$INSTALLED but expected v$(VERSION)"; \
	    exit 1; \
	  fi; \
	  echo "  $(BINARY) --version: $$INSTALLED"
	@echo ""
	@echo "Smoke test passed for v$(VERSION)."
