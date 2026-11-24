# 🖥️ `04_DNAharness.md` — EXECUTION SANDBOXES & HARDWARE PHYSICS MATRIX
> **Connected To:** `02_DNAwork.md` · `06_LINK_Router.md` · `07_LINK_Protocol.md`

---

## 1. THE 3 EXECUTION NODES & THEIR PHYSICAL LAWS

| Node ID | Environment | CPU / RAM / Disk Physics | Network & Tooling Physics | Allowed Operations ✅ | Forbidden Operations ❌ |
|---|---|---|---|---|---|
| **`NODE-A`** | **Arena Sandbox** (`Map / Harvester`) | `1.98 GiB` RAM, **`0 B` Swap** (hard OOM-killer), `/tmp` & `/home/user` | Ultra-fast GitHub/API egress (`>100 MB/s`), `git`, `python3`, `rustc`, `tsc`, `zip` | `git clone --depth 1 --filter=blob:none --sparse`, code archaeology, standalone `rustc --test` (0 external crates), SQLite injectors, `.zip` vault packaging | **NEVER** run `cargo test --workspace` or `npm install` of heavy monorepos (triggers instant OOM crash) |
| **`NODE-Z`** | **Z.ai Sandbox** (`Reduce / Kernel Builder`) | Multi-core Linux container, persistent `/home/z/my-project/`, `/tmp` target dir | Pre-installed Rust toolchain, Next.js 16, SQLite3; uploads in `/home/z/my-project/upload/`, downloads in `/home/z/my-project/download/` | Full crate compilation (`CARGO_TARGET_DIR=/tmp/my-project/target CARGO_BUILD_JOBS=2`), `cargo clippy`, SQLite production seeding, git commits, release zipping | **NEVER** one-shot 10 unverified zips at once; **NEVER** leave background cron tasks (`post_reset_proof.sh`) running during kernel builds |
| **`NODE-L`** | **Local Workstation** (`Asus TUF FX505` + `S23 FE`) | Persistent local storage, full GPU/audio (`Yesido`/`Lavalier`), Trust Wallet (`$33`) | Human-in-the-loop commander bridge, `.zip` relay between `NODE-A` and `NODE-Z`, Railway deployment trigger | Storing checkpoint `.zip` & `.db` backups, maintaining `KIA-BRAIN-OS` 9-file genome, final production launch | Never expose private wallet seed phrases to any cloud sandbox |