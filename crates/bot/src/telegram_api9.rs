//! `telegram_api9` — آداپتورهای Bot API 9 مینی‌اپ تلگرام (پورت گنجینهٔ tdesktop).
//!
//! پورت از `tdesktop-telegram-gems/02_kian_v2_adapters/kian_telegram_bot_api9.rs`
//! (ممیزی `telegramdesktop/tdesktop` + الگوی رسمی Bot API ۹):
//!
//! * [`validate_init_data`] — اعتبارسنجی `Telegram.WebApp.initData` با HMAC
//!   بات-توکن: رشتهٔ چکِ `key=value` مرتب با LF، **فقط `hash` حذف می‌شود** و
//!   فیلد `signature` امروزی در HMAC می‌ماند؛ کلید مشتق
//!   `HMAC-SHA256("WebAppData", bot_token)` و مقایسهٔ زمان-ثابت.
//!   این تجزیه‌گر برای اعتبارسنجی امضای Ed25519 ثالث نیست.
//! * [`StartAppPayload`] — پارامتر شروع وایرال دقیقاً `p_<productId>_r_<refTgId>`.
//! * [`StarsInvoicePayload`] — بار فاکتور Stars با ارز `XTR` (سقف ۱۲۸ بایت،
//!   کلید جست‌وجوی DB — نه توکن احراز).
//!
//! وابستگی صفر: SHA-256/HMAC به‌صورت خالص پیاده شده (بدون openssl-system).

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

const MAX_INIT_DATA_BYTES: usize = 16 * 1024;
const MAX_INIT_DATA_FIELDS: usize = 128;
const MAX_STARS_INVOICE_PAYLOAD_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitDataError {
    EmptyInput,
    InputTooLarge,
    TooManyFields,
    EmptyParameter,
    InvalidKey,
    InvalidPercentEncoding,
    InvalidUtf8,
    DuplicateParameter,
    EmptyBotToken,
    MissingHash,
    InvalidHash,
    InvalidSignature,
    MissingAuthDate,
    InvalidAuthDate,
    FutureAuthDate,
    Expired,
    InvalidFreshnessPolicy,
}

impl fmt::Display for InitDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for InitDataError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInitData {
    /// Decoded fields, including `hash` and `signature` when supplied.
    pub fields: BTreeMap<String, String>,
    pub auth_date: i64,
    pub user_json: Option<String>,
    pub query_id: Option<String>,
    pub start_param: Option<String>,
}

/// Validates raw Telegram.WebApp.initData using the bot-token HMAC scheme.
///
/// `now_unix` and `max_age_secs` are explicit so callers can use a trusted clock
/// and deterministic policy. `signature` is included in the HMAC check string;
/// only `hash` is excluded. The returned user JSON must only be parsed/used after
/// this function succeeds. Keep `bot_token` on the server.
///
/// # Errors
/// Returns [`InitDataError`] on any structural, size, HMAC, freshness or
/// encoding failure — the exact variant names the failure reason (fail-closed).
pub fn validate_init_data(
    raw: &str,
    bot_token: &str,
    now_unix: i64,
    max_age_secs: i64,
) -> Result<ValidatedInitData, InitDataError> {
    if raw.is_empty() {
        return Err(InitDataError::EmptyInput);
    }
    if raw.len() > MAX_INIT_DATA_BYTES {
        return Err(InitDataError::InputTooLarge);
    }
    if bot_token.is_empty() {
        return Err(InitDataError::EmptyBotToken);
    }
    if now_unix < 0 || max_age_secs < 0 {
        return Err(InitDataError::InvalidFreshnessPolicy);
    }

    let fields = parse_query(raw)?;
    let supplied_hash = fields.get("hash").ok_or(InitDataError::MissingHash)?;
    let supplied_hash = decode_hash(supplied_hash).ok_or(InitDataError::InvalidHash)?;

    let mut check_string = String::new();
    for (key, value) in &fields {
        if key == "hash" {
            continue;
        }
        if !check_string.is_empty() {
            check_string.push('\n');
        }
        check_string.push_str(key);
        check_string.push('=');
        check_string.push_str(value);
    }

    // Telegram Mini Apps bot-token HMAC: key="WebAppData", message=bot token.
    let secret_key = hmac_sha256(b"WebAppData", bot_token.as_bytes());
    let expected_hash = hmac_sha256(&secret_key, check_string.as_bytes());
    if !constant_time_eq(&expected_hash, &supplied_hash) {
        return Err(InitDataError::InvalidSignature);
    }

    let auth_date = fields
        .get("auth_date")
        .ok_or(InitDataError::MissingAuthDate)?
        .parse::<i64>()
        .map_err(|_| InitDataError::InvalidAuthDate)?;
    if auth_date <= 0 {
        return Err(InitDataError::InvalidAuthDate);
    }
    if auth_date > now_unix.saturating_add(60) {
        return Err(InitDataError::FutureAuthDate);
    }
    if now_unix.saturating_sub(auth_date) > max_age_secs {
        return Err(InitDataError::Expired);
    }

    Ok(ValidatedInitData {
        user_json: fields.get("user").cloned(),
        query_id: fields.get("query_id").cloned(),
        start_param: fields.get("start_param").cloned(),
        auth_date,
        fields,
    })
}

fn parse_query(raw: &str) -> Result<BTreeMap<String, String>, InitDataError> {
    let raw = raw.strip_prefix('?').unwrap_or(raw);
    if raw.is_empty() {
        return Err(InitDataError::EmptyInput);
    }

    let mut fields = BTreeMap::new();
    for pair in raw.split('&') {
        if pair.is_empty() {
            return Err(InitDataError::EmptyParameter);
        }
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = form_url_decode(raw_key)?;
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(InitDataError::InvalidKey);
        }
        let value = form_url_decode(raw_value)?;
        if fields.contains_key(&key) {
            return Err(InitDataError::DuplicateParameter);
        }
        fields.insert(key, value);
        if fields.len() > MAX_INIT_DATA_FIELDS {
            return Err(InitDataError::TooManyFields);
        }
    }
    Ok(fields)
}

fn form_url_decode(input: &str) -> Result<String, InitDataError> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                decoded.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(InitDataError::InvalidPercentEncoding);
                }
                let hi = hex_nibble(bytes[i + 1]).ok_or(InitDataError::InvalidPercentEncoding)?;
                let lo = hex_nibble(bytes[i + 2]).ok_or(InitDataError::InvalidPercentEncoding)?;
                decoded.push((hi << 4) | lo);
                i += 3;
            }
            byte => {
                decoded.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|_| InitDataError::InvalidUtf8)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_hash(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let bytes = value.as_bytes();
    let mut result = [0u8; 32];
    for i in 0..32 {
        let hi = hex_nibble(bytes[i * 2])?;
        let lo = hex_nibble(bytes[i * 2 + 1])?;
        result[i] = (hi << 4) | lo;
    }
    Some(result)
}

/// هگز امن (فقط در تست‌ها برای امضا و بردارهای مرجع به کار می‌رود).
#[cfg(test)]
fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Constant-time for equal-length hashes; both values are public-format SHA-256
/// digests, and malformed lengths are rejected before this comparison.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for i in 0..left.len() {
        difference |= left[i] ^ right[i];
    }
    difference == 0
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > key_block.len() {
        key_block[..32].copy_from_slice(&sha256(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut inner = Vec::with_capacity(64 + message.len());
    let mut outer = Vec::with_capacity(64 + 32);
    for byte in key_block {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    outer.extend_from_slice(&sha256(&inner));
    sha256(&outer)
}

// پورت ۱:۱ الگوریتم مرجع SHA-256 (FIPS 180-4) — نام‌های تک‌حرفی a..h و
// ثابت‌های K استانداردِ بدون جداکننده، فرمول pad و chunks_exact هم همان
// فرم مرجع‌اند.
#[allow(
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::manual_div_ceil,
    clippy::chunks_exact_to_as_chunks
)]
fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let padded_len = ((input.len() + 9 + 63) / 64) * 64;
    let mut padded = Vec::with_capacity(padded_len);
    padded.extend_from_slice(input);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = [
        0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    for block in padded.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (i, chunk) in block.chunks_exact(4).take(16).enumerate() {
            words[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for i in 16..64 {
            let s0 = words[i - 15].rotate_right(7)
                ^ words[i - 15].rotate_right(18)
                ^ (words[i - 15] >> 3);
            let s1 = words[i - 2].rotate_right(17)
                ^ words[i - 2].rotate_right(19)
                ^ (words[i - 2] >> 10);
            words[i] = words[i - 16]
                .wrapping_add(s0)
                .wrapping_add(words[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sum1)
                .wrapping_add(choose)
                .wrapping_add(K[i])
                .wrapping_add(words[i]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut digest = [0u8; 32];
    for (chunk, word) in digest.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartAppPayload {
    pub product_id: u64,
    pub ref_tg_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartAppPayloadError {
    InvalidFormat,
    InvalidProductId,
    InvalidReferrerId,
}

impl StartAppPayload {
    /// Parses exactly `p_<positive productId>_r_<positive refTgId>`.
    ///
    /// # Errors
    /// [`StartAppPayloadError`] — invalid format, zero or non-numeric product
    /// id / referrer id.
    pub fn parse(input: &str) -> Result<Self, StartAppPayloadError> {
        let rest = input
            .strip_prefix("p_")
            .ok_or(StartAppPayloadError::InvalidFormat)?;
        let (product, referrer) = rest
            .split_once("_r_")
            .ok_or(StartAppPayloadError::InvalidFormat)?;
        let product_id = product
            .parse::<u64>()
            .map_err(|_| StartAppPayloadError::InvalidProductId)?;
        let ref_tg_id = referrer
            .parse::<u64>()
            .map_err(|_| StartAppPayloadError::InvalidReferrerId)?;
        if product_id == 0 {
            return Err(StartAppPayloadError::InvalidProductId);
        }
        if ref_tg_id == 0 {
            return Err(StartAppPayloadError::InvalidReferrerId);
        }
        Ok(Self { product_id, ref_tg_id })
    }

    #[must_use]
    pub fn to_start_param(self) -> String {
        format!("p_{}_r_{}", self.product_id, self.ref_tg_id)
    }
}

impl FromStr for StartAppPayload {
    type Err = StartAppPayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarsInvoicePayload {
    /// Server-generated opaque order ID; fulfillment must still look up the order in DB.
    pub order_id: String,
    pub product_id: u64,
    pub buyer_tg_id: u64,
    pub ref_tg_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarsPayloadError {
    InvalidFormat,
    InvalidOrderId,
    InvalidProductId,
    InvalidBuyerId,
    InvalidReferrerId,
    TooLong,
}

impl StarsInvoicePayload {
    pub const CURRENCY: &'static str = "XTR";

    /// Compact server-generated Bot API invoice payload (max 128 UTF-8 bytes).
    /// It is a DB lookup key, not a signed authorization token.
    ///
    /// # Errors
    /// [`StarsPayloadError`] — invalid/oversized order id, zero ids or payload
    /// beyond 128 bytes.
    pub fn encode(&self) -> Result<String, StarsPayloadError> {
        if self.order_id.is_empty()
            || self.order_id.len() > 64
            || !self
                .order_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(StarsPayloadError::InvalidOrderId);
        }
        if self.product_id == 0 {
            return Err(StarsPayloadError::InvalidProductId);
        }
        if self.buyer_tg_id == 0 {
            return Err(StarsPayloadError::InvalidBuyerId);
        }
        if self.ref_tg_id == Some(0) {
            return Err(StarsPayloadError::InvalidReferrerId);
        }
        let referrer = self
            .ref_tg_id
            .map_or_else(|| "-".to_owned(), |id| id.to_string());
        let payload = format!(
            "kian1|{}|{}|{}|{}",
            self.order_id, self.product_id, self.buyer_tg_id, referrer
        );
        if payload.len() > MAX_STARS_INVOICE_PAYLOAD_BYTES {
            return Err(StarsPayloadError::TooLong);
        }
        Ok(payload)
    }

    /// Parses a payload produced by [`Self::encode`] and re-checks every
    /// canonical rule.
    ///
    /// # Errors
    /// [`StarsPayloadError`] — wrong prefix, non-numeric fields, zero ids,
    /// oversized payload or trailing segments.
    pub fn parse(payload: &str) -> Result<Self, StarsPayloadError> {
        if payload.len() > MAX_STARS_INVOICE_PAYLOAD_BYTES {
            return Err(StarsPayloadError::TooLong);
        }
        let mut parts = payload.split('|');
        if parts.next() != Some("kian1") {
            return Err(StarsPayloadError::InvalidFormat);
        }
        let order_id = parts.next().ok_or(StarsPayloadError::InvalidFormat)?.to_string();
        let product = parts.next().ok_or(StarsPayloadError::InvalidFormat)?;
        let buyer = parts.next().ok_or(StarsPayloadError::InvalidFormat)?;
        let referrer = parts.next().ok_or(StarsPayloadError::InvalidFormat)?;
        if parts.next().is_some() {
            return Err(StarsPayloadError::InvalidFormat);
        }
        let value = Self {
            order_id,
            product_id: product
                .parse::<u64>()
                .map_err(|_| StarsPayloadError::InvalidProductId)?,
            buyer_tg_id: buyer
                .parse::<u64>()
                .map_err(|_| StarsPayloadError::InvalidBuyerId)?,
            ref_tg_id: if referrer == "-" {
                None
            } else {
                Some(
                    referrer
                        .parse::<u64>()
                        .map_err(|_| StarsPayloadError::InvalidReferrerId)?,
                )
            },
        };
        // Reuse all canonical checks, including ID and order token validation.
        value.encode()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign_fields(token: &str, fields: &[(&str, &str)]) -> String {
        let mut pairs: Vec<(String, String)> = fields
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let check = pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n");
        let secret = hmac_sha256(b"WebAppData", token.as_bytes());
        let hash = encode_hex(&hmac_sha256(&secret, check.as_bytes()));
        let query = fields
            .iter()
            .map(|(k, v)| format!("{k}={}", percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{query}&hash={hash}")
    }

    fn percent_encode(value: &str) -> String {
        let mut out = String::new();
        for byte in value.bytes() {
            if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
                out.push(byte as char);
            } else {
                out.push('%');
                out.push_str(&format!("{:02X}", byte));
            }
        }
        out
    }

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            encode_hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hmac_sha256_rfc4231_case_1() {
        assert_eq!(
            encode_hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn validates_init_data_and_keeps_signature_in_hmac_string() {
        let token = "123456:unit-test-token";
        let raw = sign_fields(
            token,
            &[
                ("user", r#"{"id":42,"first_name":"Kian + User"}"#),
                ("signature", "ed25519-test-signature"),
                ("start_param", "p_17_r_9001"),
                ("auth_date", "1700000000"),
                ("query_id", "query-abc"),
            ],
        );
        let verified = validate_init_data(&raw, token, 1_700_000_030, 60).unwrap();
        assert_eq!(verified.auth_date, 1_700_000_000);
        assert_eq!(verified.query_id.as_deref(), Some("query-abc"));
        assert_eq!(verified.start_param.as_deref(), Some("p_17_r_9001"));
        assert_eq!(
            verified.user_json.as_deref(),
            Some(r#"{"id":42,"first_name":"Kian + User"}"#)
        );
        assert_eq!(verified.fields.get("signature").map(String::as_str), Some("ed25519-test-signature"));
    }

    #[test]
    fn changing_signature_invalidates_hmac() {
        let token = "123456:unit-test-token";
        let raw = sign_fields(
            token,
            &[("auth_date", "1700000000"), ("signature", "signed-value")],
        );
        let tampered = raw.replace("signed-value", "other-value");
        assert_eq!(
            validate_init_data(&tampered, token, 1_700_000_030, 60),
            Err(InitDataError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_duplicate_hash_fields() {
        let raw = "auth_date=1700000000&hash=aa&hash=bb";
        assert_eq!(
            validate_init_data(raw, "token", 1_700_000_030, 60),
            Err(InitDataError::DuplicateParameter)
        );
    }

    #[test]
    fn rejects_stale_auth_date_after_signature_verification() {
        let token = "token";
        let raw = sign_fields(token, &[("auth_date", "1000"), ("user", r#"{"id":42}"#)]);
        assert_eq!(
            validate_init_data(&raw, token, 2000, 300),
            Err(InitDataError::Expired)
        );
    }

    #[test]
    fn parses_and_formats_startapp_referral() {
        let payload = StartAppPayload::parse("p_123_r_456").unwrap();
        assert_eq!(payload, StartAppPayload { product_id: 123, ref_tg_id: 456 });
        assert_eq!(payload.to_start_param(), "p_123_r_456");
        assert_eq!(StartAppPayload::parse("p_0_r_456"), Err(StartAppPayloadError::InvalidProductId));
        assert!(StartAppPayload::parse("p_123_r_456_extra").is_err());
    }

    #[test]
    fn stars_invoice_payload_round_trips_and_uses_xtr() {
        let payload = StarsInvoicePayload {
            order_id: "ord_kian_01-abc".to_string(),
            product_id: 123,
            buyer_tg_id: 456,
            ref_tg_id: Some(789),
        };
        let encoded = payload.encode().unwrap();
        assert!(encoded.len() <= MAX_STARS_INVOICE_PAYLOAD_BYTES);
        assert_eq!(StarsInvoicePayload::parse(&encoded).unwrap(), payload);
        assert_eq!(StarsInvoicePayload::CURRENCY, "XTR");
    }
}
