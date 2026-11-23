.PHONY: dna build test audit rea-gate rea-gate-kernel rea-gate-step4 rea-gate-all

dna:          ## بازتولید قطعی کل ریپو (Deterministic Regeneration)
	@echo "KIAN DNA: regenerate codegen + fixtures deterministically"

build:
	cargo build --workspace --release

test:
	cargo test --workspace

audit:
	cargo clippy --workspace --all-targets -- -D warnings
	cargo deny check

rea-gate:     ## دروازه REA گام ۱ — فرانتاند (گام ۱ از ۵)
	node tools/rea/release-exposure.mjs apps/web/.next/standalone

rea-gate-kernel: ## دروازه REA گام ۲ — باینری کرنل پول بدون سیمبل (گام ۲ از ۵)
	cargo build --release -p kian-server
	node tools/rea/release-exposure.mjs target/release/kian-server

rea-gate-step4: ## دروازه REA گام ۴ — ادمین و مغز: X-KIAN-Key · redaction · sandbox
	cargo build --release -p kian-server
	node tools/rea/step4-gate.mjs

rea-gate-step5: ## دروازه نهایی گام ۵ — حاکمیت + چهارزنجیره + ۱۰/۱۰
	node tools/rea/step5-gate.mjs

rea-gate-all: ## هر دو دروازه — انتشار کامل
	node tools/rea/release-exposure.mjs apps/web/.next/standalone
	node tools/rea/release-exposure.mjs target/release/kian-server
