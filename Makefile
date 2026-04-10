.PHONY: help build run test lint format verify clean bump install publish bump-formula

BINARY   := hackertuah
TAP_REPO := https://github.com/program247365/homebrew-tap.git
TAP_DIR  := /tmp/homebrew-tap-update
VERSION  := $(shell cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json;print(json.load(sys.stdin)['packages'][0]['version'])")

help:
	@echo "Available commands:"
	@echo "  make build        - Build the project (cargo build)"
	@echo "  make run          - Run the project (cargo run)"
	@echo "  make test         - Run tests (cargo test)"
	@echo "  make lint         - Run clippy linter (cargo clippy)"
	@echo "  make format       - Run cargo fmt"
	@echo "  make verify       - Format, lint, build, and test (use after any change)"
	@echo "  make clean        - Clean build artifacts (cargo clean)"
	@echo "  make bump         - Bump version with cog (cog bump --auto)"
	@echo "  make install      - Build and install binary to ~/bin"
	@echo "  make publish      - Bump version, create GitHub release, update Homebrew formula"
	@echo "  make bump-formula - Update Homebrew tap formula to current version"

build:
	cargo build

run:
	cargo run

test:
	cargo test

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

verify: format lint build test

clean:
	cargo clean

bump:
	cog bump --auto

install:
	mkdir -p ~/bin
	cargo build --release
	cp target/release/$(BINARY) ~/bin/$(BINARY)

# ── Homebrew release workflow ──────────────────────────────────────────────────
# Usage:
#   make publish       — bump version, push, create GitHub release, update formula
#   make bump-formula  — update Homebrew tap formula to current version only

publish: ## Bump version, push, create GitHub release, update Homebrew formula
	cog bump --auto
	$(eval VERSION := $(shell cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json;print(json.load(sys.stdin)['packages'][0]['version'])"))
	@echo "Publishing v$(VERSION)..."
	git push origin main
	git push origin v$(VERSION)
	gh release create v$(VERSION) \
		--repo program247365/hackertuah \
		--title "v$(VERSION)" \
		--generate-notes
	$(MAKE) bump-formula VERSION=$(VERSION)

bump-formula: ## Update Homebrew tap formula to current version
	$(eval SHA256 := $(shell curl -sL "https://github.com/program247365/hackertuah/archive/refs/tags/v$(VERSION).tar.gz" | shasum -a 256 | awk '{print $$1}'))
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
