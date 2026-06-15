//! # SLM Adapter — Inference Backend Abstraction
//!
//! Defines the trait for connecting the EQ Gateway engine to a local SLM
//! inference backend (llama.cpp / llama-server-mini.exe), plus:
//! - [`ModelProfile`] — configuration for a specific model
//! - [`LlamaCppAdapter`] — HTTP client for llama-server-mini.exe
//! - [`MockAdapter`] — test double returning deterministic results
//!
//! # Architecture
//!
//! ```text
//! EQEngine::process_user_input()
//!     └── slm_adapter.classify(text, session_id)
//!              ├── LlamaCppAdapter → HTTP POST → llama-server-mini:9120/v1/chat/completions
//!              └── Returns InferenceResult with affect, intent, risk, summary
//! ```

use crate::EngineError;
use eq_state_compiler::{AffectPrimary, IntentCategory, RiskLevel};
use serde::{Deserialize, Serialize};
use std::time::Instant;

// ============================================================================
// Model Profile
// ============================================================================

/// Configuration for a specific SLM model.
#[derive(Debug, Clone)]
pub struct ModelProfile {
    /// Display name (e.g., "phi-4", "llama-3.2")
    pub name: String,
    /// Backend host (default: "127.0.0.1")
    pub host: String,
    /// Backend port (default: 9120)
    pub port: u16,
    /// Maximum tokens for inference response
    pub max_tokens: usize,
    /// Temperature for sampling (0.0–2.0)
    pub temperature: f32,
    /// Context window size
    pub context_size: usize,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ModelProfile {
    fn default() -> Self {
        Self {
            name: "phi-4".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9120,
            max_tokens: 256,
            temperature: 0.1,
            context_size: 4096,
            timeout_ms: 30_000,
        }
    }
}

impl ModelProfile {
    /// Build the backend base URL.
    pub fn backend_url(&self) -> String {
        format!("http://{}:{}/v1/chat/completions", self.host, self.port)
    }

    /// Build the health URL.
    pub fn health_url(&self) -> String {
        format!("http://{}:{}/health", self.host, self.port)
    }
}

// ============================================================================
// Inference Result
// ============================================================================

/// Structured result from SLM inference, ready for EQ State compilation.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// The raw text output from the SLM (for debugging/audit)
    pub raw_output: String,
    /// Classified primary emotional state
    pub affect_primary: AffectPrimary,
    /// Valence: -1.0 (negative) to 1.0 (positive)
    pub affect_valence: f64,
    /// Arousal: 0.0 (calm) to 1.0 (aroused)
    pub affect_arousal: f64,
    /// Classified user intent category
    pub intent_category: IntentCategory,
    /// Assessed risk level
    pub risk_level: RiskLevel,
    /// Privacy-safe anonymized summary of the user's text
    pub anonymized_summary: String,
    /// Total inference time in milliseconds
    pub processing_time_ms: u64,
}

// ============================================================================
// SLM Adapter Trait
// ============================================================================

/// Abstraction over SLM inference backends.
///
/// Implementations:
/// - [`LlamaCppAdapter`] — real HTTP calls to llama-server-mini.exe
/// - [`MockAdapter`] — deterministic results for testing
pub trait SLMAdapter: Send + Sync {
    /// Classify user text and return structured inference results.
    ///
    /// # Arguments
    /// * `text` — The user's raw input text (PII already redacted)
    /// * `session_id` — Ephemeral session identifier
    ///
    /// # Returns
    /// [`InferenceResult`] with affect, intent, risk, and summary.
    fn classify(&self, text: &str, session_id: &str) -> Result<InferenceResult, EngineError>;

    /// Quick health check: can the backend be reached?
    fn health_check(&self) -> Result<bool, EngineError>;

    /// Return a human-readable status string.
    fn status(&self) -> String;
}

// ============================================================================
// Classification Prompt
// ============================================================================

/// System prompt that instructs the SLM to produce structured EQ State output.
/// The model is asked to output a JSON object with specific fields.
const CLASSIFICATION_SYSTEM_PROMPT: &str = r#"You are EQ Gateway, an emotional context classifier.
Analyze the user's message and respond with ONLY a JSON object (no other text).
Use exactly this schema:

{
  "affect_primary": "neutral|calm|curious|pleased|hopeful|confused|uncertain|frustrated|angry|sad|anxious|overwhelmed|fatigued|embarrassed|lonely|excited|urgent",
  "affect_valence": -1.0 to 1.0,
  "affect_arousal": 0.0 to 1.0,
  "intent_category": "practical_guidance|emotional_support|decision_support|venting|planning|clarification|conflict_navigation|reflection|task_execution|creative_help|technical_help|safety_related",
  "risk_level": "none|low|medium|high|crisis",
  "anonymized_summary": "A short, privacy-safe summary that removes names, locations, and identifiers. Max 100 words."
}

Rules:
- affect_primary must be one of the listed values
- affect_valence: negative for negative emotions, positive for positive
- affect_arousal: 0.0 for very calm, 1.0 for very agitated/excited
- intent_category must be one of the listed values
- risk_level: "none" for normal chat, escalate if safety concern
- anonymized_summary: replace proper nouns with generic terms (e.g., "the user", "their manager")
- Output ONLY the JSON object, no markdown, no explanation
"#;

// ============================================================================
// OpenAI-Compatible Chat Types
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: usize,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
    #[allow(dead_code)]
    #[serde(default)]
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
    #[allow(dead_code)]
    #[serde(default)]
    role: String,
}

/// Parsed classification output from the SLM.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SlmClassification {
    affect_primary: String,
    affect_valence: f64,
    affect_arousal: f64,
    intent_category: String,
    risk_level: String,
    anonymized_summary: String,
}

// ============================================================================
// LlamaCppAdapter — Real HTTP Client
// ============================================================================

/// Adapter that calls `llama-server-mini.exe` via its HTTP API.
///
/// # Example
/// ```ignore
/// let profile = ModelProfile::default();
/// let adapter = LlamaCppAdapter::new(profile);
/// let result = adapter.classify("Hello, how are you?", "session-1");
/// ```
pub struct LlamaCppAdapter {
    profile: ModelProfile,
    client: ureq::Agent,
}

impl LlamaCppAdapter {
    /// Create a new adapter for a llama.cpp backend.
    pub fn new(profile: ModelProfile) -> Self {
        let client = ureq::Agent::new_with_defaults();
        Self { profile, client }
    }

    /// Parse the SLM's JSON output into an InferenceResult.
    fn parse_classification(raw: &str) -> Result<InferenceResult, EngineError> {
        // Try to extract JSON from the response (handle possible markdown fences)
        let json_str = if let Some(start) = raw.find('{') {
            if let Some(end) = raw.rfind('}') {
                &raw[start..=end]
            } else {
                raw
            }
        } else {
            raw
        };

        let parsed: SlmClassification = serde_json::from_str(json_str)
            .map_err(|e| EngineError::ConfigError(format!(
                "SLM output parse failed: {} — raw: {}", e, raw
            )))?;

        // Map string enums to Rust enums with fallback to Unknown
        let affect_primary = match parsed.affect_primary.to_lowercase().as_str() {
            "neutral" => AffectPrimary::Neutral,
            "calm" => AffectPrimary::Calm,
            "curious" => AffectPrimary::Curious,
            "pleased" => AffectPrimary::Pleased,
            "hopeful" => AffectPrimary::Hopeful,
            "confused" => AffectPrimary::Confused,
            "uncertain" => AffectPrimary::Uncertain,
            "frustrated" => AffectPrimary::Frustrated,
            "angry" => AffectPrimary::Angry,
            "sad" => AffectPrimary::Sad,
            "anxious" => AffectPrimary::Anxious,
            "overwhelmed" => AffectPrimary::Overwhelmed,
            "fatigued" => AffectPrimary::Fatigued,
            "embarrassed" => AffectPrimary::Embarrassed,
            "lonely" => AffectPrimary::Lonely,
            "excited" => AffectPrimary::Excited,
            "urgent" => AffectPrimary::Urgent,
            _ => AffectPrimary::Unknown,
        };

        let intent_category = match parsed.intent_category.to_lowercase().as_str() {
            "practical_guidance" => IntentCategory::PracticalGuidance,
            "emotional_support" => IntentCategory::EmotionalSupport,
            "decision_support" => IntentCategory::DecisionSupport,
            "venting" => IntentCategory::Venting,
            "planning" => IntentCategory::Planning,
            "clarification" => IntentCategory::Clarification,
            "conflict_navigation" => IntentCategory::ConflictNavigation,
            "reflection" => IntentCategory::Reflection,
            "task_execution" => IntentCategory::TaskExecution,
            "creative_help" => IntentCategory::CreativeHelp,
            "technical_help" => IntentCategory::TechnicalHelp,
            "safety_related" => IntentCategory::SafetyRelated,
            _ => IntentCategory::Unknown,
        };

        let risk_level = match parsed.risk_level.to_lowercase().as_str() {
            "none" => RiskLevel::None,
            "low" => RiskLevel::Low,
            "medium" => RiskLevel::Medium,
            "high" => RiskLevel::High,
            "crisis" => RiskLevel::Crisis,
            _ => RiskLevel::Unknown,
        };

        Ok(InferenceResult {
            raw_output: raw.to_string(),
            affect_primary,
            affect_valence: parsed.affect_valence.clamp(-1.0, 1.0),
            affect_arousal: parsed.affect_arousal.clamp(0.0, 1.0),
            intent_category,
            risk_level,
            anonymized_summary: parsed.anonymized_summary,
            processing_time_ms: 0, // Set by caller
        })
    }
}

impl SLMAdapter for LlamaCppAdapter {
    fn classify(&self, text: &str, _session_id: &str) -> Result<InferenceResult, EngineError> {
        let start = Instant::now();

        let request = ChatRequest {
            model: self.profile.name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: CLASSIFICATION_SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: text.to_string(),
                },
            ],
            max_tokens: self.profile.max_tokens,
            temperature: self.profile.temperature,
            stream: false,
        };

        let response = self
            .client
            .post(&self.profile.backend_url())
            .header("Content-Type", "application/json")
            .send_json(serde_json::to_value(&request)
                .map_err(|e| EngineError::ConfigError(format!("serialize request: {}", e)))?)
            .map_err(|e| {
                // Map ureq errors to EngineError
                match &e {
                    ureq::Error::Timeout(_) => EngineError::InferenceTimeout,
                    _ => EngineError::ConfigError(format!("HTTP error: {}", e)),
                }
            })?;

        // Read response body
        let resp_body: String = response
            .into_body()
            .read_to_string()
            .map_err(|e| EngineError::ConfigError(format!("read response: {}", e)))?;

        let chat_resp: ChatResponse = serde_json::from_str(&resp_body)
            .map_err(|e| EngineError::ConfigError(format!(
                "parse chat response: {} — body: {}",
                e,
                &resp_body.chars().take(200).collect::<String>()
            )))?;

        let raw_output = chat_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let mut result = Self::parse_classification(&raw_output)?;
        result.processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(result)
    }

    fn health_check(&self) -> Result<bool, EngineError> {
        let url = self.profile.health_url();
        match self.client.get(&url).call() {
            Ok(resp) => Ok(resp.status() == 200),
            Err(_) => Ok(false),
        }
    }

    fn status(&self) -> String {
        match self.health_check() {
            Ok(true) => format!("connected to {}:{} ({})", self.profile.host, self.profile.port, self.profile.name),
            Ok(false) => format!("backend unreachable at {}:{}", self.profile.host, self.profile.port),
            Err(e) => format!("error: {}", e),
        }
    }
}

// ============================================================================
// MockAdapter — Test Double
// ============================================================================

/// Deterministic adapter for testing the pipeline without a live backend.
pub struct MockAdapter {
    /// If true, `classify` will return an error (for error-path testing)
    pub fail_on_call: bool,
    /// Custom response to return (overrides default fixture)
    pub custom_result: Option<InferenceResult>,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self {
            fail_on_call: false,
            custom_result: None,
        }
    }

    /// Default test fixture result.
    fn default_result(processing_time_ms: u64) -> InferenceResult {
        InferenceResult {
            raw_output: r#"{"affect_primary":"curious","affect_valence":0.3,"affect_arousal":0.4,"intent_category":"clarification","risk_level":"none","anonymized_summary":"User is asking a question about how the system works."}"#.to_string(),
            affect_primary: AffectPrimary::Curious,
            affect_valence: 0.3,
            affect_arousal: 0.4,
            intent_category: IntentCategory::Clarification,
            risk_level: RiskLevel::None,
            anonymized_summary: "User is asking a question about how the system works.".to_string(),
            processing_time_ms,
        }
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SLMAdapter for MockAdapter {
    fn classify(&self, _text: &str, _session_id: &str) -> Result<InferenceResult, EngineError> {
        if self.fail_on_call {
            return Err(EngineError::InferenceTimeout);
        }
        let mut result = self.custom_result.clone()
            .unwrap_or_else(|| Self::default_result(5));
        result.processing_time_ms = 5;
        Ok(result)
    }

    fn health_check(&self) -> Result<bool, EngineError> {
        Ok(!self.fail_on_call)
    }

    fn status(&self) -> String {
        if self.fail_on_call {
            "mock adapter (failed)".to_string()
        } else {
            "mock adapter (healthy)".to_string()
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // ModelProfile tests
    // ------------------------------------------------------------------

    #[test]
    fn test_model_profile_defaults() {
        let p = ModelProfile::default();
        assert_eq!(p.name, "phi-4");
        assert_eq!(p.host, "127.0.0.1");
        assert_eq!(p.port, 9120);
        assert_eq!(p.max_tokens, 256);
        assert_eq!(p.temperature, 0.1);
    }

    #[test]
    fn test_model_profile_urls() {
        let p = ModelProfile::default();
        assert_eq!(p.backend_url(), "http://127.0.0.1:9120/v1/chat/completions");
        assert_eq!(p.health_url(), "http://127.0.0.1:9120/health");
    }

    // ------------------------------------------------------------------
    // Classification parsing tests
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_valid_classification() {
        let raw = r#"{"affect_primary":"frustrated","affect_valence":-0.6,"affect_arousal":0.8,"intent_category":"venting","risk_level":"low","anonymized_summary":"User is expressing frustration about a technical issue at work."}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Frustrated);
        assert_eq!(result.affect_valence, -0.6);
        assert_eq!(result.affect_arousal, 0.8);
        assert_eq!(result.intent_category, IntentCategory::Venting);
        assert_eq!(result.risk_level, RiskLevel::Low);
        assert!(result.anonymized_summary.contains("technical issue"));
    }

    #[test]
    fn test_parse_with_markdown_fences() {
        let raw = "```json\n{\"affect_primary\":\"anxious\",\"affect_valence\":-0.4,\"affect_arousal\":0.7,\"intent_category\":\"emotional_support\",\"risk_level\":\"none\",\"anonymized_summary\":\"User is feeling anxious about an upcoming event.\"}\n```";
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Anxious);
        assert_eq!(result.intent_category, IntentCategory::EmotionalSupport);
    }

    #[test]
    fn test_parse_invalid_json_returns_error() {
        let result = LlamaCppAdapter::parse_classification("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_field_uses_fallback() {
        let raw = r#"{"affect_primary":"unknown_mood","affect_valence":0.0,"affect_arousal":0.5,"intent_category":"unknown_intent","risk_level":"unknown_risk","anonymized_summary":"Test."}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Unknown);
        assert_eq!(result.intent_category, IntentCategory::Unknown);
        assert_eq!(result.risk_level, RiskLevel::Unknown);
    }

    #[test]
    fn test_parse_clamps_values() {
        let raw = r#"{"affect_primary":"calm","affect_valence":100.0,"affect_arousal":-5.0,"intent_category":"reflection","risk_level":"none","anonymized_summary":"Test."}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_valence, 1.0); // Clamped
        assert_eq!(result.affect_arousal, 0.0); // Clamped
    }

    #[test]
    fn test_parse_all_emotional_variants() {
        let variants = [
            ("neutral", AffectPrimary::Neutral),
            ("calm", AffectPrimary::Calm),
            ("curious", AffectPrimary::Curious),
            ("pleased", AffectPrimary::Pleased),
            ("hopeful", AffectPrimary::Hopeful),
            ("confused", AffectPrimary::Confused),
            ("uncertain", AffectPrimary::Uncertain),
            ("frustrated", AffectPrimary::Frustrated),
            ("angry", AffectPrimary::Angry),
            ("sad", AffectPrimary::Sad),
            ("anxious", AffectPrimary::Anxious),
            ("overwhelmed", AffectPrimary::Overwhelmed),
            ("fatigued", AffectPrimary::Fatigued),
            ("embarrassed", AffectPrimary::Embarrassed),
            ("lonely", AffectPrimary::Lonely),
            ("excited", AffectPrimary::Excited),
            ("urgent", AffectPrimary::Urgent),
        ];
        for (input, expected) in &variants {
            let raw = format!(r#"{{"affect_primary":"{}","affect_valence":0.0,"affect_arousal":0.5,"intent_category":"clarification","risk_level":"none","anonymized_summary":"Test."}}"#, input);
            let result = LlamaCppAdapter::parse_classification(&raw).unwrap();
            assert_eq!(result.affect_primary, *expected, "Mismatch for input '{}'", input);
        }
    }

    #[test]
    fn test_parse_all_intent_variants() {
        let variants = [
            ("practical_guidance", IntentCategory::PracticalGuidance),
            ("emotional_support", IntentCategory::EmotionalSupport),
            ("decision_support", IntentCategory::DecisionSupport),
            ("venting", IntentCategory::Venting),
            ("planning", IntentCategory::Planning),
            ("clarification", IntentCategory::Clarification),
            ("conflict_navigation", IntentCategory::ConflictNavigation),
            ("reflection", IntentCategory::Reflection),
            ("task_execution", IntentCategory::TaskExecution),
            ("creative_help", IntentCategory::CreativeHelp),
            ("technical_help", IntentCategory::TechnicalHelp),
            ("safety_related", IntentCategory::SafetyRelated),
        ];
        for (input, expected) in &variants {
            let raw = format!(r#"{{"affect_primary":"neutral","affect_valence":0.0,"affect_arousal":0.5,"intent_category":"{}","risk_level":"none","anonymized_summary":"Test."}}"#, input);
            let result = LlamaCppAdapter::parse_classification(&raw).unwrap();
            assert_eq!(result.intent_category, *expected, "Mismatch for input '{}'", input);
        }
    }

    #[test]
    fn test_parse_all_risk_variants() {
        let variants = [
            ("none", RiskLevel::None),
            ("low", RiskLevel::Low),
            ("medium", RiskLevel::Medium),
            ("high", RiskLevel::High),
            ("crisis", RiskLevel::Crisis),
        ];
        for (input, expected) in &variants {
            let raw = format!(r#"{{"affect_primary":"neutral","affect_valence":0.0,"affect_arousal":0.5,"intent_category":"reflection","risk_level":"{}","anonymized_summary":"Test."}}"#, input);
            let result = LlamaCppAdapter::parse_classification(&raw).unwrap();
            assert_eq!(result.risk_level, *expected, "Mismatch for input '{}'", input);
        }
    }

    // ------------------------------------------------------------------
    // MockAdapter tests
    // ------------------------------------------------------------------

    #[test]
    fn test_mock_adapter_returns_default() {
        let adapter = MockAdapter::new();
        let result = adapter.classify("Hello", "test-1").unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Curious);
        assert_eq!(result.intent_category, IntentCategory::Clarification);
        assert_eq!(result.risk_level, RiskLevel::None);
        assert_eq!(result.processing_time_ms, 5);
    }

    #[test]
    fn test_mock_adapter_fail_mode() {
        let mut adapter = MockAdapter::new();
        adapter.fail_on_call = true;
        let result = adapter.classify("Hello", "test-2");
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::InferenceTimeout => {} // expected
            other => panic!("Expected InferenceTimeout, got: {:?}", other),
        }
    }

    #[test]
    fn test_mock_adapter_custom_result() {
        let custom = InferenceResult {
            raw_output: String::new(),
            affect_primary: AffectPrimary::Angry,
            affect_valence: -0.8,
            affect_arousal: 0.9,
            intent_category: IntentCategory::SafetyRelated,
            risk_level: RiskLevel::High,
            anonymized_summary: "User is expressing anger.".to_string(),
            processing_time_ms: 0,
        };
        let mut adapter = MockAdapter::new();
        adapter.custom_result = Some(custom.clone());
        let result = adapter.classify("I'm very angry!", "test-3").unwrap();
        assert_eq!(result.affect_primary, custom.affect_primary);
        assert_eq!(result.intent_category, custom.intent_category);
        assert_eq!(result.risk_level, custom.risk_level);
    }

    #[test]
    fn test_mock_adapter_health_check() {
        let adapter = MockAdapter::new();
        assert!(adapter.health_check().unwrap());
        assert_eq!(adapter.status(), "mock adapter (healthy)");
    }

    #[test]
    fn test_mock_adapter_failed_health_check() {
        let mut adapter = MockAdapter::new();
        adapter.fail_on_call = true;
        assert!(!adapter.health_check().unwrap());
        assert_eq!(adapter.status(), "mock adapter (failed)");
    }
}
