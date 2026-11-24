//! Drop-in Rust Ranker for KIAN v2 Marketplace (`crates/domain` / `crates/db`)
//! Directly ported from `xai-org/x-algorithm/xai-value-model/scoring.rs` & `home-mixer/params/param.rs`.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct KianRankWeights {
    pub copy_link_share: f64,   // 20.0
    pub reply_or_review: f64,   // 5.0
    pub mutual_reply_boost: f64,// 15.0
    pub remix_or_quote: f64,    // 5.0
    pub share_dm: f64,          // 5.0
    pub follow_creator: f64,    // 4.0
    pub like_or_star: f64,      // 0.708
    pub click_dwell_sec: f64,   // 0.385
    pub click: f64,             // 0.274
    pub not_interested: f64,    // -53.07
    pub report: f64,            // -234.0
    pub author_decay: f64,      // e.g. 0.5
    pub author_floor: f64,      // e.g. 0.25
}

impl Default for KianRankWeights {
    fn default() -> Self {
        Self {
            copy_link_share: 20.0,
            reply_or_review: 5.0,
            mutual_reply_boost: 15.0,
            remix_or_quote: 5.0,
            share_dm: 5.0,
            follow_creator: 4.0,
            like_or_star: 0.708,
            click_dwell_sec: 0.385,
            click: 0.274,
            not_interested: -53.07,
            report: -234.0,
            author_decay: 0.5,
            author_floor: 0.25,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CandidateSignals {
    pub creator_id: u64,
    pub p_copy_link: f64,
    pub p_reply: f64,
    pub is_mutual_follow: bool,
    pub p_remix: f64,
    pub p_share_dm: f64,
    pub p_follow_creator: f64,
    pub p_like: f64,
    pub p_click: f64,
    pub expected_click_dwell_secs: f64,
    pub p_not_interested: f64,
    pub p_report: f64,
}

pub fn fuse_score(w: &KianRankWeights, s: &CandidateSignals) -> f64 {
    let reply_w = if s.is_mutual_follow {
        w.reply_or_review + w.mutual_reply_boost
    } else {
        w.reply_or_review
    };
    s.p_copy_link * w.copy_link_share
        + s.p_reply * reply_w
        + s.p_remix * w.remix_or_quote
        + s.p_share_dm * w.share_dm
        + s.p_follow_creator * w.follow_creator
        + s.p_like * w.like_or_star
        + s.p_click * w.click
        + s.expected_click_dwell_secs * w.click_dwell_sec
        + s.p_not_interested * w.not_interested
        + s.p_report * w.report
}

pub fn apply_creator_diversity(scores: &[f64], creators: &[u64], decay: f64, floor: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..scores.len()).collect();
    order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
    let mut seen: HashMap<u64, u32> = HashMap::new();
    let mut out = vec![0.0; scores.len()];
    for idx in order {
        let k = *seen.get(&creators[idx]).unwrap_or(&0);
        let mult = (1.0 - floor) * decay.powf(f64::from(k)) + floor;
        out[idx] = scores[idx] * mult;
        seen.insert(creators[idx], k + 1);
    }
    out
}
