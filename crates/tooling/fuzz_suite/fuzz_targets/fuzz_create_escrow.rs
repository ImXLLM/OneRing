#![no_main]
//! Fuzz target 2 — the `CreateEscrowRequest` parser.
use libfuzzer_sys::fuzz_target;
use dropagentx_contracts::CreateEscrowRequest;

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<CreateEscrowRequest>(data);
});
