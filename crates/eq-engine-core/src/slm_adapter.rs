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
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

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
const CLASSIFICATION_SYSTEM_PROMPT: &str = r#"You are EQ Gateway, an emotional context classifier. Respond with ONLY a JSON object."#;

/// JSON envelope template for user messages.
/// The model follows this format reliably — wraps instruction, schema example, and user text in a JSON object.
const USER_MESSAGE_TEMPLATE: &str = r#"{"instruction":"Classify the emotional context of this message. Respond with ONLY a JSON object following the schema. No conversation, no explanation, no markdown.","schema":{"affect_primary":"frustrated","affect_valence":-0.7,"affect_arousal":0.8,"intent_category":"technical_help","risk_level":"low","anonymized_summary":"User is frustrated with a build system bug."},"user_message":"__USER_TEXT__"}"#;

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
}

impl LlamaCppAdapter {
    /// Create a new adapter for a llama.cpp backend.
    pub fn new(profile: ModelProfile) -> Self {
        Self { profile }
    }

    /// Parse the SLM's JSON output into an InferenceResult.
    ///
    /// Handles two schema formats:
    /// - **Flat** (our target): `{"affect_primary":"...", "intent_category":"...", ...}`
    /// - **Nested** (Phi-4 model output): `{"affect":{"primary":"...", ...}, "intent":{"category":"...", ...}, ...}`
    ///
    /// Also maps model-specific vocabulary to our canonical enum values
    /// (e.g., `"frustration"` → `"frustrated"`, `"seeking_guidance"` → `"practical_guidance"`).
    fn parse_classification(raw: &str) -> Result<InferenceResult, EngineError> {
        // Extract JSON from the response (handle possible markdown fences / leading text)
        let json_str = if let Some(start) = raw.find('{') {
            if let Some(end) = raw.rfind('}') {
                &raw[start..=end]
            } else {
                raw
            }
        } else {
            raw
        };

        // Parse as generic JSON value first
        let root: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| EngineError::ConfigError(format!(
                "SLM output parse failed: {} — raw: {}", e, raw
            )))?;

        // Helper: extract a string from a JSON value, checking nested paths
        let get_str = |value: &serde_json::Value, flat_key: &str, nested_parent: &str, nested_key: &str| -> Option<String> {
            // Try flat first
            if let Some(s) = value.get(flat_key).and_then(|v| v.as_str()) {
                return Some(s.to_string());
            }
            // Try nested
            if let Some(obj) = value.get(nested_parent) {
                if let Some(s) = obj.get(nested_key).and_then(|v| v.as_str()) {
                    return Some(s.to_string());
                }
            }
            None
        };

        let get_f64 = |value: &serde_json::Value, flat_key: &str, nested_parent: &str, nested_key: &str| -> Option<f64> {
            // Try flat first
            if let Some(n) = value.get(flat_key).and_then(|v| v.as_f64()) {
                return Some(n);
            }
            // Try nested
            if let Some(obj) = value.get(nested_parent) {
                if let Some(n) = obj.get(nested_key).and_then(|v| v.as_f64()) {
                    return Some(n);
                }
            }
            None
        };

        // Extract raw text fields
        let affect_primary_raw = get_str(&root, "affect_primary", "affect", "primary")
            .unwrap_or_default();
        let intent_category_raw = get_str(&root, "intent_category", "intent", "category")
            .unwrap_or_default();
        let risk_level_raw = get_str(&root, "risk_level", "risk", "level")
            .unwrap_or_default();
        let anonymized_summary = get_str(&root, "anonymized_summary", "context", "anonymized_summary")
            .unwrap_or_default();

        let affect_valence = get_f64(&root, "affect_valence", "affect", "valence")
            .unwrap_or(0.0);
        let affect_arousal = get_f64(&root, "affect_arousal", "affect", "arousal")
            .unwrap_or(0.5);

        // Map affect primary (handle model vocabulary like "frustration" → "frustrated")
        let affect_primary = match affect_primary_raw.to_lowercase().as_str() {
            // Canonical values
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
            // Model vocabulary mappings (nested format)
            "frustration" | "frustrate" => AffectPrimary::Frustrated,
            "anger" => AffectPrimary::Angry,
            "anxiety" | "nervousness" | "nervous" => AffectPrimary::Anxious,
            "sadness" | "sorrow" => AffectPrimary::Sad,
            "confusion" => AffectPrimary::Confused,
            "uncertainty" => AffectPrimary::Uncertain,
            "curiosity" => AffectPrimary::Curious,
            "excitement" => AffectPrimary::Excited,
            "disappointment" | "disappointed" => AffectPrimary::Sad,
            "hopefulness" => AffectPrimary::Hopeful,
            "overwhelm" | "overwhelming" => AffectPrimary::Overwhelmed,
            "fatigue" | "tiredness" | "tired" => AffectPrimary::Fatigued,
            "calmness" => AffectPrimary::Calm,
            _ => {
                // Try substring matching for composite descriptions
                let lower = affect_primary_raw.to_lowercase();
                if lower.contains("frustrat") { AffectPrimary::Frustrated }
                else if lower.contains("angr") || lower.contains("anger") { AffectPrimary::Angry }
                else if lower.contains("anxi") || lower.contains("nervous") || lower.contains("worry") { AffectPrimary::Anxious }
                else if lower.contains("sad") || lower.contains("disappoint") { AffectPrimary::Sad }
                else if lower.contains("confus") || lower.contains("unsure") { AffectPrimary::Confused }
                else if lower.contains("curious") || lower.contains("interest") { AffectPrimary::Curious }
                else if lower.contains("calm") || lower.contains("peace") { AffectPrimary::Calm }
                else if lower.contains("pleased") || lower.contains("happy") || lower.contains("glad") { AffectPrimary::Pleased }
                else if lower.contains("hope") { AffectPrimary::Hopeful }
                else if lower.contains("overwhelm") { AffectPrimary::Overwhelmed }
                else if lower.contains("fatigue") || lower.contains("tired") || lower.contains("exhaust") { AffectPrimary::Fatigued }
                else if lower.contains("embarrass") || lower.contains("ashamed") { AffectPrimary::Embarrassed }
                else if lower.contains("lonely") || lower.contains("alone") || lower.contains("isolat") { AffectPrimary::Lonely }
                else if lower.contains("excit") || lower.contains("thrill") { AffectPrimary::Excited }
                else if lower.contains("urgent") || lower.contains("emergency") || lower.contains("immediate") { AffectPrimary::Urgent }
                else { AffectPrimary::Unknown }
            }
        };

        // Map intent category (handle model vocabulary)
        let intent_category = match intent_category_raw.to_lowercase().as_str() {
            // Canonical values
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
            // Model vocabulary mappings (nested format)
            "seeking_guidance" | "seeking_help" | "seeking_advice" => IntentCategory::PracticalGuidance,
            "seeking_support" | "seeking_comfort" | "seeking_reassurance" => IntentCategory::EmotionalSupport,
            "issue_reporting" | "reporting_issue" | "technical_issue" | "technical_difficulty" => IntentCategory::TechnicalHelp,
            "seeking_information" | "information_request" | "asking_question" => IntentCategory::Clarification,
            "seeking_clarification" | "request_clarification" => IntentCategory::Clarification,
            "expressing_concern" | "safety_concern" | "risk_report" => IntentCategory::SafetyRelated,
            "decision_making" | "weighing_options" => IntentCategory::DecisionSupport,
            "sharing_feelings" | "emotional_expression" | "expressing_emotion" => IntentCategory::Venting,
            "reflective" | "self_reflection" | "introspection" => IntentCategory::Reflection,
            "planning_ahead" | "making_plans" | "organizing" => IntentCategory::Planning,
            _ => {
                let lower = intent_category_raw.to_lowercase();
                if lower.contains("technical") || lower.contains("bug") || lower.contains("error") || lower.contains("compile") { IntentCategory::TechnicalHelp }
                else if lower.contains("guidance") || lower.contains("advice") || lower.contains("how to") { IntentCategory::PracticalGuidance }
                else if lower.contains("support") || lower.contains("comfort") || lower.contains("reassur") { IntentCategory::EmotionalSupport }
                else if lower.contains("question") || lower.contains("clarif") || lower.contains("explain") || lower.contains("information") { IntentCategory::Clarification }
                else if lower.contains("safety") || lower.contains("concern") || lower.contains("risk") || lower.contains("crisis") { IntentCategory::SafetyRelated }
                else if lower.contains("decision") || lower.contains("choose") || lower.contains("option") { IntentCategory::DecisionSupport }
                else if lower.contains("venting") || lower.contains("feeling") || lower.contains("emotion") || lower.contains("frustrat") { IntentCategory::Venting }
                else if lower.contains("reflect") || lower.contains("think") { IntentCategory::Reflection }
                else if lower.contains("plan") || lower.contains("organiz") { IntentCategory::Planning }
                else if lower.contains("task") || lower.contains("execute") { IntentCategory::TaskExecution }
                else if lower.contains("creativ") || lower.contains("brainstorm") { IntentCategory::CreativeHelp }
                else if lower.contains("help") || lower.contains("assist") { IntentCategory::PracticalGuidance }
                else { IntentCategory::Unknown }
            }
        };

        // Map risk level
        let risk_level = match risk_level_raw.to_lowercase().as_str() {
            "none" => RiskLevel::None,
            "low" => RiskLevel::Low,
            "medium" => RiskLevel::Medium,
            "high" => RiskLevel::High,
            "crisis" => RiskLevel::Crisis,
            _ => {
                let lower = risk_level_raw.to_lowercase();
                if lower.contains("high") || lower.contains("elevated") || lower.contains("severe") || lower.contains("very") { RiskLevel::High }
                else if lower.contains("crisis") || lower.contains("critical") || lower.contains("emergency") { RiskLevel::Crisis }
                else if lower.contains("medium") || lower.contains("moderate") { RiskLevel::Medium }
                else if lower.contains("none") || lower.contains("no risk") || lower.contains("low") { RiskLevel::Low }
                else { RiskLevel::Unknown }
            }
        };

        Ok(InferenceResult {
            raw_output: raw.to_string(),
            affect_primary,
            affect_valence: affect_valence.clamp(-1.0, 1.0),
            affect_arousal: affect_arousal.clamp(0.0, 1.0),
            intent_category,
            risk_level,
            anonymized_summary,
            processing_time_ms: 0, // Set by caller
        })
    }
}

impl SLMAdapter for LlamaCppAdapter {
    fn classify(&self, text: &str, _session_id: &str) -> Result<InferenceResult, EngineError> {
        let start = Instant::now();

        // Sanitize user text for JSON embedding (escape backslashes and quotes)
        let sanitized = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");

        // Use JSON envelope format — Phi-4 follows this reliably
        let user_content = USER_MESSAGE_TEMPLATE.replace("__USER_TEXT__", &sanitized);

        let request = ChatRequest {
            model: self.profile.name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: CLASSIFICATION_SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user_content,
                },
            ],
            max_tokens: self.profile.max_tokens,
            temperature: self.profile.temperature,
            stream: false,
        };

        let request_body = serde_json::to_string(&request)
            .map_err(|e| EngineError::ConfigError(format!("serialize request: {}", e)))?;

        // Build and send HTTP/1.1 request via TcpStream
        let addr = format!("{}:{}", self.profile.host, self.profile.port);
        let timeout = Duration::from_millis(self.profile.timeout_ms);
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| EngineError::ConfigError(format!("connect to {}: {}", addr, e)))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| EngineError::ConfigError(format!("set timeout: {}", e)))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|e| EngineError::ConfigError(format!("set timeout: {}", e)))?;

        let http_request = format!(
            "POST /v1/chat/completions HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            addr,
            request_body.len(),
            request_body
        );

        stream
            .write_all(http_request.as_bytes())
            .map_err(|e| EngineError::ConfigError(format!("write request: {}", e)))?;

        // Read response
        let mut resp_buf = Vec::new();
        stream
            .read_to_end(&mut resp_buf)
            .map_err(|e| EngineError::ConfigError(format!("read response: {}", e)))?;

        // Parse HTTP response
        let resp_str = String::from_utf8_lossy(&resp_buf);
        let parts: Vec<&str> = resp_str.splitn(2, "\r\n\r\n").collect();
        if parts.len() < 2 {
            return Err(EngineError::ConfigError(format!(
                "malformed HTTP response: {}",
                &resp_str.chars().take(200).collect::<String>()
            )));
        }

        let _headers = parts[0];
        let resp_body = parts[1];

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
        let addr = format!("{}:{}", self.profile.host, self.profile.port);
        let timeout = Duration::from_millis(5000);
        match TcpStream::connect_timeout(
            &addr
                .parse::<std::net::SocketAddr>()
                .map_err(|e| EngineError::ConfigError(format!("invalid address {}: {}", addr, e)))?,
            timeout,
        ) {
            Ok(mut stream) => {
                // Send a simple GET /health request
                let req = format!(
                    "GET /health HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                    addr
                );
                let _ = stream.write_all(req.as_bytes());
                let mut buf = [0u8; 512];
                match stream.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let resp = String::from_utf8_lossy(&buf[..n]);
                        Ok(resp.contains("200 OK") || resp.contains("\"status\": \"ok\""))
                    }
                    _ => Ok(false),
                }
            }
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
    // Nested schema parsing tests (model's actual output format)
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_nested_schema_model_output() {
        // Full nested output as produced by Phi-4-mini-instruct
        let raw = r#"{
 "schema_version": "0.1",
 "session": { "ephemeral_session_id": "6e7d9f0c-9f1d-4a2b-9f7a-8a6d9f9b2e4a", "timestamp_local": "2023-04-05T14:40:22Z", "device_processing_only": true },
 "affect": { "primary": "frustration", "secondary": ["anger"], "valence": -0.6, "arousal": 0.7, "confidence": 0.85, "evidence_type": "semantic_inference" },
 "intent": { "category": "seeking_guidance", "subtype": "technical_support", "confidence": 0.95 },
 "risk": { "level": "low", "signals": ["technical difficulties"], "confidence": 0.75, "requires_local_escalation": false },
 "privacy": { "sensitivity_level": "low", "raw_text_shared": false, "pii_removed": true, "sensitive_domains_detected": [], "redaction_confidence": 1.0 },
 "response_policy": { "tone": "neutral", "warmth": 0.5, "directness": 0.8, "length": "medium", "pace": "normal", "max_followup_questions": 2, "format": "prose" },
 "context": { "anonymized_summary": "User is expressing frustration and anger due to a persistent bug in the build system.", "included_raw_excerpt": false, "retrieval_notes_included": false }
}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        // affect.primary: "frustration" → Frustrated
        assert_eq!(result.affect_primary, AffectPrimary::Frustrated, "nested frustration→Frustrated");
        assert_eq!(result.affect_valence, -0.6);
        assert_eq!(result.affect_arousal, 0.7);
        // intent.category: "seeking_guidance" → PracticalGuidance
        assert_eq!(result.intent_category, IntentCategory::PracticalGuidance, "nested seeking_guidance→PracticalGuidance");
        // risk.level: "low" → Low
        assert_eq!(result.risk_level, RiskLevel::Low);
        // context.anonymized_summary
        assert!(result.anonymized_summary.contains("frustration and anger"));
    }

    #[test]
    fn test_parse_nested_model_vocabulary() {
        // Test various model-specific vocabulary mappings
        let cases = vec![
            (r#"{"affect":{"primary":"anger"},"intent":{"category":"issue_reporting"},"risk":{"level":"high"},"context":{"anonymized_summary":"Test."},"affect_valence":-0.8,"affect_arousal":0.9}"#,
             AffectPrimary::Angry, IntentCategory::TechnicalHelp, RiskLevel::High),
            (r#"{"affect":{"primary":"anxiety"},"intent":{"category":"seeking_support"},"risk":{"level":"medium"},"context":{"anonymized_summary":"Test."},"affect_valence":-0.5,"affect_arousal":0.8}"#,
             AffectPrimary::Anxious, IntentCategory::EmotionalSupport, RiskLevel::Medium),
            (r#"{"affect":{"primary":"sadness"},"intent":{"category":"venting"},"risk":{"level":"none"},"context":{"anonymized_summary":"Test."},"affect_valence":-0.7,"affect_arousal":0.3}"#,
             AffectPrimary::Sad, IntentCategory::Venting, RiskLevel::None),
            (r#"{"affect":{"primary":"curiosity"},"intent":{"category":"seeking_information"},"risk":{"level":"low"},"context":{"anonymized_summary":"Test."},"affect_valence":0.3,"affect_arousal":0.4}"#,
             AffectPrimary::Curious, IntentCategory::Clarification, RiskLevel::Low),
            (r#"{"affect":{"primary":"confusion"},"intent":{"category":"clarification"},"risk":{"level":"none"},"context":{"anonymized_summary":"Test."},"affect_valence":-0.1,"affect_arousal":0.5}"#,
             AffectPrimary::Confused, IntentCategory::Clarification, RiskLevel::None),
        ];
        for (i, (raw, expected_affect, expected_intent, expected_risk)) in cases.iter().enumerate() {
            let result = LlamaCppAdapter::parse_classification(raw).unwrap();
            assert_eq!(result.affect_primary, *expected_affect, "Case {}: affect mismatch for {}", i, raw);
            assert_eq!(result.intent_category, *expected_intent, "Case {}: intent mismatch for {}", i, raw);
            assert_eq!(result.risk_level, *expected_risk, "Case {}: risk mismatch for {}", i, raw);
        }
    }

    #[test]
    fn test_parse_nested_missing_fields_fallback() {
        // Nested with missing fields should use defaults
        let raw = r#"{"affect":{"primary":"unknown"},"intent":{"category":"unknown"},"risk":{"level":"none"},"context":{"anonymized_summary":"Test."}}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Unknown);
        assert_eq!(result.intent_category, IntentCategory::Unknown);
        assert_eq!(result.risk_level, RiskLevel::None);
        assert_eq!(result.affect_valence, 0.0); // default
        assert_eq!(result.affect_arousal, 0.5); // default
    }

    #[test]
    fn test_parse_flat_takes_priority_over_nested() {
        // When both flat and nested keys exist, flat should win
        let raw = r#"{"affect_primary":"calm","affect":{"primary":"frustration"},"affect_valence":0.2,"affect_arousal":0.1,"intent_category":"reflection","intent":{"category":"seeking_guidance"},"risk_level":"none","risk":{"level":"high"},"anonymized_summary":"Flat wins."}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Calm, "flat should win over nested");
        assert_eq!(result.intent_category, IntentCategory::Reflection, "flat should win over nested");
        assert_eq!(result.risk_level, RiskLevel::None, "flat should win over nested");
        assert_eq!(result.anonymized_summary, "Flat wins.");
    }

    #[test]
    fn test_parse_substring_matching() {
        // Fallback substring matching for composite descriptions
        let raw = r#"{"affect_primary":"feeling very frustrated and annoyed","affect_valence":-0.5,"affect_arousal":0.6,"intent_category":"needs technical help with build","risk_level":"pretty high honestly","anonymized_summary":"Test."}"#;
        let result = LlamaCppAdapter::parse_classification(raw).unwrap();
        assert_eq!(result.affect_primary, AffectPrimary::Frustrated, "should match 'frustrat' substring");
        assert_eq!(result.intent_category, IntentCategory::TechnicalHelp, "should match 'technical' substring");
        assert_eq!(result.risk_level, RiskLevel::High, "should match 'high' substring");
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
