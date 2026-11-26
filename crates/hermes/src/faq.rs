//! Instant FAQ answerer — port of the Hermes bot "Zero-Token Instant FAQ" pattern (folder 03).
//!
//! Customers' frequent questions are answered with zero LLM tokens in "0
//! milliseconds" via deterministic keyword matching; only when no hook matches does the question
//! reach the Hermes engine (LLM). Every canned answer is plain international English (the
//! [`crate::prompt`] rule for the MAIRON brand).
#![forbid(unsafe_code)]

/// One of the key FAQ questions — for tests and reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaqTopic {
    /// Short key
    pub key: &'static str,
    /// Canned answer (international English)
    pub answer: &'static str,
    /// Matching keywords (all lowercase)
    pub keywords: &'static [&'static str],
}

/// MAIRON customers' key questions — purchase/escrow, commission, Tether/TON, building with Hermes,
/// support/arbitration + catalog and pricing (port of the Hermes bot FAQ).
pub const FAQ_TOPICS: &[FaqTopic] = &[
    FaqTopic {
        key: "buy_escrow",
        keywords: &["escrow", "buy", "purchase", "payment", "delivery", "safe to pay"],
        answer: "Every purchase on MAIRON is escrow-protected: your money stays locked until the product is delivered, and it is only released after your explicit confirmation.\n• Payment runs inside the app with Din — you never send money directly to a seller.\n• If a product is not delivered, the dispute goes to the Parliament arbitration and the amount is returned to you.\nEvery purchase sits under a safe ceiling with instant release; there is no risk in holding your money.",
    },
    FaqTopic {
        key: "affiliate",
        keywords: &["affiliate", "commission", "referral", "invite", "earn", "percentage"],
        answer: "The referral commission is 25 percent of every sale, credited to your account immediately after the purchase settles.\n• Get your personal invite link from the referral page inside the mini-app.\n• No matter how much your network buys, you always receive the 25 percent share; there is no ceiling.\nFor fast growth, also share your link across your Telegram channels.",
    },
    FaqTopic {
        key: "usdt_ton",
        keywords: &["usdt", "ton", "deposit", "withdraw", "charge", "wallet", "base", "usdc"],
        answer: "USDT/TON deposits and withdrawals run from the wallet section of the mini-app, and every address is validated against the official chain checksums before it is registered.\n• Deposit: copy your own dedicated address and wait for the network confirmation; credit is automatic.\n• Withdrawal: the request is registered and, after admin approval, sent to your address.\nThe network fee is separate from the amount; double-check the address before sending.",
    },
    FaqTopic {
        key: "build_with_hermes",
        keywords: &["hermes", "build", "product", "studio", "create", "toyspark"],
        answer: "To build a product, write your idea inside the Hermes studio — every turn returns a complete, standalone HTML app with sales metadata attached.\n• First turn: define the goal, the visual style and the audience, or just write \"build\".\n• Output: a one-page interactive web application, ready to publish on the MAIRON market.\nAfter publishing, you can edit the suggested price and commission at any time.",
    },
    FaqTopic {
        key: "support_parliament",
        keywords: &["support", "arbitration", "parliament", "dispute", "complaint", "refund", "refound"],
        answer: "Support and arbitration run inside the \"MAIRON Parliament\": every dispute is resolved with member votes and published rules.\n• First open the support button inside the mini-app and send a message; the answer is usually instant.\n• If it is not settled, an arbitration case is registered and the escrow holds until the final vote.\nArbitration decisions are recorded in a transparent ledger; no party can take the money unilaterally.",
    },
    FaqTopic {
        key: "catalog",
        keywords: &["catalog", "products", "market", "store", "what do you sell"],
        answer: "The MAIRON market lists twenty active products: the eight most popular Trinity titles and twelve curated community products.\n• Popular titles: the SC workbench, the physical-physics FISHKAL game, the trading arena, the Nova messenger and four other titles.\n• Every product is a single-page online app; the live preview opens before you buy.\nSee the complete list inside the mini-app, in the Market section.",
    },
    FaqTopic {
        key: "pricing",
        keywords: &["price", "cost", "how much", "din", "pricing"],
        answer: "Prices run from 300 to 1,200 Din for the popular titles, and community products start from 50 Din.\n• Every price sits under a human-approved ceiling; after purchase, settlement and delivery are instant.\n• With a referral link, 25 percent of every purchase returns to your account.\nThe detailed price of each product is listed on its own market page.",
    },
];

/// Instant match — a question → canned answer (no tokens, no network).
///
/// Matching runs on lowercased text. The first topic whose keyword is present wins.
#[must_use]
pub fn match_instant_faq(query: &str) -> Option<&'static str> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }
    FAQ_TOPICS
        .iter()
        .find(|t| t.keywords.iter().any(|kw| q.contains(&kw.to_lowercase())))
        .map(|t| t.answer)
}

/// The found topic key — for routing, logs, and tests.
#[must_use]
pub fn match_instant_faq_key(query: &str) -> Option<&'static str> {
    let q = query.trim().to_lowercase();
    FAQ_TOPICS
        .iter()
        .find(|t| t.keywords.iter().any(|kw| q.contains(&kw.to_lowercase())))
        .map(|t| t.key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key mission question gets an answer with zero tokens.
    #[test]
    fn key_customer_questions_hit_instant_answers() {
        assert!(match_instant_faq_key("how do escrow-protected purchases work?").is_some());
        assert!(match_instant_faq_key("what is the affiliate commission percentage?").is_some());
        assert!(match_instant_faq_key("how do usdt and ton deposits and withdrawals work?").is_some());
        assert!(match_instant_faq_key("how do I build a product with the hermes studio?").is_some());
        assert!(match_instant_faq_key("where is the parliament arbitration support?").is_some());
    }

    /// Answers are non-empty and multi-line; the first-line + bullets structure is preserved.
    #[test]
    fn answers_are_structured_and_nonempty() {
        for t in FAQ_TOPICS {
            assert!(!t.answer.trim().is_empty(), "{}", t.key);
            assert!(t.answer.contains('•'), "{} without a bullet", t.key);
            assert!(t.answer.chars().count() > 120, "{} too short", t.key);
            assert!(!t.keywords.is_empty(), "{} without keywords", t.key);
        }
        // at least 5 key mission topics
        assert!(FAQ_TOPICS.len() >= 5);
    }

    /// Latin keywords also match (case-insensitive); an unrelated question → None.
    #[test]
    fn latin_keywords_and_misses() {
        assert!(match_instant_faq_key("how to pay with USDT?").is_some());
        assert!(match_instant_faq_key("ESCROW safe?").is_some());
        assert_eq!(match_instant_faq_key(""), None);
        assert_eq!(match_instant_faq_key("hello there!"), None);
        assert_eq!(match_instant_faq_key("xyzabc"), None);
    }

    /// Pure English: every answer and every keyword is free of Arabic-script codepoints.
    #[test]
    fn faq_is_pure_english() {
        for t in FAQ_TOPICS {
            for s in t.keywords.iter().copied().chain(std::iter::once(t.answer)) {
                assert!(
                    !s.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)),
                    "{} carries non-English script",
                    t.key
                );
            }
        }
    }
}
