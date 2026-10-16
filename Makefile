# ruse workspace helpers
# Prefer these targets for day-to-day work; they wrap cargo / cargo xtask.

.DEFAULT_GOAL := help

CARGO        ?= cargo
XTASK        := $(CARGO) xtask
CHANNEL      ?= dev
PREFIX       ?= $(HOME)/.cargo
INSTALL_PATH ?= $(PREFIX)

.PHONY: help install build release check fmt fmt-check clippy test ci eval \
	desktop-test package-cli package-desktop support-bundle run clean watch docs

help: ## Show available targets
	@printf '%s\n' \
		'Usage: make <target> [VAR=value]' \
		'' \
		'Targets:'
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*?## / {printf "  %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@printf '%s\n' \
		'' \
		'Variables:' \
		'  CHANNEL=$(CHANNEL)          package channel (dev|beta|stable)' \
		'  PREFIX=$(PREFIX)            cargo install --root prefix' \
		'  CARGO=$(CARGO)' \
		'' \
		'Also available: cargo xtask help'

install: ## Install CLI binaries (ruse, ruse-cli) into PREFIX
	# Touch the bin entrypoints so cargo invalidates install fingerprints even
	# when only path-deps (ruse-tui/core/…) changed — otherwise `cargo install
	# --path --force` can skip recompilation and leave a stale binary that
	# fails to enter the TUI after source edits.
	touch crates/ruse-cli/src/bin/ruse.rs crates/ruse-cli/src/bin/ruse-cli.rs
	$(CARGO) install --path crates/ruse-cli --root $(INSTALL_PATH) --locked --force

build: ## Debug build of the workspace
	$(CARGO) build --workspace

release: ## Release build of the workspace
	$(CARGO) build --workspace --release

check: ## fmt-check + clippy + test (via xtask)
	$(XTASK) check

fmt: ## Format all Rust sources
	$(XTASK) fmt

fmt-check: ## Check formatting without writing
	$(XTASK) fmt-check

clippy: ## Clippy with -D warnings
	$(XTASK) clippy

test: ## Run workspace tests
	$(XTASK) test

ci: ## Full quality gate with evidence artifacts
	$(XTASK) ci

eval: ## Run the default evaluation corpus
	$(XTASK) eval

desktop-test: ## Desktop Rust / Tauri / frontend test + build gate
	$(XTASK) desktop-test

package-cli: ## Build and smoke-test a local CLI release artifact
	$(XTASK) package-cli --channel $(CHANNEL)

package-desktop: ## Plan/build and smoke-test a local Desktop release
	$(XTASK) package-desktop --channel $(CHANNEL)

support-bundle: ## Generate a redacted local support bundle
	$(XTASK) support-bundle --channel $(CHANNEL)

run: ## Run the agent CLI (pass ARGS='...')
	$(CARGO) run -p ruse-cli --bin ruse -- $(ARGS)

clean: ## Remove Cargo target/ and common local artifacts
	$(CARGO) clean
	rm -rf target/ruse-ci target/ruse-eval target/agent-support

watch: ## Re-run checks on change (requires cargo-watch)
	$(CARGO) watch -x 'xtask check'

docs: ## Build workspace rustdoc
	$(CARGO) doc --workspace --no-deps --open
