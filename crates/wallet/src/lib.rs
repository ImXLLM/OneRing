//! `kian-wallet` — four-chain USDT wallet — TRX/SOL/TON/BSC + treasury and key custody.
//!
//! Custody design for M5 — everything offline and deterministic:
//!  * No private key lives in this crate — address validation relies only on
//!    the chains' official checksums (Tron `Base58Check`, `EIP-55` with
//!    keccak-256, TON CRC16-XMODEM) — meaning "a wrong address is never recorded" and
//!    funds never go to a broken address.
//!  * Treasuries come only from env (server layer) — no treasury address in code.
//!  * Custody rule: every treasury movement must reference a parliament bill
//!    (`bill:…`) — amounts above the threshold also require a human vote (coupling with M5
//!    nation and Article 7 of the charter).
//!  * USDT amounts on the wire are decimal strings with ≤6 decimal places; storage is scaled
//!    in `micro-USDT` (10⁻⁶) — no floats on money. BNB Chain's on-chain unit has 18
//!    decimals; our wire layer normalizes user-declared amounts to ≤6 digits.
//!
//! Canonical USDT contracts (verified at step 5):
//!  * TRC20  — `TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t` (tron.network)
//!  * SPL    — `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` (Solscan)
//!  * TON    — `EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs` (Tonscan Jetton Master)
//!  * BEP20  — `0x55d398326f99059fF775485246999027B3197955` (canonical EIP-55; `PancakeSwap`)
#![forbid(unsafe_code)]

pub mod base_treasury;
pub mod ccxt_core;

use sha2::{Digest, Sha256};

/// Crate id — for smoke tests and logs.
pub const NAME: &str = "kian-wallet";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Health probe — wiring liveness indicator.
#[must_use]
pub fn hello() -> &'static str {
    "kian_wallet · KIAN is alive"
}

// ── errors ─────────────────────────────────────────────────────────────────────

/// Wallet error — HTTP mapping lives in the server layer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WalletError {
    /// Address does not parse under the chain rules (checksum/length/alphabet).
    #[error("invalid {chain} address: {detail}")]
    InvalidAddress {
        /// Error chain.
        chain: &'static str,
        /// Honest description of the problem.
        detail: String,
    },
    /// Invalid USDT amount (zero/negative/more than 6 decimals/out of range).
    #[error("invalid USDT amount: {0}")]
    InvalidAmount(String),
    /// Treasury custody rule violation — movement without a parliament bill.
    #[error("custody violation: {0}")]
    CustodyViolation(String),
}

/// Standard wallet result.
pub type WalletResult<T> = Result<T, WalletError>;

// ── chains ───────────────────────────────────────────────────────────────

/// Four supported chains — all USDT; the wire unit is uniformly micro-USDT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    /// Tron — TRC20
    Trx,
    /// Solana — SPL
    Sol,
    /// TON — Jetton
    Ton,
    /// BNB Smart Chain — BEP20
    Bsc,
}

/// Canonical USDT contract on Tron (TRC20).
pub const USDT_TRC20: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
/// Canonical USDT contract on Solana (SPL mint).
pub const USDT_SPL: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
/// Canonical USDT contract on TON (Jetton Master).
pub const USDT_TON: &str = "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs";
/// Canonical USDT contract on BNB Chain (BEP20, canonical EIP-55 form).
pub const USDT_BEP20: &str = "0x55d398326f99059fF775485246999027B3197955";

impl Chain {
    /// All four chains in a fixed order — for the metadata endpoint.
    pub const ALL: [Chain; 4] = [Chain::Trx, Chain::Sol, Chain::Ton, Chain::Bsc];

    /// Wire/database label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Trx => "TRX",
            Self::Sol => "SOL",
            Self::Ton => "TON",
            Self::Bsc => "BSC",
        }
    }

    /// Id parsed from the wire label — case-insensitive.
    #[must_use]
    pub fn from_str_raw(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "TRX" => Some(Self::Trx),
            "SOL" => Some(Self::Sol),
            "TON" => Some(Self::Ton),
            "BSC" => Some(Self::Bsc),
            _ => None,
        }
    }

    /// Persian name — mini-app/panel UI.
    #[must_use]
    pub const fn fa_name(&self) -> &'static str {
        match self {
            Self::Trx => "\u{62a}\u{631}\u{648}\u{646}",
            Self::Sol => "\u{633}\u{648}\u{644}\u{627}\u{646}\u{627}",
            Self::Ton => "\u{62a}\u{648}\u{646}",
            Self::Bsc => "\u{628}\u{627}\u{6cc}\u{646}\u{646}\u{633} \u{627}\u{633}\u{645}\u{627}\u{631}\u{62a} \u{686}\u{6cc}\u{646}",
        }
    }

    /// This chain's canonical USDT contract.
    #[must_use]
    pub const fn usdt_contract(&self) -> &'static str {
        match self {
            Self::Trx => USDT_TRC20,
            Self::Sol => USDT_SPL,
            Self::Ton => USDT_TON,
            Self::Bsc => USDT_BEP20,
        }
    }

    /// On-chain USDT unit — 6 on TRX/SOL/TON and 18 on BSC.
    /// The KIAN wire is independent of this, scaled in micro-USDT (10⁻⁶).
    #[must_use]
    pub const fn usdt_onchain_decimals(&self) -> u32 {
        match self {
            Self::Trx | Self::Sol | Self::Ton => 6,
            Self::Bsc => 18,
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── keccak-256 (for EIP-55) — pure implementation, no external dependency ──────────

const KECCAK_RC: [u64; 24] = [
    0x0000_0000_0000_0001,
    0x0000_0000_0000_8082,
    0x8000_0000_0000_808A,
    0x8000_0000_8000_8000,
    0x0000_0000_0000_808B,
    0x0000_0000_8000_0001,
    0x8000_0000_8000_8081,
    0x8000_0000_0000_8009,
    0x0000_0000_0000_008A,
    0x0000_0000_0000_0088,
    0x0000_0000_8000_8009,
    0x0000_0000_8000_000A,
    0x0000_0000_8000_808B,
    0x8000_0000_0000_008B,
    0x8000_0000_0000_8089,
    0x8000_0000_0000_8003,
    0x8000_0000_0000_8002,
    0x8000_0000_0000_0080,
    0x0000_0000_0000_800A,
    0x8000_0000_8000_000A,
    0x8000_0000_8000_8081,
    0x8000_0000_0000_8080,
    0x0000_0000_8000_0001,
    0x8000_0000_8000_8008,
];

const KECCAK_RHO: [u32; 24] = [
    1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
];

const KECCAK_PI: [usize; 24] = [
    10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
];

/// One full keccak-f[1600] round.
fn keccak_f(a: &mut [u64; 25]) {
    for &rc in &KECCAK_RC {
        // theta
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = a[x] ^ a[x + 5] ^ a[x + 10] ^ a[x + 15] ^ a[x + 20];
        }
        for x in 0..5 {
            let d = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            for y in 0..5 {
                a[x + 5 * y] ^= d;
            }
        }
        // rho + pi (rotation/offset with the reference implementation's bug fixed)
        let mut last = a[1];
        for i in 0..24 {
            let j = KECCAK_PI[i];
            let tmp = a[j];
            a[j] = last.rotate_left(KECCAK_RHO[i]);
            last = tmp;
        }
        // chi
        for y in 0..5 {
            let row = [
                a[5 * y],
                a[5 * y + 1],
                a[5 * y + 2],
                a[5 * y + 3],
                a[5 * y + 4],
            ];
            for x in 0..5 {
                a[5 * y + x] = row[x] ^ (!row[(x + 1) % 5] & row[(x + 2) % 5]);
            }
        }
        // iota
        a[0] ^= rc;
    }
}

/// Ethereum's original keccak-256 (0x01 padding — not SHA3-256 with 0x06).
#[must_use]
#[allow(clippy::cast_possible_truncation)] // Deliberately taking the lane's low byte — with mask 0xff
pub fn keccak256(input: &[u8]) -> [u8; 32] {
    const RATE: usize = 136;
    let mut st = [0u64; 25];
    let (blocks, remainder) = input.as_chunks::<RATE>();
    for block in blocks {
        absorb(&mut st, block);
        keccak_f(&mut st);
    }
    // Final block with keccak padding: 0x01 … 0x80
    let mut last = [0u8; RATE];
    last[..remainder.len()].copy_from_slice(remainder);
    last[remainder.len()] ^= 0x01;
    last[RATE - 1] ^= 0x80;
    absorb(&mut st, &last);
    keccak_f(&mut st);
    // 32-byte output — only the low byte of each lane (mask 0xFF);
    // Below 8 bits, truncation is never violated.
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        // Deliberate low byte of the lane — below 8 bits, truncation is never violated
        let lane_byte = ((st[i / 8] >> (8 * (i % 8))) & 0xff) as u8;
        *b = lane_byte;
    }
    out
}

/// XOR a 136-byte block into the state lanes (little-endian).
fn absorb(st: &mut [u64; 25], block: &[u8]) {
    for (i, lane) in block.as_chunks::<8>().0.iter().enumerate() {
        let mut v = 0u64;
        for (j, b) in lane.iter().enumerate() {
            v |= u64::from(*b) << (8 * j);
        }
        st[i] ^= v;
    }
}

// ── Base58 (Tron/Solana) ────────────────────────────────────────────────────

/// Bitcoin's Base58 alphabet (no 0/O/I/l) — Tron and Solana use the same.
const B58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Base58 decode to bytes — `None` on a character outside the alphabet.
#[must_use]
pub fn b58_decode(s: &str) -> Option<Vec<u8>> {
    if s.is_empty() {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::new();
    for c in s.bytes() {
        let val = u32::try_from(B58_ALPHABET.iter().position(|&a| a == c)?).ok()?;
        // bytes = bytes × 58 + val (bignum over a big-endian byte array)
        let mut carry = val;
        for b in bytes.iter_mut().rev() {
            let x = u32::from(*b) * 58 + carry;
            *b = (x & 0xff) as u8;
            carry = x >> 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    // Each leading '1' = one zero byte
    let pad = s.len() - s.trim_start_matches('1').len();
    let first_non_zero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    let mut out = vec![0u8; pad];
    out.extend_from_slice(&bytes[first_non_zero..]);
    Some(out)
}

/// CRC16-XMODEM (poly 0x1021, zero init) — TON user-friendly address checksum.
#[must_use]
pub fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &b in data {
        crc ^= u16::from(b) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

// ── Per-chain address validation ───────────────────────────────────────────────

/// Tron: `Base58Check` — 25 bytes = `0x41` + 20 bytes + sha256d[:4].
fn validate_trx(addr: &str) -> WalletResult<()> {
    let fail = |d: &str| WalletError::InvalidAddress {
        chain: "TRX",
        detail: d.to_owned(),
    };
    if addr.len() != 34 {
        return Err(fail("must be exactly 34 base58 chars"));
    }
    let decoded = b58_decode(addr).ok_or_else(|| fail("not valid base58"))?;
    if decoded.len() != 25 || decoded[0] != 0x41 {
        return Err(fail("decoded body must be 25 bytes with 0x41 prefix"));
    }
    let (body, chk) = decoded.split_at(21);
    let digest = Sha256::digest(Sha256::digest(body));
    if chk != &digest[..4] {
        return Err(fail("base58check checksum mismatch"));
    }
    Ok(())
}

/// Solana: plain Base58 — exactly 32 bytes (ed25519 public key), no checksum.
fn validate_sol(addr: &str) -> WalletResult<()> {
    let fail = |d: &str| WalletError::InvalidAddress {
        chain: "SOL",
        detail: d.to_owned(),
    };
    if !(32..=44).contains(&addr.len()) {
        return Err(fail("length must be 32..=44 base58 chars"));
    }
    let decoded = b58_decode(addr).ok_or_else(|| fail("not valid base58"))?;
    if decoded.len() != 32 {
        return Err(fail("decoded must be exactly 32 bytes"));
    }
    Ok(())
}

/// TON: user-friendly base64url — 36 bytes = tag + workchain + 32 bytes + CRC16-XMODEM
/// (big-endian; verified against the jUSDT master). tag: 0x11 bounceable /
/// 0x51 non-bounceable; workchain: 0 or -1.
fn validate_ton(addr: &str) -> WalletResult<()> {
    use base64::Engine as _;
    let fail = |d: &str| WalletError::InvalidAddress {
        chain: "TON",
        detail: d.to_owned(),
    };
    if addr.len() != 48 {
        return Err(fail("user-friendly form must be exactly 48 chars"));
    }
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let data = engine
        .decode(addr.as_bytes())
        .map_err(|_| fail("not valid base64url"))?;
    if data.len() != 36 {
        return Err(fail("decoded must be exactly 36 bytes"));
    }
    if data[0] != 0x11 && data[0] != 0x51 {
        return Err(fail(
            "tag must be 0x11 (bounceable) or 0x51 (non-bounceable)",
        ));
    }
    let wc = i8::from_be_bytes([data[1]]);
    if wc != 0 && wc != -1 {
        return Err(fail("workchain must be 0 or -1"));
    }
    let expected = crc16_xmodem(&data[..34]);
    let wire = u16::from_be_bytes([data[34], data[35]]);
    if expected != wire {
        return Err(fail("crc16-xmodem checksum mismatch"));
    }
    Ok(())
}

/// EIP-55 checksum over the hex form — input is 40 characters without 0x, all lowercase.
fn eip55_checksum(lower_hex: &[u8]) -> String {
    let hash = keccak256(lower_hex);
    let mut out = String::with_capacity(40);
    for (i, b) in lower_hex.iter().enumerate() {
        let nibble = if i % 2 == 0 {
            hash[i / 2] >> 4
        } else {
            hash[i / 2] & 0x0f
        };
        match char::from(*b) {
            c @ 'a'..='f' => {
                if nibble >= 8 {
                    out.push(c.to_ascii_uppercase());
                } else {
                    out.push(c);
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// BNB Chain: `0x` + 40 hex; all-lowercase/all-uppercase accepted without checksum, mixed only
/// if it reads as valid EIP-55 (the pure keccak-256 above).
fn validate_bsc(addr: &str) -> WalletResult<()> {
    let fail = |d: &str| WalletError::InvalidAddress {
        chain: "BSC",
        detail: d.to_owned(),
    };
    let body = addr.strip_prefix("0x").unwrap_or(addr);
    if body.len() != 40 || !body.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(fail("must be 0x + 40 hex chars"));
    }
    let lower = body.to_ascii_lowercase();
    let all_lower = body == lower;
    let all_upper = body == body.to_ascii_uppercase();
    if !all_lower && !all_upper {
        let canonical = eip55_checksum(lower.as_bytes());
        if body != canonical {
            return Err(fail("EIP-55 checksum mismatch"));
        }
    }
    Ok(())
}

/// Wallet address validation for one chain — the gateway for registering a user's address.
///
/// # Errors
/// [`WalletError::InvalidAddress`] with an honest reason.
pub fn validate_address(chain: Chain, addr: &str) -> WalletResult<()> {
    let addr = addr.trim();
    if addr.is_empty() || addr.len() > 96 {
        return Err(WalletError::InvalidAddress {
            chain: chain.as_str(),
            detail: "address must be 1..=96 chars".into(),
        });
    }
    match chain {
        Chain::Trx => validate_trx(addr),
        Chain::Sol => validate_sol(addr),
        Chain::Ton => validate_ton(addr),
        Chain::Bsc => validate_bsc(addr),
    }
}

// ── USDT amount ───────────────────────────────────────────────────────────────

/// Parse a USDT amount string — decimal with ≤6 decimal places and > 0; output in micro-USDT.
/// The same strict microTON contract (M3): no floats, no negatives, no zero.
///
/// # Errors
/// [`WalletError::InvalidAmount`].
pub fn parse_usdt(amount: &str) -> WalletResult<i64> {
    let fail = |d: &str| WalletError::InvalidAmount(d.to_owned());
    let (whole, frac) = amount.split_once('.').map_or((amount, ""), |(w, f)| (w, f));
    if whole.is_empty() && frac.is_empty() {
        return Err(fail("empty amount"));
    }
    if whole.chars().any(|c| !c.is_ascii_digit()) || frac.chars().any(|c| !c.is_ascii_digit()) {
        return Err(fail("not a decimal string"));
    }
    if frac.len() > 6 {
        return Err(fail("at most 6 decimal places"));
    }
    let whole_val: u64 = if whole.is_empty() {
        0
    } else {
        whole.parse().map_err(|_| fail("too large"))?
    };
    let mut micro: u128 = u128::from(whole_val) * 1_000_000;
    if !frac.is_empty() {
        let padded = format!("{frac:0<6}");
        micro += padded.parse::<u128>().map_err(|_| fail("too large"))?;
    }
    if micro == 0 {
        return Err(fail("zero amount is meaningless"));
    }
    if micro > i64::MAX as u128 {
        return Err(fail("out of range"));
    }
    #[allow(clippy::cast_possible_truncation)] // Checked against i64 overflow
    Ok(micro as i64)
}

// ── Treasury custody ─────────────────────────────────────────────────────────────

/// Treasury custody policy — the M5 contractual rule:
/// every treasury movement must carry a parliament bill; amounts ≥ the threshold require
/// a human vote (the same Article 7 gate, this time in micro-USDT scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustodyPolicy {
    /// Human-vote threshold in micro-USDT — 0 means no amount ever requires a human vote.
    pub human_threshold_micro: i64,
}

impl Default for CustodyPolicy {
    /// Default: every amount ≥ 50 USDT equals the human threshold (wire rate: 1000 Đ/USDT
    /// ⇔ charter threshold 50,000 Đ) — the real rate comes from env in the server layer.
    fn default() -> Self {
        Self {
            human_threshold_micro: 50 * 1_000_000,
        }
    }
}

impl CustodyPolicy {
    /// Review a treasury movement plan — output: does it require a human vote?
    ///
    /// # Errors
    /// [`WalletError::CustodyViolation`] when there is no valid bill reference (`bill:…`).
    pub fn check_movement(&self, amount_micro: i64, bill_id: Option<&str>) -> WalletResult<bool> {
        if amount_micro <= 0 {
            return Err(WalletError::CustodyViolation(
                "amount must be positive".into(),
            ));
        }
        let bill = bill_id.ok_or_else(|| {
            WalletError::CustodyViolation(
                "treasury movement requires a parliament bill reference".into(),
            )
        })?;
        let bill = bill.trim();
        let ok_ref = bill.starts_with("bill:") && bill.len() > 5 && bill.len() <= 64;
        if !ok_ref {
            return Err(WalletError::CustodyViolation(
                "bill reference must look like bill:<id>".into(),
            ));
        }
        let needs_human =
            self.human_threshold_micro > 0 && amount_micro >= self.human_threshold_micro;
        Ok(needs_human)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_smoke() {
        assert_eq!(NAME, "kian-wallet");
        assert!(hello().contains("alive"));
    }

    #[test]
    fn keccak_matches_reference_vectors() {
        // Reference vectors (pycryptodome keccak-256 — step 5 sandbox)
        let h = |b: &[u8]| hex::encode(keccak256(b));
        assert_eq!(
            h(b""),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
        assert_eq!(
            h(b"abc"),
            "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
        );
        // Multi-block — 200 bytes > the 136 rate (covers the chunks_exact path)
        assert_eq!(
            h(&[b'a'; 200]),
            "96ea54061def936c4be90b518992fdc6f12f535068a256229aca54267b4d084d"
        );
    }

    #[test]
    fn eip55_checksum_matches_spec_vectors() {
        // The four official EIP-55 vectors
        for v in [
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
        ] {
            assert!(validate_bsc(v).is_ok(), "{v} must pass");
        }
        // Corrupt one checksum letter — must be rejected
        assert!(validate_bsc("0x5AAEB6053F3E94C9b9A09f33669435E7Ef1BeAed").is_err());
    }

    #[test]
    fn canonical_usdt_contracts_validate_on_their_own_chain() {
        // Real evidence: each chain's canonical contract must parse under that chain's rules
        assert!(validate_address(Chain::Trx, USDT_TRC20).is_ok());
        assert!(validate_address(Chain::Sol, USDT_SPL).is_ok());
        assert!(validate_address(Chain::Ton, USDT_TON).is_ok());
        assert!(validate_address(Chain::Bsc, USDT_BEP20).is_ok());
    }

    #[test]
    fn trx_rejects_tampered_checksum() {
        // The same address with one character swapped — the checksum breaks
        assert!(validate_trx("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6u").is_err());
        assert!(
            validate_trx("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6").is_err(),
            "\u{637}\u{648}\u{644} \u{6f3}\u{6f3}"
        );
        assert!(validate_trx("1111111111111111111111111111111111").is_err());
    }

    #[test]
    fn ton_rejects_tampered_crc_and_bad_tag() {
        let good = USDT_TON;
        assert!(validate_ton(good).is_ok());
        // Corrupt CRC: the last character is changed (the original ends with 's')
        let bad = format!("{}t", &good[..47]);
        assert!(validate_ton(&bad).is_err());
        // Bad length
        assert!(validate_ton(&good[..47]).is_err());
        // Standard base64 with +/ — a character outside the url-safe alphabet
        assert!(validate_ton("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id+sDs").is_err());
    }

    #[test]
    fn sol_rejects_wrong_length_and_junk() {
        assert!(validate_sol(USDT_SPL).is_ok());
        // Character outside the alphabet (even with the right length)
        assert!(
            validate_sol("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8Benw0YB").is_err(),
            "\u{6a9}\u{627}\u{631}\u{627}\u{6a9}\u{62a}\u{631} '0' \u{628}\u{6cc}\u{631}\u{648}\u{646} \u{627}\u{644}\u{641}\u{628}\u{627}"
        );
        assert!(validate_sol("0OIl").is_err(), "\u{6a9}\u{627}\u{631}\u{627}\u{6a9}\u{62a}\u{631} \u{628}\u{6cc}\u{631}\u{648}\u{646} \u{627}\u{644}\u{641}\u{628}\u{627}");
        // 45 characters → 33 bytes — not 32 (a leading '1' prefix brings a zero byte)
        let too_long = format!("1{USDT_SPL}");
        assert!(validate_sol(&too_long).is_err());
        // 35 characters → one byte — not 32
        let too_short = "1111111111111111111111111111111112";
        assert!(validate_sol(too_short).is_err());
    }

    #[test]
    fn bsc_rejects_bad_case_and_junk() {
        assert!(validate_bsc(USDT_BEP20).is_ok(), "\u{641}\u{631}\u{645} \u{6a9}\u{627}\u{646}\u{648}\u{646}\u{6cc}\u{6a9} \u{628}\u{627}\u{6cc}\u{62f} \u{628}\u{62e}\u{648}\u{627}\u{646}\u{62f}");
        assert!(
            validate_bsc("55d398326f99059ff775485246999027b3197955").is_ok(),
            "\u{647}\u{645}\u{647}‌\u{6a9}\u{648}\u{686}\u{6a9} \u{628}\u{62f}\u{648}\u{646} 0x"
        );
        assert!(
            validate_bsc("0x55D398326F99059FF775485246999027B3197955").is_ok(),
            "\u{647}\u{645}\u{647}‌\u{628}\u{632}\u{631}\u{6af} \u{645}\u{62c}\u{627}\u{632}"
        );
        // One letter differs from the canonical form — the checksum breaks
        assert!(validate_bsc("0x55D398326f99059fF775485246999027B3197955").is_err());
        assert!(validate_bsc("0x123").is_err());
        assert!(validate_bsc("0xzzd398326f99059ff775485246999027b3197955").is_err());
    }

    #[test]
    fn chain_metadata_is_honest() {
        assert_eq!(Chain::from_str_raw("trx"), Some(Chain::Trx));
        assert_eq!(Chain::from_str_raw("BSC"), Some(Chain::Bsc));
        assert_eq!(Chain::from_str_raw("DOGE"), None);
        assert_eq!(
            Chain::Bsc.usdt_onchain_decimals(),
            18,
            "BSC \u{631}\u{648}\u{6cc} \u{632}\u{646}\u{62c}\u{6cc}\u{631}\u{647} \u{6f1}\u{6f8} \u{627}\u{639}\u{634}\u{627}\u{631} \u{62f}\u{627}\u{631}\u{62f}"
        );
        for c in Chain::ALL {
            assert_ne!(c.usdt_contract(), "", "contract must be present");
            assert_eq!(c.to_string(), c.as_str());
        }
    }

    #[test]
    fn usdt_amount_parsing_is_strict() {
        assert_eq!(parse_usdt("1").unwrap(), 1_000_000);
        assert_eq!(parse_usdt("0.5").unwrap(), 500_000);
        assert_eq!(parse_usdt("1.234567").unwrap(), 1_234_567);
        assert!(parse_usdt("0.1234567").is_err(), "\u{628}\u{6cc}\u{634} \u{627}\u{632} \u{6f6} \u{631}\u{642}\u{645} \u{627}\u{639}\u{634}\u{627}\u{631}");
        assert!(parse_usdt("0").is_err());
        assert!(parse_usdt("-1").is_err());
        assert!(parse_usdt("").is_err());
        assert!(parse_usdt("1e3").is_err());
        assert!(parse_usdt("abc").is_err());
    }

    #[test]
    fn custody_requires_bill_reference_always() {
        let policy = CustodyPolicy::default();
        // No bill — absolutely forbidden
        assert!(policy.check_movement(5_000_000, None).is_err());
        // Malformed bill
        assert!(policy.check_movement(5_000_000, Some("esc:9")).is_err());
        assert!(policy.check_movement(5_000_000, Some("bill:")).is_err());
        // Valid bill — below the threshold: no human vote required
        assert!(
            !policy.check_movement(5_000_000, Some("bill:x1")).unwrap(),
            "\u{632}\u{6cc}\u{631} \u{622}\u{633}\u{62a}\u{627}\u{646}\u{647} — \u{628}\u{62f}\u{648}\u{646} \u{631}\u{623}\u{6cc} \u{627}\u{646}\u{633}\u{627}\u{646}\u{6cc}"
        );
        // Above the threshold (50 USDT): a human vote is required
        assert!(
            policy.check_movement(60_000_000, Some("bill:x1")).unwrap(),
            "\u{628}\u{627}\u{644}\u{627}\u{6cc} \u{622}\u{633}\u{62a}\u{627}\u{646}\u{647} — \u{631}\u{623}\u{6cc} \u{627}\u{646}\u{633}\u{627}\u{646}\u{6cc} \u{644}\u{627}\u{632}\u{645} \u{627}\u{633}\u{62a}"
        );
        // Zero/negative amount
        assert!(policy.check_movement(0, Some("bill:x1")).is_err());
        assert!(policy.check_movement(-1, Some("bill:x1")).is_err());
    }

    #[test]
    fn b58_decode_handles_leading_zeros() {
        // '11' → two zero bytes
        assert_eq!(b58_decode("11").unwrap(), vec![0u8, 0]);
        assert_eq!(b58_decode("2").unwrap(), vec![1]);
        assert!(b58_decode("").is_none(), "empty string is not base58");
        assert!(b58_decode("0OIl").is_none());
    }

    #[test]
    fn crc16_xmodem_matches_reference() {
        // Reference vector: CRC16-XMODEM("123456789") = 0x31C3
        assert_eq!(crc16_xmodem(b"123456789"), 0x31C3);
    }
}
