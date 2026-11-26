.PHONY: dna build test audit rea-gate rea-gate-kernel rea-gate-step4 rea-gate-all

dna:          ## Deterministic full-repo regeneration (Deterministic Regeneration)
	@echo "KIAN DNA: regenerate codegen + fixtures deterministically"

build:
	cargo build --workspace --release

test:
	cargo test --workspace

audit:
	cargo clippy --workspace --all-targets -- -D warnings
	cargo deny check

rea-gate:     ## REA gate step 1 — frontend (step 1 of 5)
	node tools/rea/release-exposure.mjs apps/web/.next/standalone

rea-gate-kernel: ## REA gate step 2 — symbol-stripped pool kernel binary (step 2 of 5)
	cargo build --release -p kian-server
	node tools/rea/release-exposure.mjs target/release/kian-server

rea-gate-step4: ## REA gate step 4 — admin and brain: X-KIAN-Key · redaction · sandbox
	cargo build --release -p kian-server
	node tools/rea/step4-gate.mjs

rea-gate-step5: ## Final gate step 5 — governance + quad-chain + 10/10
	node tools/rea/step5-gate.mjs

rea-gate-all: ## Both gates — full release
	node tools/rea/release-exposure.mjs apps/web/.next/standalone
	node tools/rea/release-exposure.mjs target/release/kian-server
