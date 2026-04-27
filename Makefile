SHELL := /bin/bash

.PHONY: help fmt test smoke-cli smoke-generate smoke-generate-current-conf verify ci check dry-publish

help:
	@echo "Available targets:"
	@echo "  make fmt             - Check formatting"
	@echo "  make test            - Run verification test suite"
	@echo "  make smoke-cli       - Run CLI smoke checks"
	@echo "  make smoke-generate  - Run end-to-end generation smoke check"
	@echo "  make smoke-generate-current-conf - Run smoke check using repo tauri.conf.json (or fallback in CI)"
	@echo "  make verify          - Run full local verification (fmt + tests + all smokes)"
	@echo "  make ci              - Alias of verify"
	@echo "  make check           - Cargo check with verify-signature feature"
	@echo "  make dry-publish     - Cargo publish dry-run"

fmt:
	cargo fmt --all -- --check

test:
	bash scripts/test.sh

smoke-cli:
	bash scripts/smoke-cli.sh

smoke-generate:
	bash scripts/smoke-generate.sh

smoke-generate-current-conf:
	bash scripts/smoke-generate-current-conf.sh

verify: fmt test smoke-cli smoke-generate smoke-generate-current-conf

ci: verify

check:
	cargo check --features verify-signature

dry-publish:
	cargo publish --dry-run --allow-dirty
