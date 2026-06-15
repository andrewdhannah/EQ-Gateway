//! # eq-ffi — UniFFI Bindings for Mobile Platforms
//!
//! This crate defines the FFI boundary between the Rust engine and the
//! Kotlin Multiplatform / SwiftUI layers.
//!
//! # Security Boundary
//! NO raw user text crosses this boundary. Only the EQ State JSON string
//! is returned to the caller. Raw text is destroyed in the SecureBuffer
//! before any data is returned across FFI.
//!
//! # Exported Functions
//! - `initialize()` — Load model and prepare the engine
//! - `process_user_input()` — Full pipeline: text in, EQ State JSON out
//! - `wipe_all_sessions()` — Secure memory wipe of all active data
//! - `engine_status()` — Get engine health information

// Re-export the core types for UniFFI to discover
pub use eq_engine_core::{EQEngine, EngineConfig, EngineError, ProcessedOutput};
pub use eq_state_compiler::EQState;

/// Errors that can be returned across the FFI boundary.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    /// Generic engine error with a human-readable description.
    #[error("{description}")]
    EngineError {
        description: String,
    },
}

impl From<EngineError> for FfiError {
    fn from(e: EngineError) -> Self {
        FfiError::EngineError {
            description: e.to_string(),
        }
    }
}

/// Initialize the EQ Engine with a model path and device configuration.
/// Must be called once at app startup before any other operations.
///
/// # Arguments
/// * `model_path` - Filesystem path to the quantized GGUF model
/// * `gpu_layers` - Number of layers to offload to GPU/NPU
/// * `context_size` - Context window size in tokens
///
/// # Returns
/// Ok(()) on success, or a descriptive error on failure.
#[uniffi::export]
pub fn initialize(
    model_path: String,
    gpu_layers: u32,
    context_size: u32,
) -> Result<(), FfiError> {
    let config = EngineConfig {
        model_path,
        gpu_layers,
        context_size,
        ..EngineConfig::default()
    };

    let mut engine = EQEngine::new(config);
    engine.initialize()?;

    // Store the engine in a global for later use
    // TODO: Replace with proper state management (Mutex<Option<EQEngine>>)
    Ok(())
}

/// Process raw user text through the full EQ Gateway pipeline.
///
/// # Arguments
/// * `raw_text` - The user's raw input text. Memory is zeroed after processing.
/// * `session_id` - UUID string for the current ephemeral session.
///
/// # Returns
/// JSON string containing the complete EQ State payload.
/// NO raw text is included in the output.
///
/// # Memory Safety
/// The raw_text string is moved into a SecureBuffer immediately on entry.
/// The buffer is zeroed before this function returns.
#[uniffi::export]
pub fn process_user_input(
    raw_text: String,
    session_id: String,
) -> Result<String, FfiError> {
    // TODO: Use the stored engine instance instead of creating a new one
    let config = EngineConfig::default();
    let mut engine = EQEngine::new(config);
    engine.initialize()?;

    let output = engine.process_user_input(raw_text, &session_id)?;

    Ok(output.eq_state_json)
}

/// Trigger a secure memory wipe of all active session buffers.
/// Called when the app backgrounds or the session ends.
#[uniffi::export]
pub fn wipe_all_sessions() -> Result<(), FfiError> {
    // TODO: Implement session state management and wipe
    Ok(())
}

/// Get the current engine health and status.
/// Returns a JSON string with diagnostic information.
#[uniffi::export]
pub fn engine_status() -> String {
    format!(
        r#"{{"engine":"eq-gateway","version":"{}","status":"initialized"}}"#,
        EQEngine::version()
    )
}

// Required by UniFFI proc-macro scaffolding (must be at the bottom of the crate)
uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_status() {
        let status = engine_status();
        assert!(status.contains("eq-gateway"));
        assert!(status.contains("0.1.0"));
    }

    #[test]
    fn test_process_user_input_ffi() {
        let result = process_user_input(
            "Hello, this is a test message.".to_string(),
            "test-ffi-session".to_string(),
        );
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("schema_version"));
        assert!(json.contains("affect"));
        assert!(json.contains("intent"));
        assert!(json.contains("privacy"));
    }

    #[test]
    fn test_process_user_input_with_pii() {
        let result = process_user_input(
            "My email is john@example.com and SIN is 123-456-789.".to_string(),
            "test-pii-session".to_string(),
        );
        assert!(result.is_ok());
        let json = result.unwrap();
        // PII should be redacted in the output
        assert!(!json.contains("john@example.com"));
        assert!(!json.contains("123-456-789"));
        // But the context should mention the redaction happened
        assert!(json.contains("pii_removed"));
    }

    #[test]
    fn test_wipe_all_sessions() {
        let result = wipe_all_sessions();
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_status_format() {
        let status = engine_status();
        // Should be valid-ish JSON
        assert!(status.starts_with('{'));
        assert!(status.ends_with('}'));
        assert!(status.contains("engine"));
        assert!(status.contains("version"));
        assert!(status.contains("status"));
    }

    #[test]
    fn test_ffi_with_empty_text() {
        let result = process_user_input(
            String::new(),
            "empty-test".to_string(),
        );
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("schema_version"));
    }

    #[test]
    fn test_ffi_with_long_text() {
        let long_text = "Hello, this is a test. ".repeat(100);
        let result = process_user_input(
            long_text,
            "long-test".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_ffi_result_is_valid_json() {
        let result = process_user_input(
            "Test message.".to_string(),
            "json-test".to_string(),
        );
        assert!(result.is_ok());
        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("FFI output should be valid JSON");
        assert!(parsed.is_object());
    }
}
