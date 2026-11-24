# گاوصندوق جواهرات مهندسی (3 مخزن: Firecrawl · Dify · JavaGuide)

- **نسخه / کامیت‌های مرجع:**
  - `firecrawl/firecrawl` @ `8538d3a`
  - `langgenius/dify` @ `a92a64aa`
  - `Snailclimb/JavaGuide` @ `992dae3`

## ساختار پوشه‌ها

1. `01_firecrawl_extraction_and_ssrf/`
   - `html.rs`, `markdown.rs`, `fetch.rs`, `guard.rs`, `netguard.rs`: هستهٔ Rust برای پاک‌سازی HTML به Markdown بهینه برای LLM، حفاظت دو‌مرحله‌ای SSRF/DNS-Rebinding (شامل NAT64، CGNAT و آدرس‌های متادیتای ابری `169.254.169.254` و `100.100.100.200`)، و پروکسی SOCKS5 اجباری مرورگر.
   - `removeUnwantedElements.ts`, `html-to-markdown.ts`, `llmExtract.ts`: حذف ۴۰+ تگ زائد، تبدیل چندلایه HTML-to-Markdown و پرامپت‌های Strict Structured Output.
2. `02_dify_workflow_rag_and_guardrails/`
   - `llm_generator_prompts.py`, `planner_prompts.py`, `node_builder_prompts.py`, `graph_topology.py`: پرامپت‌های استاندارد تولید عنوان، کد سندباکس، QA، JSON Schema و معمار گراف ۱۸ گره‌ای Dify.
   - `input_moderation.py`, `output_moderation.py`: ماشین حالت Moderation جریانی بدون قفل (`_checked_length` + latch خطای قطعی).
   - `token_buffer_memory.py`, `fixed_text_splitter.py`: مدیریت حافظه مبتنی بر توکن و تقسیم‌کننده سلسله‌مراتبی RAG (`["\n\n", "\n", "。", ". ", " ", ""]`).
3. `03_javaguide_distributed_and_ledger/`
   - `distributed-transaction.md`, `state-machine-and-workflow.md`: الگوهای XA/2PC، TCC، Saga، Local Message Table (Outbox) و کنترل هم‌زمانی ماشین حالت با کلید Idempotency پایدار.
   - `3-commonly-used-cache-read-and-write-strategies.md`, `redis-common-blocking-problems-summary.md`, `innodb-implementation-of-mvcc.md`, `mysql-high-performance-optimization-specification-recommendations.md`, `fallback-and-circuit-breaker.md`: استراتژی‌های کش، دفاع در برابر Cache Penetration/Breakdown/Avalanche، MVCC در InnoDB و تاب‌آوری مدارقطع‌کن.
4. `04_kian_v2_rust_adapters/`
   - `kian_firecrawl_ssrf_and_md.rs`: آداپتور Rust حفاظت SSRF و نرمال‌سازی Markdown برای ابزارهای وب KIAN v2.
   - `kian_dify_rag_and_idempotency.rs`: آداپتور Rust تقسیم‌کننده RAG دیفای + فیلتر جریانی خروجی + ماشین حالت CAS/Idempotent برای دفترکل و Escrow در KIAN v2.
