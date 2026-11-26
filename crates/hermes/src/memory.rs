//! The Hermes 7-layer memory — the fixed classification of notes.
//!
//! Each note lives in exactly one layer; the layers are a compile-time closed
//! enum so there is no scratchable free-form string.

/// Memory layers — the brain charter's 7 layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryLayer {
    /// Working — short sentences from the current session
    Working,
    /// Episodic — "what the user asked / received"
    Episodic,
    /// Semantic — stable facts about the user
    Semantic,
    /// Procedural — recurring work patterns
    Procedural,
    /// Identity — the reputation score and its changes
    Identity,
    /// Financial — money events as read (never stores an amount itself)
    Financial,
    /// Social — relationships and referrals
    Social,
}

impl MemoryLayer {
    /// The database storage string — fixed `snake_case`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryLayer::Working => "working",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Semantic => "semantic",
            MemoryLayer::Procedural => "procedural",
            MemoryLayer::Identity => "identity",
            MemoryLayer::Financial => "financial",
            MemoryLayer::Social => "social",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exactly 7 layers with unique strings — the database contract.
    #[test]
    fn seven_layers_with_unique_strings() {
        let all = [
            MemoryLayer::Working,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Procedural,
            MemoryLayer::Identity,
            MemoryLayer::Financial,
            MemoryLayer::Social,
        ];
        assert_eq!(all.len(), 7);
        let mut strings: Vec<&str> = all.iter().map(|l| l.as_str()).collect();
        strings.sort_unstable();
        assert!(strings.windows(2).all(|w| w[0] != w[1]), "layer strings must be unique");
    }
}
