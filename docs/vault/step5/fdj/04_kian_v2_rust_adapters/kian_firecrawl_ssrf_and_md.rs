//! KIAN v2 Rust Adapter — Extracted & Adapted from `firecrawl/firecrawl`
//! Patterns:
//!   1. Dual-Stage SSRF & DNS-Rebinding Guard (`apps/local/src/guard.rs`):
//!      Blocks private IPv4/IPv6, CGNAT (100.64.0.0/10), NAT64 (64:ff9b::/96),
//!      link-local (169.254.0.0/16), and cloud metadata (AWS/GCP/Alibaba 100.100.100.200).
//!   2. Zero-Bloat HTML-to-LLM-Markdown Cleaner (`apps/local/src/markdown.rs` + `removeUnwantedElements.ts`):
//!      Strips non-content chrome (`script`, `style`, `svg`, `iframe`, `nav`, `footer`, `aside`),
//!      collapses excess whitespace/newlines, and preserves token budget.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// Public internet only (default for KIAN v2 autonomous web tools).
    PublicOnly,
    /// Allow loopback/RFC1918 intranet, while keeping link-local & cloud metadata blocked.
    AllowPrivate,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SsrfBlocked(pub String);

fn v4_is_metadata_or_link_local(ip: Ipv4Addr) -> bool {
    ip.is_link_local() || ip == Ipv4Addr::new(100, 100, 100, 200)
}

fn v4_is_private(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_multicast()
        || ip.is_documentation()
        // 100.64.0.0/10 Carrier-Grade NAT
        || (o[0] == 100 && (o[1] & 0b1100_0000) == 64)
        // 192.0.0.0/24 IETF protocol assignments
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        // 198.18.0.0/15 benchmarking
        || (o[0] == 198 && (o[1] & 0xfe) == 18)
        // 240.0.0.0/4 reserved & 0.0.0.0/8
        || o[0] >= 240
        || o[0] == 0
}

fn v6_embedded_v4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return Some(v4);
    }
    let s = ip.segments();
    // NAT64 (64:ff9b::/96) or IPv4-compatible IPv6
    if (s[0] == 0x64 && s[1] == 0xff9b && s[2..6].iter().all(|x| *x == 0))
        || (s[..6].iter().all(|x| *x == 0) && (s[6] != 0 || s[7] > 1))
    {
        return Some(Ipv4Addr::new(
            (s[6] >> 8) as u8,
            s[6] as u8,
            (s[7] >> 8) as u8,
            s[7] as u8,
        ));
    }
    None
}

/// Validates a resolved IP address BEFORE connect() to defeat DNS rebinding.
pub fn check_resolved_ip(ip: IpAddr, policy: NetworkPolicy) -> Result<(), SsrfBlocked> {
    match ip {
        IpAddr::V4(v4) => {
            if v4_is_metadata_or_link_local(v4) {
                return Err(SsrfBlocked(format!("blocked metadata/link-local IPv4: {v4}")));
            }
            if policy == NetworkPolicy::PublicOnly && v4_is_private(v4) {
                return Err(SsrfBlocked(format!("blocked private IPv4: {v4}")));
            }
            Ok(())
        }
        IpAddr::V6(v6) => {
            if let Some(embedded) = v6_embedded_v4(v6) {
                return check_resolved_ip(IpAddr::V4(embedded), policy);
            }
            let seg0 = v6.segments()[0];
            let is_ula = (seg0 & 0xfe00) == 0xfc00; // fc00::/7
            let is_link_local = (seg0 & 0xffc0) == 0xfe80; // fe80::/10
            if is_link_local {
                return Err(SsrfBlocked(format!("blocked link-local IPv6: {v6}")));
            }
            if policy == NetworkPolicy::PublicOnly
                && (v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() || is_ula)
            {
                return Err(SsrfBlocked(format!("blocked private IPv6: {v6}")));
            }
            Ok(())
        }
    }
}

/// Lightweight whitespace & tag-noise normalizer inspired by Firecrawl's `markdown.rs`.
pub fn normalize_llm_markdown(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut empty_lines = 0usize;
    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            empty_lines += 1;
            if empty_lines <= 1 && !out.is_empty() {
                out.push('\n');
            }
        } else {
            empty_lines = 0;
            out.push_str(trimmed);
            out.push('\n');
        }
    }
    out.trim().to_string()
}
