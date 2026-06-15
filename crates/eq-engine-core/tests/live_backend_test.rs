//! # Live Backend Integration Tests
//!
//! These tests verify that the [`LlamaCppAdapter`] can communicate with
//! a real llama.cpp backend over HTTP.
//!
//! ## Prerequisites
//!
//! The backend (or router) must be running. By default, tests attempt to
//! reach the backend at `127.0.0.1:9120`.
//!
//! ## Running
//!
//! ```bash
//! # Run only if the backend is reachable (auto-detected)
//! cargo test -p eq-engine-core --test live_backend_test -- --ignored
//!
//! # Force run even if auto-detect says unreachable
//! export EQ_SKIP_BACKEND_CHECK=1
//! cargo test -p eq-engine-core --test live_backend_test -- --ignored
//! ```
//!
//! ## What is tested
//!
//! 1. **Health check** — `/health` endpoint returns 200
//! 2. **Chat completion** — `POST /v1/chat/completions` returns valid JSON
//! 3. **Inference latency** — time measurement for a real inference call
//!
//! ## Known issues
//!
//! The Phi-4-mini-instruct model has a strong output prior that produces a
//! nested EQ schema format (with `affect.primary`, `intent.category`, etc.)
//! rather than the flat `affect_primary`, `intent_category` schema our
//! `parse_classification` expects. These tests verify the HTTP transport
//! layer; schema alignment is a separate prompt-engineering task.

use eq_engine_core::slm_adapter::{ModelProfile, LlamaCppAdapter, SLMAdapter};
use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr};
use std::time::Duration;

// ============================================================================
// Backend Discovery
// ============================================================================

/// Try to discover a running backend. Returns (host, port) or None.
///
/// Checks, in order:
/// 1. `EQ_BACKEND_URL` environment variable
/// 2. Router at `127.0.0.1:8080`
/// 3. Direct backend at `127.0.0.1:9120`
fn discover_backend() -> Option<(String, u16)> {
    let timeout = Duration::from_secs(2);

    // Helper: try to open a TCP connection to the given addr
    let can_connect = |host: &str, port: u16| -> bool {
        let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
            Ok(a) => a,
            Err(_) => return false,
        };
        TcpStream::connect_timeout(&addr, timeout).is_ok()
    };

    // Check env var first
    if let Ok(url) = std::env::var("EQ_BACKEND_URL") {
        let url = url.trim().to_lowercase();
        let url = url.strip_prefix("http://").unwrap_or(&url);
        if let Some((host, port_str)) = url.split_once(':') {
            if let Ok(port) = port_str.parse::<u16>() {
                if can_connect(host, port) {
                    // Verify it responds to /health using a raw HTTP request
                    let health_addr: SocketAddr = match format!("{}:{}", host, port).parse() {
                        Ok(a) => a,
                        Err(_) => return None,
                    };
                    if let Ok(mut stream) = TcpStream::connect_timeout(&health_addr, timeout) {
                        let _ = stream.write_all(b"GET /health HTTP/1.0\r\n\r\n");
                        let mut buf = [0u8; 256];
                        if stream.read(&mut buf).is_ok() {
                            let resp = String::from_utf8_lossy(&buf);
                            if resp.contains("200 OK") || resp.contains("ok") {
                                return Some((host.to_string(), port));
                            }
                        }
                    }
                }
            }
        }
        return None;
    }

    // Try backend directly first (port 9120) — this preserves our system prompt
    if can_connect("127.0.0.1", 9120) {
        return Some(("127.0.0.1".to_string(), 9120));
    }

    // Fall back to router (port 8080) — may replace system prompts
    if can_connect("127.0.0.1", 8080) {
        return Some(("127.0.0.1".to_string(), 8080));
    }

    None
}

/// Returns true if a backend is reachable.
fn backend_is_reachable() -> bool {
    // Allow bypass via env var
    if std::env::var("EQ_FORCE_INTEGRATION").is_ok() {
        return true;
    }
    discover_backend().is_some()
}

// ============================================================================
// Adapter Factory
// ============================================================================

/// Create a `LlamaCppAdapter` pointed at a live backend.
fn create_live_adapter() -> Option<LlamaCppAdapter> {
    let (host, port) = discover_backend()?;
    let profile = ModelProfile {
        host,
        port,
        timeout_ms: 120_000, // Allow generous timeout for first inference
        ..Default::default()
    };
    Some(LlamaCppAdapter::new(profile))
}

// ============================================================================
// Tests
// ============================================================================

/// Verify the backend health check returns OK.
#[test]
#[ignore = "requires running llama.cpp backend"]
fn backend_health_check() {
    if !backend_is_reachable() {
        eprintln!("SKIP: no backend reachable (start the router or llama-server-mini)");
        return;
    }

    let adapter = create_live_adapter().expect("adapter creation should succeed");
    let healthy = adapter
        .health_check()
        .expect("health check should not error");
    assert!(healthy, "backend should report healthy");
}

/// Verify we can send a classification prompt and receive a chat completion response.
///
/// This tests the HTTP transport layer: request/response, JSON parsing, and
/// the backend's ability to produce a valid chat completion.
#[test]
#[ignore = "requires running llama.cpp backend"]
fn backend_classify_returns_response() {
    if !backend_is_reachable() {
        eprintln!("SKIP: no backend reachable (start the router or llama-server-mini)");
        return;
    }

    let adapter = create_live_adapter().expect("adapter creation should succeed");

    // Use the adapter's classify method
    let result = adapter.classify(
        "I'm really frustrated with this bug in the build system. It keeps failing and I can't figure out why.",
        "integration-test-session",
    );

    // The result may be an error if the model output doesn't match our schema,
    // but we should NOT get a transport-level error.
    match result {
        Ok(inference) => {
            // If parsing succeeded, we got a valid classification
            assert!(!inference.raw_output.is_empty(), "should have raw output");
            assert!(inference.processing_time_ms > 0, "should have positive processing time");
            eprintln!(
                "Inference completed in {}ms — affect={:?}, intent={:?}, risk={:?}",
                inference.processing_time_ms,
                inference.affect_primary,
                inference.intent_category,
                inference.risk_level,
            );
        }
        Err(e) => {
            // If parsing failed, at least verify it's a ConfigError (parse issue) 
            // not a transport/connection error
            let err_str = e.to_string();
            assert!(
                err_str.contains("SLM output parse failed")
                    || err_str.contains("parse chat response"),
                "Unexpected error (should be parse error, not connection): {}",
                err_str
            );
            eprintln!("Classification returned parse error (expected — model uses different schema): {}", e);
        }
    }
}

/// Verify the `status()` method works with a live backend.
#[test]
#[ignore = "requires running llama.cpp backend"]
fn backend_status_reports_connection() {
    if !backend_is_reachable() {
        eprintln!("SKIP: no backend reachable (start the router or llama-server-mini)");
        return;
    }

    let adapter = create_live_adapter().expect("adapter creation should succeed");
    let status = adapter.status();
    assert!(
        status.contains("connected") || status.contains("backend unreachable"),
        "status should reflect backend state: {}",
        status
    );
    eprintln!("Backend status: {}", status);
}
