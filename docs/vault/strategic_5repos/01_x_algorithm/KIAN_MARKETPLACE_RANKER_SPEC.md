# KIAN Marketplace & Feed Ranker Spec (Ported from `xai-org/x-algorithm`)

## Core Formula (`xai-value-model/scoring.rs` + `vm-ranker/scoring/dpp_model.rs`)
1. **Additive / Multiplicative Fusion (`fuse_heads`)**:
   - `engage_sum = P(save/bookmark)*20.0 + P(comment)*5.0 + P(share_dm)*5.0 + P(remix/fork)*5.0 + P(follow_creator)*4.0 + P(like)*0.708 + click_dwell_secs*0.385 - P(hide)*53.07 - P(report)*234.0`
   - `multiplicative`: `skip_factor(P(skip)) * (engage_sum + bias) * (1.0 + dwell_secs * w_dwell)`
2. **Cold-Start Thompson Sampling (`author_cold_start.rs`)**:
   - New KIAN Studio products (`age <= 7200s`, `impressions < 200`, `creator_followers < 50000`) sample `Beta(0.75 + conversions, 49.25 + non_conversions)` and lift promising items into catalog slots `15..16`.
3. **Author Diversity Penalty (`diversity_multiplier`)**:
   - `multiplier = (1.0 - floor) * decay.powf(k) + floor` where `k` is prior occurrences of the same creator in the ranked slate.
