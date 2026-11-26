# 🔗 `06_LINK_Router.md` — HYBRID MULTI-AGENT COGNITIVE ROUTER
> **Bridges:** `01_DNAme.md` $\leftrightarrow$ `03_DNAmind.md` $\leftrightarrow$ `04_DNAharness.md`  
> **Purpose:** Defines how to combine multiple distinct AI models into a single Hybrid Super-Brain.

---

## 1. HYBRID BRAIN TOPOLOGY (`Aragon-4` × `Arena Harvester` × `Z.ai Builder`)

```text
                  ┌──────────────────────────────────────────┐
                  │   NODE-C: CONTROL PLANE (Aragon-4 Mind)  │
                  │  Loads: [01_DNAme + 02_DNAwork + 03_Mind]│
                  └──────┬────────────────────────────┬──────┘
                         │ Issues Sparse              │ Issues Checkpointed
                         │ Harvester Prompt           │ Integration Prompt
                         ▼                            ▼
        ┌────────────────────────────────┐  ┌────────────────────────────────┐
        │   NODE-A: ARENA MAP CLUSTER    │  │   NODE-Z: Z.AI REDUCE CLUSTER  │
        │  Fast Network · 1.98GB RAM     │  │  Persistent Disk · Cargo Build │
        │  Outputs: Verified .zip Vaults ├──►  Outputs: Tested Git Commits   │
        │  + Pure .rs + SQLite Injectors │  │  + Production .db + Final .zip │
        └────────────────────────────────┘  └────────────────────────────────┘
        2. ROUTING RULES FOR HYBRID EXECUTION

    When a task requires scanning external GitHub repos, APIs, or docs:
        Route to NODE-A (Arena) using --depth 1 --filter=blob:none --sparse.
        Require NODE-A to pre-compile any Rust module via standalone rustc --test (zero external dependencies) and package into /home/user/<name>-gems.zip.
        If an NODE-A chat hits context saturation (Empty response), spawn a fresh NODE-A session with a scoped sparse-checkout prompt.
    When a task requires modifying the monorepo, running Cargo tests, or migrating SQLite:
        Route to NODE-Z (Z.ai) in layered checkpoints.
        Separate SQLite Knowledge Injection (python3 inject_*.py -> 0 compile overhead) from Rust Kernel Wiring (crates/*/src/*.rs).
    When evaluating strategy, architecture, or debugging a failure across nodes:
        Route to NODE-C (Control Plane) loaded with 01_DNAme.md + 02_DNAwork.md + 03_DNAmind.md.

text


---

### Seventh File (Communication Bridge 2): `07_LINK_Protocol.md` (Prompt Compiler & Inter-Agent Communication Protocol)

```markdown
# 📡 `07_LINK_Protocol.md` — INTER-AGENT HANDSHAKE & MASTER PROMPT COMPILER
> **Bridges:** `02_DNAwork.md` $\leftrightarrow$ `04_DNAharness.md` $\leftrightarrow$ `05_DNAskills.md`  
> **Purpose:** The exact template the Control Plane must use when writing a `MASTER PROMPT` for `Z.ai` or `Arena`.

---

## 1. MANDATORY 7-BLOCK STRUCTURE FOR EVERY `Z.AI` MASTER PROMPT
Whenever the Control Plane generates an execution prompt for `Z.ai`, it MUST include these 7 deterministic blocks:

1. **Baseline Anchor:** State the exact expected starting commit hash, test count, `"KbEntry"` count, active product count (`20`), and ledger invariant (`trialBalanceSum == 0`).
2. **Build Environment Lock:** Explicitly specify `CARGO_TARGET_DIR=/tmp/my-project/target CARGO_BUILD_JOBS=2`.
3. **Vault Unpack & Sanitization:** Unpack `/home/z/my-project/upload/<vault>.zip` to `/tmp/...`, mount curated blueprints to `docs/vault/...`, and sweep for `sk-`/`ghp_`/`eyJ` secrets -> `DUMMY-TEST-FIXTURE`.
4. **Module-by-Module Specification:** Exact target file paths (`crates/<crate>/src/<module>.rs`), exact struct/function signatures, and `lib.rs` exports.
5. **Idempotent SQLite Injection:** Run `inject_*.py` twice (or verify `inserted` / `updated`) and assert `trialBalanceSum == 0` before and after commit.
6. ** Targeted Verification Gate:** Run `cargo test` and `cargo clippy -- -D warnings` on touched crates + secret scan on `git diff --cached`.
7. **Compact Persian/Bilingual Telemetry Report:** Require a final table comparing `Before` vs `After` metrics and the new commit hash.