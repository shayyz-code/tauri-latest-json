SHELL := /bin/bash

.PHONY: help fmt clippy test smoke-cli smoke-generate smoke-generate-current-conf smoke-real-app verify ci check dry-publish

help:
	@echo "Available targets:"
	@echo "  make fmt             - Check formatting"
	@echo "  make clippy          - Run clippy with warnings as errors"
	@echo "  make test            - Run verification test suite"
	@echo "  make smoke-cli       - Run CLI smoke checks"
	@echo "  make smoke-generate  - Run end-to-end generation smoke check"
	@echo "  make smoke-generate-current-conf - Run smoke check using repo tauri.conf.json (or fallback in CI)"
	@echo "  make smoke-real-app  - Run smoke check against local real-tauri-app (ignored by git)"
	@echo "  make verify          - Run full local verification (fmt + tests + all smokes)"
	@echo "  make ci              - Alias of verify"
	@echo "  make check           - Cargo check with verify-signature feature"
	@echo "  make dry-publish     - Cargo publish dry-run"

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	bash scripts/test.sh

smoke-cli:
	bash scripts/smoke-cli.sh

smoke-generate:
	bash scripts/smoke-generate.sh

smoke-generate-current-conf:
	bash scripts/smoke-generate-current-conf.sh

smoke-real-app:
	bash scripts/smoke-real-tauri-app.sh

verify: fmt test smoke-cli smoke-generate smoke-generate-current-conf

ci: verify

check:
	cargo check --features verify-signature

dry-publish:
	cargo publish --dry-run --allow-dirty
