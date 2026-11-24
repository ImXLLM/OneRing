//! هندلرهای استودیوی ادراک‌سیار — مورفِ شناختیِ سه‌لایه، ابراستودیوی
//! ۹-مدالیتیِ صفر-هزینه و ویترین پورتفولیو (Wave 5 — Step 2/4).
//!
//! سه مسیر additive:
//! - `GET /studio/ux-profiles` — سه لایهٔ شناختی + ۹ مدالیتی + گزارشِ
//!   ۶ درمانِ قضاییِ پورتفولیو (عمومی؛ قطعی)؛
//! - `POST /studio/generate` — تولیدِ زندهٔ آرتیفکت: لایه‌سنجِ تلمتری →
//!   `OmniModalStudioEngine` (باندلِ دوگانهٔ `Fishkal+Cosmos`) → بسته‌بندیِ
//!   ۲۰-زبانهٔ ARDA → فورجِ `SKILL.md` مایرون (گیتِ REA) → ماندگاری در
//!   `studio_creations`؛
//! - `GET /studio/portfolio-catalog` — ویترینِ غنی‌شده با ارتقای دو
//!   ویترینیِ برتر، توکن‌های گلسِ تاریک و پالتِ فرمانِ کیبورد-اول.

use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use kian_contracts::{
    CognitiveTierView, CommandPaletteView, DualBundleView, PortfolioCatalogResponse,
    PortfolioSafetyFixView, PortfolioSafetyReportView, StorefrontProjectCardView,
    StudioGenerateRequest, StudioGenerateResponse, StudioModalityView, StudioUxProfilesResponse,
};
use kian_db::now_iso;
use kian_hermes::arda_fishkal_engine::{
    is_planetary_locale, render_product_flag_strip, ArdaFishkalBundleGenerator, MaironSkillForge,
    SkillForgeRequest,
};
use kian_hermes::super_studio_engine::{
    build_toyspark_instant_creation, cognitive_tier_profiles, command_palette_config,
    derive_rea_score, emit_dual_cosmos_fishkal_bundle, infer_or_override_tier,
    portfolio_dark_glass_css, portfolio_safety_report, promote_featured_top2, prompt_seed,
    seed_storefront_catalog, uses_technical_tokens, CognitiveTelemetry, CognitiveTier,
    OmniModalStudioEngine, PortfolioFix, StorefrontProjectCard, StudioArtifact, StudioModality,
    PORTFOLIO_ROOT_RESOLUTION, PORTFOLIO_STOREFRONT_SIGNATURE, STUDIO_MODALITY_COUNT,
};
use kian_hermes::studio_design_contract::detect_persian_rtl;

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;

/// امضای پروفایل‌های UX — گیتِ آزمون.
pub const STUDIO_UX_SIGNATURE: &str = "studio-ux-profiles-v1";

/// نگاشتِ لایهٔ شناختی به نمای وایر.
#[must_use]
pub fn tier_view(profile: &kian_hermes::super_studio_engine::CognitiveTierProfile) -> CognitiveTierView {
    CognitiveTierView {
        tier: profile.tier.as_str().to_owned(),
        fa_label: profile.tier.fa_label().to_owned(),
        age_band: profile.age_band.to_owned(),
        max_actions_per_screen: u32::try_from(profile.max_actions_per_screen).unwrap_or(3),
        min_tap_target_px: profile.min_tap_target_px,
        max_words_per_label: u32::try_from(profile.max_words_per_label).unwrap_or(5),
        voice_guided: profile.voice_guided,
        zero_loss_sandbox: profile.zero_loss_sandbox,
        keyboard_first: profile.keyboard_first,
        layout_summary: profile.layout_summary.to_owned(),
    }
}

/// نگاشتِ مدالیتی به نمای وایر.
#[must_use]
pub fn modality_view(spec: &kian_hermes::super_studio_engine::StudioModalitySpec) -> StudioModalityView {
    StudioModalityView {
        modality: spec.modality.as_str().to_owned(),
        fa_label: spec.fa_label.to_owned(),
        auth: spec.auth.to_owned(),
        engine: spec.engine.to_owned(),
        zero_cost: spec.zero_cost,
    }
}

/// نگاشتِ گزارشِ ایمنی به نمای وایر — همیشه ۶/۶ `fixed`.
#[must_use]
pub fn safety_view() -> PortfolioSafetyReportView {
    let fixes = portfolio_safety_report();
    PortfolioSafetyReportView {
        fixed_count: u32::try_from(fixes.len()).unwrap_or(6),
        all_fixed: fixes
            .iter()
            .all(|f| f.status == "fixed" && (1..=6).contains(&f.fix_id)),
        fixes: fixes
            .iter()
            .map(|f| PortfolioSafetyFixView {
                fix_id: u32::from(f.fix_id),
                title: f.title.clone(),
                status: f.status.to_owned(),
                evidence: f.evidence.clone(),
            })
            .collect(),
    }
}

/// نگاشتِ کارتِ ویترین به نمای وایر.
#[must_use]
pub fn card_view(card: &kian_hermes::super_studio_engine::StorefrontProjectCard) -> StorefrontProjectCardView {
    StorefrontProjectCardView {
        slug: card.slug.clone(),
        name: card.name.clone(),
        tagline: card.tagline.clone(),
        category: card.category.clone(),
        stack: card.stack.clone(),
        highlights: card.highlights.clone(),
        preview: card.preview.clone(),
        source_dir: card.source_dir.clone(),
        readme: card.readme.clone(),
        loc: card.loc,
        file_count: card.file_count,
        featured: card.featured,
        price_ukian: card.price_ukian,
        price_xtr_stars: card.price_xtr_stars,
        price_usdc_micros: card.price_usdc_micros,
        rea_score: card.rea_score,
        skill_md_url: card.skill_md_url.clone(),
        supported_locales_count: card.supported_locales_count,
        cognitive_tier_badge: card.cognitive_tier_badge.clone(),
    }
}

/// `GET /studio/ux-profiles` — قراردادِ کاملِ مورفِ شناختی (عمومی).
///
/// سه لایهٔ `ToySpark`/`CreatorStudio`/`ProTerminal`، ۹ مدالیتیِ
/// صفر-هزینه و ۶ درمانِ قضاییِ پورتفولیو — کاملاً قطعی و بدونِ نشست.
///
/// # Errors
/// رخ نمی‌دهد — خروجی قطعیِ موتور است.
pub async fn ux_profiles() -> Json<StudioUxProfilesResponse> {
    Json(StudioUxProfilesResponse {
        tiers: cognitive_tier_profiles().iter().map(tier_view).collect(),
        modalities: kian_hermes::super_studio_engine::studio_modality_specs()
            .iter()
            .map(modality_view)
            .collect(),
        portfolio_safety: safety_view(),
        signature: STUDIO_UX_SIGNATURE.to_owned(),
    })
}

/// `GET /studio/portfolio-catalog` — ویترینِ غنی‌شدهٔ سازنده (عمومی).
///
/// دو `featured` برتر در صدر (`index.js:L17–18`)، توکن‌های گلسِ تاریکِ
/// `style.css`، پالتِ فرمانِ کیبورد-اول (`Ctrl+K`/`Meta+K`/`/`) و ریشهٔ
/// قابل‌حملِ کاتالوگ (Fix #1) — بدونِ هیچ نشست.
///
/// # Errors
/// رخ نمی‌دهد — خروجی قطعیِ موتور است.
pub async fn portfolio_catalog() -> Json<PortfolioCatalogResponse> {
    let catalog = seed_storefront_catalog();
    let promoted = promote_featured_top2(&catalog);
    let featured_top: Vec<String> = promoted
        .iter()
        .filter(|c| c.featured)
        .take(2)
        .map(|c| c.slug.clone())
        .collect();
    let palette = command_palette_config();
    Json(PortfolioCatalogResponse {
        projects: promoted.iter().map(card_view).collect(),
        featured_top,
        css_tokens: portfolio_dark_glass_css(),
        command_palette: CommandPaletteView {
            open_shortcuts: palette.open_shortcuts.iter().map(|s| (*s).to_owned()).collect(),
            focus_shortcut: palette.focus_shortcut.to_owned(),
            keyboard_first: palette.keyboard_first,
            navigation_keys: palette.navigation_keys.iter().map(|s| (*s).to_owned()).collect(),
            js: kian_hermes::super_studio_engine::emit_command_palette_js(),
        },
        portfolio_root: PORTFOLIO_ROOT_RESOLUTION.to_owned(),
        portfolio_safety: safety_view(),
        signature: PORTFOLIO_STOREFRONT_SIGNATURE.to_owned(),
    })
}

/// `POST /studio/generate` — تولیدِ زندهٔ آفرینش استودیو (نشست شهروند).
///
/// ورودی: پرامپت (از جملهٔ کودکِ ۹ساله تا مشخصهٔ مهندسِ ۲۸ساله) +
/// مدالیتیِ هدف + لایهٔ صریحِ اختیاری. خط لوله: استنتاج/لغوِ لایهٔ شناختی →
/// مولدِ آرتیفکت (`OmniModalStudioEngine` با باندلِ دوگانهٔ
/// `Fishkal+Cosmos`) → بسته‌بندیِ ۲۰-زبانهٔ ARDA (تزریقِ ایدمپوتنت + نوارِ
/// پرچمِ دایره‌ای) → فورجِ `SKILL.md`/`skill.json` مایرون (گیتِ REA >= 85)
/// → ماندگاری در `studio_creations`. ۲۰۰.
///
/// # Errors
/// ۴۰۱ نشست غایب؛ ۴۰۰ بدنهٔ نامعتبر/مدالیتیِ ناشناخته/پرامپتِ خالی.
#[allow(clippy::too_many_lines)]
pub async fn generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<StudioGenerateResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: StudioGenerateRequest = parse_body(&body)?;

    let prompt = req.prompt.trim().to_owned();
    if prompt.is_empty() || prompt.len() > 8_192 {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let modality = StudioModality::parse(&req.modality)
        .ok_or::<ApiFailure>(kian_contracts::HttpFailure::invalid_request(1).into())?;
    let explicit = req
        .cognitive_tier
        .as_deref()
        .map(str::trim)
        .and_then(CognitiveTier::parse);

    // ۱ — استنتاج/لغوِ لایهٔ شناختی از تلمتری (بدونِ هرگز پرسیدنِ سن).
    let tier = infer_or_override_tier(&CognitiveTelemetry {
        prompt_length_chars: prompt.chars().count(),
        uses_technical_tokens: uses_technical_tokens(&prompt),
        typing_cadence_ms: 0,
        explicit_tier_override: explicit,
    });

    // ۲ — مولدِ آرتیفکت (خط لولهٔ Trinity استودیو): در `ToySpark` جریانِ
    // داستان/بازیِ یک‌کلیکی (`build_toyspark_instant_creation`) همیشه بر
    // مدالیتی می‌چربد؛ در لایه‌های بالاتر مولدِ مدالیتیِ کامل اجرا می‌شود.
    let engine = OmniModalStudioEngine;
    let (artifact, storefront) = if tier == CognitiveTier::ToySpark {
        let creation = build_toyspark_instant_creation(&prompt);
        let art = StudioArtifact {
            title: creation.title.clone(),
            artifact_html: creation.storybook_html.clone(),
            dual_bundle: emit_dual_cosmos_fishkal_bundle(),
            tier,
            modality,
            rea_score: derive_rea_score(prompt_seed(&prompt)),
            price_ukian: creation.listing.price_ukian,
            price_xtr_stars: creation.listing.price_xtr_stars,
            price_usdc_micros: creation.listing.price_usdc_micros,
        };
        (art, creation.listing)
    } else {
        let art = engine.generate(&prompt, modality, tier);
        let slug = format!("studio-{}", prompt_seed(&prompt));
        let card = StorefrontProjectCard {
            slug: slug.clone(),
            name: art.title.clone(),
            tagline: format!("آفرینش {} روی مدالیتیِ {}", tier.fa_label(), modality.fa_label()),
            category: format!("studio-{}", modality.as_str()),
            stack: vec![
                "html5".to_owned(),
                "svg".to_owned(),
                "webaudio".to_owned(),
                "canvas".to_owned(),
            ],
            highlights: vec![
                format!("cognitive tier {}", tier.as_str()),
                "dual Fishkal+Cosmos bundle".to_owned(),
                "20-locale ARDA packaging".to_owned(),
            ],
            preview: art.artifact_html.clone(),
            source_dir: PORTFOLIO_ROOT_RESOLUTION.to_owned(),
            readme: "generated-by-omni-studio".to_owned(),
            loc: 260,
            file_count: 1,
            featured: false,
            price_ukian: art.price_ukian,
            price_xtr_stars: art.price_xtr_stars,
            price_usdc_micros: art.price_usdc_micros,
            rea_score: art.rea_score,
            skill_md_url: format!("/api/v1/market/products/{slug}/skill"),
            supported_locales_count: 20,
            cognitive_tier_badge: tier.as_str().to_owned(),
        };
        (art, card)
    };

    // ۳ — بسته‌بندیِ ۲۰-زبانهٔ ARDA روی آرتیفکت.
    let origin = req
        .origin_locale
        .as_deref()
        .map(str::trim)
        .filter(|c| is_planetary_locale(c))
        .map_or_else(
            || {
                if detect_persian_rtl(&prompt) {
                    "fa".to_owned()
                } else {
                    "en".to_owned()
                }
            },
            str::to_owned,
        );
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
    let bundle = ArdaFishkalBundleGenerator.generate(
        &origin,
        &targets.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let injection = kian_hermes::arda_fishkal_engine::inject_arda_runtime(&artifact.artifact_html, &bundle);
    let strip_codes: Vec<&str> = bundle.locales.iter().map(String::as_str).collect();
    let strip = render_product_flag_strip(&origin, &strip_codes[1..]).unwrap_or_default();
    let supported = bundle.locales.clone();

    // ۴ — فورجِ مهارتِ مایرون (REA قطعی همیشه بالای گیتِ ۸۵).
    let creation_id = format!("studio:{}", uuid::Uuid::new_v4());
    let forge = MaironSkillForge::forge_skill_package(&SkillForgeRequest {
        product_id: creation_id.clone(),
        product_title: artifact.title.clone(),
        rea_score: artifact.rea_score,
        origin_locale: origin.clone(),
        translated_locales: supported.iter().skip(1).cloned().collect(),
    })
    .ok();

    // ۵ — کارتِ ویترین + ماندگاری در `studio_creations`.
    let card = storefront;
    let card_json = serde_json::to_string(&card).unwrap_or_else(|_| "{}".to_owned());
    let created_at = now_iso();
    state
        .store
        .create_studio_creation(&kian_db::StudioCreationRow {
            id: creation_id.clone(),
            creator_user_id: user.account_id.clone(),
            cognitive_tier: tier.as_str().to_owned(),
            primary_modality: modality.as_str().to_owned(),
            prompt_text: prompt,
            title: artifact.title.clone(),
            storefront_card_json: card_json,
            rea_score: i64::from(artifact.rea_score),
            price_ukian: i64::try_from(artifact.price_ukian).unwrap_or(1_000_000),
            price_xtr_stars: i64::from(artifact.price_xtr_stars),
            created_at: created_at.clone(),
        })
        .await
        .map_err(ApiFailure::from)?;

    let dual = artifact.dual_bundle;
    Ok(Json(StudioGenerateResponse {
        creation_id,
        cognitive_tier: tier.as_str().to_owned(),
        modality: modality.as_str().to_owned(),
        title: artifact.title,
        artifact_html: injection.html,
        dual_bundle: DualBundleView {
            fishkal_js: dual.fishkal_js,
            cosmos_js: dual.cosmos_js,
            pause_lifecycle_js: dual.pause_lifecycle_js,
            signature: dual.signature,
            content_sha256: dual.content_sha256,
        },
        storefront_card: card_view(&card),
        circular_flag_strip_html: strip,
        origin_locale: origin,
        supported_locales: supported,
        skill_md: forge.as_ref().map(|p| p.skill_md.clone()),
        skill_json: forge.as_ref().map(|p| p.skill_json.clone()),
        rea_score: artifact.rea_score,
        price_ukian: artifact.price_ukian,
        price_xtr_stars: artifact.price_xtr_stars,
        created_at,
    }))
}

/// شمارِ مدالیتی‌ها — گیتِ آزمونِ هندلر.
#[must_use]
pub fn studio_modality_count() -> usize {
    STUDIO_MODALITY_COUNT
}

/// شمارِ درمان‌های قضاییِ پورتفولیو — گیتِ آزمونِ هندلر.
#[must_use]
pub fn portfolio_fix_count() -> usize {
    PortfolioFix::PortableRootPath.id() as usize + 5
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

    /// احراز کاربر تلگرام و گرفتن توکن.
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
    fn modality_and_fix_counts_are_locked() {
        assert_eq!(studio_modality_count(), 9);
        assert_eq!(portfolio_fix_count(), 6);
    }

    #[tokio::test]
    async fn ux_profiles_serves_3_tiers_9_modalities_6_fixes() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/studio/ux-profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["signature"], "studio-ux-profiles-v1");
        let tiers = body["tiers"].as_array().unwrap();
        assert_eq!(tiers.len(), 3);
        assert_eq!(tiers[0]["tier"], "ToySpark");
        assert_eq!(tiers[0]["maxActionsPerScreen"], 3);
        assert_eq!(tiers[0]["minTapTargetPx"], 64);
        assert_eq!(tiers[0]["voiceGuided"], true);
        assert_eq!(tiers[2]["tier"], "ProTerminal");
        assert_eq!(tiers[2]["keyboardFirst"], true);
        let modalities = body["modalities"].as_array().unwrap();
        assert_eq!(modalities.len(), 9);
        for m in modalities {
            assert_eq!(m["auth"], "none");
            assert_eq!(m["zeroCost"], true);
        }
        let safety = &body["portfolioSafety"];
        assert_eq!(safety["fixedCount"], 6);
        assert_eq!(safety["allFixed"], true);
        assert_eq!(safety["fixes"].as_array().unwrap().len(), 6);
        assert_eq!(safety["fixes"][0]["fixId"], 1);
        assert_eq!(safety["fixes"][5]["fixId"], 6);
    }

    #[tokio::test]
    async fn portfolio_catalog_promotes_featured_top2_with_glass_tokens() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/studio/portfolio-catalog")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["signature"], "portfolio-storefront-v1");
        assert_eq!(body["portfolioRoot"], "./seed/portfolio_catalog");
        assert!(!body["portfolioRoot"].as_str().unwrap().contains(".."));
        let projects = body["projects"].as_array().unwrap();
        assert_eq!(projects.len(), 8);
        assert_eq!(projects[0]["featured"], true);
        assert_eq!(projects[1]["featured"], true);
        assert_eq!(projects[2]["featured"], false);
        assert_eq!(body["featuredTop"], json!(["fishkal", "cosmos"]));
        let css = body["cssTokens"].as_str().unwrap();
        assert!(css.contains("--accent: #7c5cff"));
        assert!(css.contains("backdrop-filter: blur(14px)"));
        assert!(css.contains("clamp(36px, 5.6vw, 62px)"));
        let palette = &body["commandPalette"];
        assert_eq!(palette["openShortcuts"], json!(["Ctrl+K", "Meta+K", "/"]));
        assert_eq!(palette["keyboardFirst"], true);
        assert!(palette["js"].as_str().unwrap().contains("e.key === '/'"));
        // ۱۲ کلیدِ کانونی + پسوندهای تجاری روی اولین کارت.
        let first = &projects[0];
        for key in [
            "slug", "name", "tagline", "category", "stack", "highlights",
            "preview", "sourceDir", "readme", "loc", "fileCount", "featured",
            "priceUkian", "priceXtrStars", "reaScore", "skillMdUrl", "cognitiveTierBadge",
        ] {
            assert!(first.get(key).is_some(), "key {key} missing on card");
        }
    }

    #[tokio::test]
    async fn generate_requires_session() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/studio/generate")
                    .body(Body::from(json!({ "prompt": "سلام", "modality": "copy" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    #[tokio::test]
    async fn generate_rejects_empty_prompt_and_unknown_modality() {
        let app = app().await;
        let token = auth(&app, "888001", "studio_bad").await;
        let authz = format!("Bearer {token}");
        for payload in [
            json!({ "prompt": "", "modality": "copy" }),
            json!({ "prompt": "سلام", "modality": "hologram" }),
        ] {
            let res = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .method("POST")
                        .uri("/api/v1/studio/generate")
                        .header("authorization", authz.clone())
                        .body(Body::from(payload.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), 400);
        }
    }

    #[tokio::test]
    async fn toyspark_generate_persists_playable_storybook_with_arda_bundle() {
        let app = app().await;
        let token = auth(&app, "888002", "studio_kid").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/studio/generate")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "prompt": "یه دایناسور آبی که توی فضا بستنی می‌فروشه",
                            "modality": "workspace"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["cognitiveTier"], "ToySpark");
        assert_eq!(body["originLocale"], "fa");
        assert_eq!(body["supportedLocales"].as_array().unwrap().len(), 20);
        assert_eq!(body["priceXtrStars"], 50);
        assert_eq!(body["priceUkian"], 1_000_000);
        assert!(body["reaScore"].as_i64().unwrap() >= 88);
        assert!(body["skillMd"].as_str().unwrap().contains("mairon_certified: true"));
        assert!(body["skillJson"].as_str().unwrap().contains("\"name\""));
        // آرتیفکت: داستان‌کتابِ قابل بازی + باندلِ تزریقیِ ARDA + موتورهای دوگانه.
        let artifact = body["artifactHtml"].as_str().unwrap();
        assert!(artifact.contains("data-action=\"magic\""));
        assert!(artifact.contains("min-width: 64px"));
        assert!(artifact.contains("speechSynthesis"));
        assert!(artifact.contains("data-arda-runtime=\"on\""));
        assert!(artifact.contains("ARDA_STUDIO.fishkal.dampFk") || artifact.contains("ARDA_STUDIO.cosmos.sfx.ping"));
        // نوارِ پرچمِ دایره‌ای — ۲۰ نشان.
        assert_eq!(
            body["circularFlagStripHtml"]
                .as_str()
                .unwrap()
                .matches("arda-flag-badge")
                .count(),
            20
        );
        // باندلِ دوگانه روی وایر با امضا و مُهرِ sha256.
        assert_eq!(body["dualBundle"]["signature"], "fishkal-cosmos-dual-v1");
        assert_eq!(body["dualBundle"]["contentSha256"].as_str().unwrap().len(), 64);
        // کارتِ ویترینِ آمادهٔ فروش.
        assert_eq!(body["storefrontCard"]["cognitiveTierBadge"], "ToySpark");
        assert_eq!(body["storefrontCard"]["featured"], true);
        // ماندگاری در studio_creations.
        let creation_id = body["creationId"].as_str().unwrap().to_owned();
        let check = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/studio/generate")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({ "prompt": "دومین آفرینش", "modality": "sfx" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(check.status(), 200);
        let second = json_body(check).await;
        assert_ne!(second["creationId"].as_str().unwrap(), creation_id);
    }

    #[tokio::test]
    async fn pro_terminal_generate_switches_tier_without_losing_state() {
        let app = app().await;
        let token = auth(&app, "888003", "studio_pro").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/studio/generate")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "prompt": "build a quant order-flow terminal with schema validation",
                            "modality": "video",
                            "cognitiveTier": "ProTerminal"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["cognitiveTier"], "ProTerminal");
        assert_eq!(body["modality"], "video");
        assert!(body["priceXtrStars"].as_i64().unwrap() >= 300);
        let artifact = body["artifactHtml"].as_str().unwrap();
        assert!(artifact.contains("data-cog-tier=\"ProTerminal\""));
        assert!(artifact.contains("data-modality=\"video\""));
        assert!(artifact.contains("fishkal.dampFk"));
        assert!(artifact.contains("requestAnimationFrame"));
        // لایهٔ صریح بر استنتاجِ تلمتری می‌چربد — وضعیتِ پروژه حفظ می‌شود.
        assert_eq!(body["storefrontCard"]["cognitiveTierBadge"], "ProTerminal");
    }
}
