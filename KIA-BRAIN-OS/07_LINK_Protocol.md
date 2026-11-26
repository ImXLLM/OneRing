---
file_id: "07_LINK_Protocol"
schema_version: "1.0.0"
layer: protocol
envelope: "InterAgentHandoff"
language: "fa-IR + en-US"
---

# 07 — LINK_Protocol: The Inter-Agent Handoff Envelope

## 1. The Envelope Schema (compatible with `HandoffPacket` in `omni_genome_router.rs`)

```json
{
  "handoff_id": "ho_<utc_iso>_<short_hash>",
  "from_model": "kian-guardian|<model_id>",
  "to_model": "<model_id>",
  "base_sha": "<git sha the handoff was sealed on>",
  "task": "<one-sentence description>",
  "invariants": ["unbreakable number/law ..."],
  "files_touched": ["crates/..."],
  "failed_hypotheses_do_not_repeat": ["hypothesis rejected in the previous generation ..."],
  "verification_proof": ["cargo test -p ... → 445 green", "clippy -D warnings → 0"],
  "target_contract": "<expected output from the handoff receiver>"
}
```

## 2. Hard Rules

1. **Without `base_sha` the handoff is invalid** — the receiver must be able to check out the same tree.
2. **`invariants` are minimal and numeric:** each invariant is one testable sentence ("KbEntry=62",
   "trialBalanceSum=0"), not a vague wish.
3. **Rejected hypotheses are never repeated** — the field is mandatory even when empty.
4. **Executable proof:** a verification claim without command output = a false packet → rejected.
5. The packet moves only between the two ends of the DNA chain (`KIA-BRAIN-OS/07_LINK_Protocol.md`
   = the schema of truth; the genome's 09 files express the same schema in the models' language).

## 3. The Acceptance Cycle

Receive → check `base_sha` → check the invariants against the real tree → re-run the verification
→ accept or return with a reason. Returning without a reason = a protocol violation.
