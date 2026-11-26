#![no_main]
//! Fuzz target 3 — the `AiChatRequest` parser (AI contracts — the ADR-009 minimum).
use libfuzzer_sys::fuzz_target;
use dropagentx_contracts::AiChatRequest;

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<AiChatRequest>(data);
});
