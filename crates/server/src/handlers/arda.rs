//! ARDA handlers — planetary 24-hour localization on the original `Fishkal` architecture.
//! (Wave 4 — Step 1/4).
//!
//! Two additive routes:
//! - `GET /arda/flags` — the public catalog of the 20 planetary languages with verbatim vector
//!   SVGs and circular badges (Article 6 transparency; no session);
//! - `POST /arda/localize-now` — live ARDA run on raw HTML or a product
//!   id: origin detection, bundle generation, idempotent injection, the circular flag
//!   strip, the mechanical-balance report and the content seal; if a product id is given,
//!   only the product’s own builder may seal it.

use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use kian_contracts::{
    ArdaFlagView, ArdaFlagsCatalogResponse, ArdaLocaleParityView, ArdaLocalizeRequest,
    ArdaLocalizeResponse, ArdaParityReportView, MAX_PREVIEW_HTML_BYTES,
};
use kian_db::now_iso;
use kian_hermes::arda_fishkal_engine::{
    arda_content_sha256, detect_origin_locale, inject_arda_runtime, is_planetary_locale,
    render_product_flag_strip, validate_catalog_parity, ArdaFishkalBundleGenerator,
    ARDA_RUNTIME_SIGNATURE, PLANETARY_LOCALE_COUNT,
};
use kian_domain::{DomainError, DomainErrorCode};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;

/// Read the supported-languages JSON — on failure → the migration 0008 default strip.
#[must_use]
pub fn parse_supported_locales(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(json)
        .unwrap_or_else(|_| vec!["en".to_owned(), "fa".to_owned(), "ar".to_owned()])
}

/// `GET /arda/flags` — the full 20-language planetary catalog (public).
///
/// SVGs verbatim from Fishkal (L2690–2704) and each language’s circular badge, ready to
/// render on the market card and the in-app switcher.
///
/// # Errors
/// No SQL error can occur while counting flags — this handler is deterministic.
pub async fn flags_catalog() -> Json<ArdaFlagsCatalogResponse> {
    let locales = kian_hermes::arda_fishkal_engine::FISHKAL_VECTOR_FLAGS
        .iter()
        .map(|l| ArdaFlagView {
            code: l.code.to_owned(),
            native_name: l.native_name.to_owned(),
            str_code: l.str_code.to_owned(),
            dir: l.dir.as_attr().to_owned(),
            intl_locale: l.intl_locale.to_owned(),
            svg: l.svg.to_owned(),
            circular_badge_html: kian_hermes::arda_fishkal_engine::render_circular_flag_badge(
                l.code,
            )
            .unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    Json(ArdaFlagsCatalogResponse {
        total: u32::try_from(locales.len()).unwrap_or(u32::MAX),
        locales,
        signature: ARDA_RUNTIME_SIGNATURE.to_owned(),
    })
}

/// `POST /arda/localize-now` — live ARDA run on raw HTML or a product.
///
/// Input: `html` or `product_id` (both → raw HTML takes priority; neither →
/// 400). Origin: a valid `origin_locale` or deterministic script detection. Target: valid
/// `target_locales` or all 20 languages minus the origin. If a product is given, only
/// the seller is authorized, and languages + timestamp are sealed (the forged
/// `COALESCE` skill is preserved). 200.
///
/// # Errors
/// 401 missing session; 400 invalid body/missing input/oversized HTML; 403 non-seller;
/// 404 missing product.
#[allow(clippy::too_many_lines)]
pub async fn localize_now(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<ArdaLocalizeResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: ArdaLocalizeRequest = parse_body(&body)?;

    let (html, product_id) = if let Some(raw) = req.html.as_deref().map(str::trim).filter(|h| !h.is_empty()) {
        if raw.len() > MAX_PREVIEW_HTML_BYTES {
            return Err(kian_contracts::HttpFailure::invalid_request(1).into());
        }
        (raw.to_owned(), None)
    } else if let Some(pid) = req.product_id.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        let product = state
            .store
            .get_product(pid)
            .await
            .map_err(ApiFailure::from)?
            .ok_or_else::<ApiFailure, _>(|| {
                DomainError::new(
                    DomainErrorCode::ProductNotFound,
                    format!("Product {pid} does not exist"),
                )
                    .into()
            })?;
        if product.seller_id != user.account_id {
            return Err(kian_contracts::HttpFailure::forbidden_escrow_role().into());
        }
        (product.preview_html.clone(), Some(pid.to_owned()))
    } else {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    };

    // origin — a valid provided value or deterministic detection.
    let origin = req
        .origin_locale
        .as_deref()
        .map(str::trim)
        .filter(|c| is_planetary_locale(c))
        .map_or_else(|| detect_origin_locale(&html).to_owned(), str::to_owned);

    // targets — the valid ones in input order, or all 20 minus the origin.
    let targets: Vec<String> = match req.target_locales.as_deref() {
        Some(list) if !list.is_empty() => list
            .iter()
            .filter(|c| is_planetary_locale(c))
            .map(String::clone)
            .filter(|c| *c != origin)
            .collect(),
        _ => kian_hermes::arda_fishkal_engine::all_locale_codes()
            .into_iter()
            .filter(|c| *c != origin)
            .map(str::to_owned)
            .collect(),
    };

    let bundle = ArdaFishkalBundleGenerator.generate(&origin, &targets.iter().map(String::as_str).collect::<Vec<_>>());
    let injection = inject_arda_runtime(&html, &bundle);
    let strip_codes: Vec<&str> = bundle.locales.iter().map(String::as_str).collect();
    let strip = render_product_flag_strip(&origin, &strip_codes[1..]).unwrap_or_default();

    // mechanical balance of the unified 6-domain catalog — a 100% gate.
    let parity = validate_catalog_parity(&kian_hermes::arda_fishkal_engine::default_planetary_catalog());
    let parity_view = ArdaParityReportView {
        base_locale: parity.base_locale.clone(),
        total_keys: u32::try_from(parity.total_keys).unwrap_or(u32::MAX),
        coverage_score: parity.coverage_score,
        fully_parity: parity.fully_parity,
        locales: parity
            .locales
            .iter()
            .map(|p| ArdaLocaleParityView {
                locale: p.locale.clone(),
                missing_keys: p.missing_keys.clone(),
                coverage_score: p.coverage_score,
            })
            .collect(),
    };

    let supported = bundle.locales.clone();
    let content_sha256 = arda_content_sha256(&injection.html);
    let mut last_localized_at = None;
    if let Some(pid) = &product_id {
        let stamp = now_iso();
        state
            .store
            .stamp_product_arda(
                pid,
                &origin,
                &serde_json::to_string(&supported).unwrap_or_else(|_| "[\"en\",\"fa\",\"ar\"]".to_owned()),
                None,
                None,
                &stamp,
            )
            .await
            .map_err(ApiFailure::from)?;
        last_localized_at = Some(stamp);
    }

    Ok(Json(ArdaLocalizeResponse {
        product_id,
        origin_locale: origin,
        supported_locales: supported,
        localized_html: injection.html,
        circular_flag_strip_html: strip,
        parity_report: parity_view,
        injected: injection.injected,
        content_sha256,
        last_localized_at,
    }))
}

/// Count of planetary languages — the handler test gate.
#[must_use]
pub fn planetary_locale_count() -> usize {
    PLANETARY_LOCALE_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use serde_json::json;
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-M3";
    const ADMIN: &str = "test-admin-key-0123456789";

    fn test_config() -> crate::config::ServerConfig {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.into());
        cfg
    }

    async fn app() -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        crate::routes::build_router(crate::state::AppState::new(store, test_config()).shared())
    }

    async fn json_body(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// Authenticate the Telegram user and obtain a token.
    async fn auth(app: &axum::Router, tg_id: &str, username: &str) -> String {
        let init_data = make_test_init_data(BOT, tg_id, Some(username));
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(json!({ "initData": init_data }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "auth of {username} must succeed");
        json_body(res).await["token"].as_str().unwrap().to_owned()
    }

    #[test]
    fn supported_locales_json_falls_back_safely() {
        assert_eq!(
            parse_supported_locales("[\"en\",\"fa\",\"ar\"]"),
            ["en", "fa", "ar"]
        );
        assert_eq!(
            parse_supported_locales("not json"),
            ["en", "fa", "ar"]
        );
    }

    #[test]
    fn planetary_count_is_locked() {
        assert_eq!(planetary_locale_count(), 20);
    }

    #[tokio::test]
    async fn flags_catalog_serves_20_verbatim_locales() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/arda/flags")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["total"], 20);
        assert_eq!(body["signature"], "arda-fishkal-20");
        let locales = body["locales"].as_array().unwrap();
        assert_eq!(locales.len(), 20);
        let fa = locales.iter().find(|l| l["code"] == "fa").unwrap();
        assert_eq!(fa["strCode"], "\u{641}\u{627}");
        assert_eq!(fa["nativeName"], "\u{641}\u{627}\u{631}\u{633}\u{6cc}");
        assert_eq!(fa["dir"], "rtl");
        assert_eq!(fa["intlLocale"], "fa-IR");
        // Fishkal verbatim SVG — the Iran flag with green `#239f40`.
        assert!(fa["svg"].as_str().unwrap().contains("viewBox=\"0 0 20 14\""));
        assert!(fa["svg"].as_str().unwrap().contains("#239f40"));
        assert!(fa["circularBadgeHtml"].as_str().unwrap().contains("border-radius:50%"));
        let ur = locales.iter().find(|l| l["code"] == "ur").unwrap();
        assert_eq!(ur["strCode"], "\u{627}\u{631}\u{62f}\u{648}");
        let en = locales.iter().find(|l| l["code"] == "en").unwrap();
        assert_eq!(en["dir"], "ltr");
    }

    #[tokio::test]
    async fn localize_now_requires_session() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/arda/localize-now")
                    .body(Body::from(json!({ "html": "<p>x</p>" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    #[tokio::test]
    async fn localize_now_rejects_missing_input() {
        let app = app().await;
        let token = auth(&app, "777001", "arda_empty").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/arda/localize-now")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(json!({}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
    }

    #[tokio::test]
    async fn localize_now_injects_fishkal_runtime_into_raw_html() {
        let app = app().await;
        let token = auth(&app, "777002", "arda_raw").await;
        let html = "<!DOCTYPE html><html><head><title>\u{645}\u{62d}\u{627}\u{633}\u{628}\u{647}‌\u{6af}\u{631} \u{633}\u{648}\u{62f} \u{628}\u{627}\u{646}\u{6a9}\u{6cc}</title></head><body><p>\u{6af}\u{632}\u{627}\u{631}\u{634} \u{645}\u{627}\u{647}\u{627}\u{646}\u{647}</p></body></html>";
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/arda/localize-now")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(json!({ "html": html }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["originLocale"], "fa");
        assert_eq!(body["injected"], true);
        assert_eq!(body["productId"], serde_json::Value::Null);
        assert!(body["lastLocalizedAt"].is_null());
        // all 20 languages minus the origin are enabled in the bundle.
        assert_eq!(body["supportedLocales"].as_array().unwrap().len(), 20);
        let localized = body["localizedHtml"].as_str().unwrap();
        // idempotent injection before `</head>` — the full treated Fishkal bundle.
        assert!(localized.contains("data-arda-runtime=\"on\""));
        assert!(localized.contains("root.lang=l; root.dir=DIR[l]; root.dataset.lang=l;"));
        assert!(localized.contains(":root[data-lang=\"fa\"]"));
        assert!(localized.contains("ARDA.canvas.drawBidiText"));
        assert!(localized.contains("role\",\"listbox\""));
        assert!(localized.contains("preventScroll:true"));
        let head_idx = localized.to_lowercase().rfind("</head>").unwrap();
        let style_idx = localized.find("<style").unwrap();
        assert!(style_idx < head_idx);
        // circular flag strip — origin + 19 translations.
        assert_eq!(
            body["circularFlagStripHtml"]
                .as_str()
                .unwrap()
                .matches("arda-flag-badge")
                .count(),
            20
        );
        // catalog mechanical balance — 100%.
        assert_eq!(body["parityReport"]["fullyParity"], true);
        assert_eq!(body["parityReport"]["coverageScore"], serde_json::json!(1.0));
        assert_eq!(body["parityReport"]["totalKeys"], 7);
        // sha256 seal — 64 hex digits.
        let sha = body["contentSha256"].as_str().unwrap();
        assert_eq!(sha.len(), 64);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn localize_now_stamps_product_for_owner_only() {
        let app = app().await;
        let seller = auth(&app, "777003", "arda_seller").await;
        let stranger = auth(&app, "777004", "arda_stranger").await;
        // the builder’s Persian product.
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "\u{62d}\u{633}\u{627}\u{628}\u{62f}\u{627}\u{631} — \u{645}\u{62d}\u{627}\u{633}\u{628}\u{647}‌\u{6af}\u{631} \u{633}\u{648}\u{62f} \u{6a9}\u{631}\u{6cc}\u{67e}\u{62a}\u{648}",
                            "description": "\u{627}\u{628}\u{632}\u{627}\u{631} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{62a}\u{6a9}‌\u{635}\u{641}\u{62d}\u{647}‌\u{627}\u{6cc} \u{641}\u{627}\u{631}\u{633}\u{6cc}",
                            "priceDin": 2_000,
                            "previewHtml": "<html lang=\"fa\" dir=\"rtl\"><body><h1>\u{645}\u{62d}\u{627}\u{633}\u{628}\u{647}‌\u{6af}\u{631} \u{633}\u{648}\u{62f}</h1><p>\u{6af}\u{632}\u{627}\u{631}\u{634} \u{645}\u{627}\u{644}\u{6cc} \u{645}\u{627}\u{647}\u{627}\u{646}\u{647}</p></body></html>"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let pid = json_body(res).await["id"].as_str().unwrap().to_owned();

        // a stranger → 403.
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/arda/localize-now")
                    .header("authorization", format!("Bearer {stranger}"))
                    .body(Body::from(json!({ "productId": pid }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // the seller themselves → 200 with a persistent seal.
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/arda/localize-now")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({ "productId": pid, "targetLocales": ["en", "ar", "tr"] }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["productId"], pid);
        assert_eq!(body["originLocale"], "fa");
        assert_eq!(body["supportedLocales"], json!(["fa", "en", "ar", "tr"]));
        assert!(body["lastLocalizedAt"].as_str().is_some());
        assert_eq!(body["injected"], true);
    }
}
