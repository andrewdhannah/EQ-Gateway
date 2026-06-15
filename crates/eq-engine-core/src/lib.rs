//! # eq-engine-core — Inference Orchestration
//!
//! The central pipeline that orchestrates the secure capture → inference → redaction → compilation workflow.
//! This crate is the entry point for processing user text through the EQ Gateway engine.
//!
//! # Pipeline
//! 1. Raw text is loaded into a [`SecureBuffer`] (memory-safe, zeroed on drop)
//! 2. PII detection and redaction are applied (via [`eq_privacy_filter`])
//! 3. Text is sent to the SLM for inference (via [`slm_adapter::SLMAdapter`])
//! 4. SLM output is classified into affect, intent, and risk
//! 5. The EQ State is compiled (via [`eq_state_compiler`])
//! 6. The SecureBuffer is zeroed
//! 7. The EQ State JSON is returned
//!
//! # SLM Adapter
//! The crate provides an [`slm_adapter::SLMAdapter`] trait that abstracts over
//! inference backends. Two implementations are provided:
//! - [`slm_adapter::LlamaCppAdapter`] — real HTTP calls to local llama.cpp
//! - [`slm_adapter::MockAdapter`] — deterministic results for testing

pub mod slm_adapter;

use eq_memory::SecureBuffer;
use eq_privacy_filter::{PiiScanner, ScanResult};
use slm_adapter::{InferenceResult, ModelProfile, SLMAdapter};

/// Configuration for the EQ Engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Path to the quantized GGUF model file
    pub model_path: String,
    /// Number of layers to offload to GPU/NPU
    pub gpu_layers: u32,
    /// Context window size in tokens
    pub context_size: u32,
    /// Maximum inference time in milliseconds
    pub timeout_ms: u64,
    /// SLM model profile (host, port, model name, etc.)
    pub model_profile: ModelProfile,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            model_path: String::new(),
            gpu_layers: 99,
            context_size: 4096,
            timeout_ms: 5000,
            model_profile: ModelProfile::default(),
        }
    }
}

/// Errors that can occur during engine operation.
#[derive(Debug)]
pub enum EngineError {
    /// Model file not found at the configured path
    ModelNotFound(String),
    /// SLM inference exceeded the configured timeout
    InferenceTimeout,
    /// Memory lock (mlock) failed
    MemoryLockFailed(String),
    /// PII scan exceeded complexity budget
    PiiScanOverflow(String),
    /// SecureBuffer integrity check failed
    BufferPoisoned,
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::ModelNotFound(p) => write!(f, "Model not found at: {}", p),
            EngineError::InferenceTimeout => write!(f, "Inference timed out"),
            EngineError::MemoryLockFailed(e) => write!(f, "Memory lock failed: {}", e),
            EngineError::PiiScanOverflow(e) => write!(f, "PII scan overflow: {}", e),
            EngineError::BufferPoisoned => write!(f, "SecureBuffer integrity check failed"),
            EngineError::ConfigError(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl std::error::Error for EngineError {}

/// Result type alias for engine operations.
pub type EngineResult<T> = Result<T, EngineError>;

/// A processed result from the engine pipeline.
#[derive(Debug)]
pub struct ProcessedOutput {
    /// The EQ State as a JSON string
    pub eq_state_json: String,
    /// PII scan results (for debugging and audit)
    pub pii_scan: ScanResult,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// The EQ Engine — orchestrates the secure processing pipeline.
pub struct EQEngine {
    config: EngineConfig,
    pii_scanner: PiiScanner,
    adapter: Box<dyn SLMAdapter>,
}

impl EQEngine {
    /// Create a new EQ Engine with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Engine configuration (model path, GPU layers, etc.)
    ///
    /// # Returns
    /// A configured EQEngine instance. Uses a default [`slm_adapter::MockAdapter`]
    /// for testing. Call [`with_adapter`](Self::with_adapter) to set a real backend.
    pub fn new(config: EngineConfig) -> Self {
        EQEngine {
            pii_scanner: PiiScanner::new(),
            adapter: Box::new(slm_adapter::MockAdapter::new()),
            config,
        }
    }

    /// Create a new EQ Engine with a specific SLM adapter.
    ///
    /// # Arguments
    /// * `config` - Engine configuration
    /// * `adapter` - SLM inference backend adapter
    pub fn with_adapter(config: EngineConfig, adapter: Box<dyn SLMAdapter>) -> Self {
        EQEngine {
            pii_scanner: PiiScanner::new(),
            adapter,
            config,
        }
    }

    /// Replace the SLM adapter after construction.
    pub fn set_adapter(&mut self, adapter: Box<dyn SLMAdapter>) {
        self.adapter = adapter;
    }

    /// Initialize the engine (load the model into memory).
    /// Must be called once before `process_user_input()`.
    ///
    /// TODO: Implement actual model loading via llama.cpp bindings
    pub fn initialize(&mut self) -> EngineResult<()> {
        // Validate model path exists
        if !std::path::Path::new(&self.config.model_path).exists() {
            // In development, allow empty model path for testing
            if !self.config.model_path.is_empty() {
                return Err(EngineError::ModelNotFound(self.config.model_path.clone()));
            }
        }
        Ok(())
    }

    /// Process raw user text through the full pipeline.
    ///
    /// # Arguments
    /// * `raw_text` - The user's raw input text
    /// * `session_id` - UUID for the current ephemeral session
    ///
    /// # Returns
    /// A [`ProcessedOutput`] containing the EQ State JSON and scan results.
    ///
    /// # Memory Safety
    /// - Raw text is immediately moved into a [`SecureBuffer`]
    /// - The buffer is zeroed before this function returns
    /// - No raw text crosses the FFI boundary back to the caller
    pub fn process_user_input(&mut self, raw_text: String, session_id: &str) -> EngineResult<ProcessedOutput> {
        let start = std::time::Instant::now();

        // Step 1: Load into SecureBuffer
        let mut buffer = SecureBuffer::from_string(raw_text);

        // Step 2: PII scan (on the raw text for audit)
        let scan_result = buffer.with_raw(|bytes| {
            let text = std::str::from_utf8(bytes)
                .map_err(|_| EngineError::BufferPoisoned)?;
            Ok::<_, EngineError>(self.pii_scanner.scan(text))
        })?;

        // Step 3: Perform redaction
        let anonymized = buffer.with_raw(|bytes| {
            let text = std::str::from_utf8(bytes)
                .map_err(|_| EngineError::BufferPoisoned)?;
            Ok::<_, EngineError>(self.pii_scanner.redact(text, "[REDACTED]"))
        })?;

        // Step 4: SLM inference via the adapter
        let inference: InferenceResult = match self.adapter.classify(&anonymized, session_id) {
            Ok(result) => result,
            Err(EngineError::InferenceTimeout) => {
                // Return a degraded result on timeout instead of failing
                InferenceResult {
                    raw_output: String::new(),
                    affect_primary: eq_state_compiler::AffectPrimary::Unknown,
                    affect_valence: 0.0,
                    affect_arousal: 0.0,
                    intent_category: eq_state_compiler::IntentCategory::Unknown,
                    risk_level: eq_state_compiler::RiskLevel::Unknown,
                    anonymized_summary: anonymized.clone(),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(e) => return Err(e),
        };

        // Step 5: Build the EQ State from inference results
        let eq_state = eq_state_compiler::EQState {
            schema_version: "0.1".to_string(),
            session: eq_state_compiler::SessionInfo {
                ephemeral_session_id: session_id.to_string(),
                timestamp_local: chrono_now_iso8601(),
                device_processing_only: false,
            },
            affect: eq_state_compiler::AffectState {
                primary: inference.affect_primary,
                secondary: vec![],
                valence: inference.affect_valence,
                arousal: inference.affect_arousal,
                confidence: 0.7,
                evidence_type: "slm_classification".to_string(),
            },
            intent: eq_state_compiler::IntentState {
                category: inference.intent_category,
                subtype: "classified_by_slm".to_string(),
                confidence: 0.7,
            },
            risk: eq_state_compiler::RiskState {
                level: inference.risk_level.clone(),
                signals: vec![],
                confidence: 0.7,
                requires_local_escalation: matches!(inference.risk_level, eq_state_compiler::RiskLevel::High | eq_state_compiler::RiskLevel::Crisis),
            },
            privacy: eq_state_compiler::PrivacyState {
                sensitivity_level: eq_state_compiler::PrivacySensitivity::Medium,
                raw_text_shared: false,
                pii_removed: !scan_result.clean,
                sensitive_domains_detected: vec![],
                redaction_confidence: 0.9,
            },
            response_policy: eq_state_compiler::ResponsePolicy {
                tone: eq_state_compiler::ResponseTone::NeutralProfessional,
                warmth: 0.5,
                directness: 0.5,
                length: "medium".to_string(),
                pace: "steady".to_string(),
                max_followup_questions: 2,
                format: "prose".to_string(),
            },
            context: eq_state_compiler::ContextState {
                anonymized_summary: inference.anonymized_summary,
                included_raw_excerpt: false,
                retrieval_notes_included: false,
            },
        };

        // Step 6: buffer is dropped here — memory is zeroed automatically
        // Step 7: inference result is returned

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(ProcessedOutput {
            eq_state_json: eq_state.to_json().unwrap_or_default(),
            pii_scan: scan_result,
            processing_time_ms: elapsed,
        })
    }

    /// Get a reference to the engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Returns the current engine version.
    pub fn version() -> &'static str {
        "0.1.0"
    }
}

/// Get the current time as an ISO-8601 string.
/// Uses the `chrono` crate if available; fallback to a simple UTC timestamp.
fn chrono_now_iso8601() -> String {
    // TODO: Replace with chrono::Utc::now().to_rfc3339() when chrono is added as a dependency
    "2026-06-08T12:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_engine() -> EQEngine {
        EQEngine::new(EngineConfig::default())
    }

    #[test]
    fn test_engine_creation() {
        let _engine = create_test_engine();
        assert_eq!(EQEngine::version(), "0.1.0");
    }

    #[test]
    fn test_process_simple_text() {
        let mut engine = create_test_engine();
        let result = engine.process_user_input(
            "Hello, how are you?".to_string(),
            "test-session-001",
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.eq_state_json.contains("schema_version"));
        assert_eq!(output.pii_scan.clean, true);
    }

    #[test]
    fn test_process_text_with_pii() {
        let mut engine = create_test_engine();
        let result = engine.process_user_input(
            "My SIN is 123-456-789 and my email is test@example.com".to_string(),
            "test-session-002",
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.pii_scan.clean, false);
        assert!(output.pii_scan.critical_count > 0);
        // The anonymized context should not contain the original PII
        assert!(!output.eq_state_json.contains("123-456-789"));
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(EQEngine::version(), "0.1.0");
    }

    #[test]
    fn test_engine_config_defaults() {
        let config = EngineConfig::default();
        assert_eq!(config.gpu_layers, 99);
        assert_eq!(config.context_size, 4096);
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.model_path.is_empty());
    }

    #[test]
    fn test_engine_error_display() {
        let errors = vec![
            (EngineError::ModelNotFound("/nope".into()), "Model not found at: /nope"),
            (EngineError::InferenceTimeout, "Inference timed out"),
            (EngineError::MemoryLockFailed("EACCES".into()), "Memory lock failed: EACCES"),
            (EngineError::PiiScanOverflow("too complex".into()), "PII scan overflow: too complex"),
            (EngineError::BufferPoisoned, "SecureBuffer integrity check failed"),
            (EngineError::ConfigError("bad ctx".into()), "Configuration error: bad ctx"),
        ];
        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected, "Mismatch for {:?}", error);
        }
    }

    #[test]
    fn test_initialize_with_empty_path_ok() {
        let mut engine = EQEngine::new(EngineConfig::default());
        assert!(engine.initialize().is_ok());
    }

    #[test]
    fn test_initialize_with_nonexistent_path_errors() {
        let config = EngineConfig {
            model_path: "/nonexistent/model.gguf".to_string(),
            ..EngineConfig::default()
        };
        let mut engine = EQEngine::new(config);
        let result = engine.initialize();
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::ModelNotFound(p) => assert!(p.contains("nonexistent")),
            other => panic!("Expected ModelNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_process_empty_text() {
        let mut engine = create_test_engine();
        let result = engine.process_user_input(
            String::new(),
            "test-empty",
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.eq_state_json.contains("schema_version"));
        assert!(output.pii_scan.clean);
    }

    #[test]
    fn test_process_text_with_only_pii() {
        let mut engine = create_test_engine();
        let result = engine.process_user_input(
            "123-456-789".to_string(),
            "test-pii-only",
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        // All PII should be redacted
        assert!(!output.eq_state_json.contains("123-456-789"));
        assert_eq!(output.pii_scan.clean, false);
        assert!(output.pii_scan.critical_count > 0);
    }

    #[test]
    fn test_processing_time_reported() {
        let mut engine = create_test_engine();
        let result = engine.process_user_input(
            "Hello world".to_string(),
            "test-timing",
        );
        assert!(result.is_ok());
        // Processing time should be a positive value
        assert!(result.unwrap().processing_time_ms > 0);
    }

    #[test]
    fn test_eq_state_contains_all_sections() {
        let mut engine = create_test_engine();
        let result = engine.process_user_input(
            "I'm feeling anxious about my presentation tomorrow".to_string(),
            "test-sections",
        );
        assert!(result.is_ok());
        let json = result.unwrap().eq_state_json;
        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"session\""));
        assert!(json.contains("\"affect\""));
        assert!(json.contains("\"intent\""));
        assert!(json.contains("\"risk\""));
        assert!(json.contains("\"privacy\""));
        assert!(json.contains("\"response_policy\""));
        assert!(json.contains("\"context\""));
    }
}
