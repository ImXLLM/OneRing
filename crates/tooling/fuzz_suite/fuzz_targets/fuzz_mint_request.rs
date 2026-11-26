#![no_main]
//! Fuzz target 1 — the `MintRequest` parser — every panic is a boundary bug.
use libfuzzer_sys::fuzz_target;
use dropagentx_contracts::MintRequest;

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<MintRequest>(data);
});
