// KIAN v2 Step-5 integration adapter (schema-mapping required before merge).
// Scores reproduce the supplied 09 card; they are asserted estimates, NOT measured benchmarks.
// The destination crate must provide ModelGenomeProfile and its field types.

pub fn builtin_gemini_4_aragon_high_profile() -> ModelGenomeProfile {
    ModelGenomeProfile {
        model_id: "gemini-4-aragon-high".into(),
        display_name: "GEMINI 4 Aragon HIGH".into(),
        reasoning_paradigm: "hybrid_thinking (profile claim; unverified)".into(),
        think_mode_active: true,
        requires_external_state_trace: false,
        max_simultaneous_rules_per_phase: 32,
        task_scores: vec![
            ("SystemArchitecture", 0.99),
            ("ComplexRustSystems", 0.97),
            ("MultiFileRefactoring", 0.96),
            ("InteractiveFrontendUx", 0.96),
            ("WebGlCanvasShaders", 0.96),
            ("SqliteLedgerDesign", 0.99),
            ("HardAlgorithmicDeduction", 0.99),
            ("ConcurrencyRootCause", 0.98),
            ("SecurityAuditing", 0.98),
            ("CodeReviewSpecCompliance", 0.99),
            ("MassiveContextRepositoryArchaeology", 0.99),
            ("AgenticToolCalling", 0.92),
            ("StructuredDataExtraction", 0.98),
            ("TestSuiteGeneration", 0.97),
            ("TechnicalGenomeSynthesis", 1.00),
            ("PersianRtlBilingualUx", 0.99),
            ("TelegramMiniAppIntegration", 0.98),
            ("SocialAlgorithmicGrowth", 0.97),
            ("LowLatencyRouting", 0.65),
            ("LongHorizonOrchestration", 0.99),
        ],
    }
}
