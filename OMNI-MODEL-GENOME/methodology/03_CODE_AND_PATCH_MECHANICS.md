#FORENSIC COGNITIVE TELEMETRY PROBE — LAYER 03/09: CODE ARCHITECTURE, IDIOMS & SURGICAL PATCH MECHANICS

You are generating the third specification file (`03_CODE_AND_PATCH_MECHANICS.md`) of the AI Ecosystem Genome. Your objective is to reverse-engineer your software engineering DNA: language proficiency hierarchy, architectural biases, surgical code-patching reliability, and compiler/runtime hallucination signatures.

Output ONLY the raw Markdown file content starting with the YAML frontmatter (`---`).

---
file_id: "03_CODE_AND_PATCH_MECHANICS"
schema_version: "1.0.0"
model_self_reported_id: "<fill exact model name/version>"
tier_1_languages: ["<top languages where you write zero-warning idiomatic production code>"]
weakest_languages_or_paradigms: ["<languages/paradigms where syntax or borrow/type errors occur most often>"]
preferred_patch_format: "<search_replace_blocks | unified_udiff | line_numbered_ranges | full_file_rewrite>"
max_safe_single_file_edit_lines: <integer>
lazy_elision_tendency: "<high | medium | low | none>"
---

# 03 — Code Architecture, Idioms & Surgical Patch Mechanics

## 1. Language Proficiency & Architectural Signature
1. **Language & Ecosystem Mastery Matrix:**
   Create a detailed table rating your real-world production reliability (`0.00` to `1.00`), common compiler traps, and crate/library version cutoff issues across:
   - **Rust** (async Tokio, lifetimes, trait objects, `Pin`/`Unsafe`, Axum, SQLite/SQLx)
   - **TypeScript / Next.js / React** (App Router, strict generics, WebSockets, Canvas/WebGL)
   - **Python** (asyncio, Pydantic v2, FastAPI, SQLite/WAL, data pipelines)
   - **Systems / Shell / SQL** (Bash/POSIX, Docker, SQLite concurrency/CAS, iptables/seccomp)
2. **Architectural Style & Bias:**
   - Do you naturally bias toward **Deep Modules** (simple interface, rich internal logic) or **Shallow Boilerplate** (excessive interfaces, tiny wrapper classes/functions)?
   - How do you handle error propagation by default (e.g., typed `thiserror`/`Result` vs. `.unwrap()`/`any` shortcuts)?

## 2. Live Code & Surgical Patch Telemetry Probes (Execute & Record)

### Probe 2.1: Zero-Copy / Borrow-Checker Stress Test (Rust)
Write a complete, zero-warning, dependency-free Rust function (~25-35 lines) that parses a raw HTTP-like header block `&'a str` into a fixed-capacity stack array `[( &'a str, &'a str ); 16]` without heap allocation (`Vec`/`String` forbidden), handling malformed lines, trimming ASCII whitespace, and rejecting header keys containing control characters.
- After writing the code, audit every lifetime and slice index for panic/UB edge cases.

### Probe 2.2: Surgical Patch Formatting & Indentation Fidelity
Compare your internal token-generation reliability across the 3 major agentic edit formats:
1. **`<<<<<<< SEARCH / ======= / >>>>>>> REPLACE` blocks**
2. **Unified Diff (`@@ -l,s +l,s @@`)**
3. **JSON-escaped string replacement (`{"old_str": "...", "new_str": "..."}`)**
Document:
- Why and when do you corrupt line numbers in Unified Diffs?
- How often do you accidentally strip leading whitespace/tabs or corrupt escaped quotes in JSON tool arguments vs. raw fenced blocks?
- What is the exact anchor context size (e.g., 3 lines above and below) that reduces your patch mismatch rate to `< 1%`?

## 3. Code Hallucination Signatures & Anti-Laziness Compiler Rules
1. **Top 5 Hallucinated APIs / Outdated Idioms:**
   List the specific library versions or APIs you most frequently confuse (e.g., mixing v1/v2 APIs, deprecated Node/Rust/Python methods, or inventing non-existent helper methods).
2. **Lazy Elision Triggers & Antidotes:**
   - Under what exact conditions do you emit placeholders like `// ... existing code ...`, `// TODO: implement`, or `pass`?
   - Write the exact, Battle-Tested System Directive (3-5 lines) that guarantees 100% complete, compilation-ready code output from you with zero placeholders.