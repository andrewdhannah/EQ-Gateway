# Rust Engine Architecture — The Guard

**Status:** Updated — Implementation Complete  
**Version:** 0.2  
**Date:** June 15, 2026  
**Layer:** Engine Layer (The Guard)  

---

## 1. Philosophy

The Rust Engine is the **only component** that touches raw, unredacted user text. Its job is to:

1. **Ingest** raw text from the mobile UI layer.
2. **Execute** the SLM inference to produce the EQ State JSON.
3. **Sanitize** — strip PII, apply privacy tiers.
4. **Wipe** — zero the raw text buffers from memory before returning.
5. **Yield** — hand only the validated EQ State payload to the KMP logic layer.

The entire component is built around a single invariant:

> **No raw user text ever crosses the FFI boundary into the KMP/UI layer.**

---

## 2. Crate Structure

```
crates/
├── eq-engine-core/          # Core inference orchestration
│   ├── src/
│   │   ├── lib.rs           # Public API (ffi-safe entry points)
│   │   ├── pipeline.rs      # The capture → infer → redact → wipe pipeline
│   │   ├── session.rs       # Ephemeral session management
│   │   └── config.rs        # Model path, device config, tier settings
│   ├── Cargo.toml
│   └── build.rs             # Links to llama.cpp or MLC-LLM
│
├── eq-privacy-filter/       # PII detection and redaction
│   ├── src/
│   │   ├── lib.rs
│   │   ├── patterns.rs      # Canadian PII patterns (SIN, health card, etc.)
│   │   ├── tier_rules.rs    # Privacy tier enforcement logic
│   │   └── redact.rs        # String masking and replacement engine
│   ├── Cargo.toml
│   └── tests/
│       └── pii_corpus.rs    # Test suite against known PII patterns
│
├── eq-memory/               # Secure memory management
│   ├── src/
│   │   ├── lib.rs
│   │   ├── secure_buffer.rs # Heap-allocated buffer that autowipes on drop
│   │   ├── zeroize.rs       # Explicit memory zeroing (uses zeroize crate)
│   │   └── auditor.rs       # Logs memory operations (no data, only metadata)
│   ├── Cargo.toml
│   └── tests/
│       └── wipe_verification.rs
│
├── eq-state-compiler/       # EQ State JSON construction
│   ├── src/
│   │   ├── lib.rs
│   │   ├── affect.rs        # Map SLM logits to AffectPrimary enum
│   │   ├── intent.rs        # Map SLM logits to IntentCategory enum
│   │   ├── risk.rs          # Risk level computation from signals
│   │   ├── privacy.rs       # Privacy state metadata construction
│   │   ├── response.rs      # Response policy derivation
│   │   └── context.rs       # Anonymized summary assembly
│   ├── Cargo.toml
│   └── tests/
│       └── compiler_tests.rs
│
└── eq-ffi/                  # UniFFI bindings for KMP/Swift
    ├── src/
    │   ├── lib.rs           # #[uniffi::export] entry points
    │   └── types.rs         # FFI-safe structs mirroring EQ State schema
    ├── Cargo.toml
    ├── uniffi.toml          # UniFFI config (kotlin/swift generation)
    └── src/eq-gateway.udl   # UniFFI definition file (if using UDL)
```

---

## 3. The Secure Pipeline (Core Data Flow)

```
  User Text (String)
       │
       ▼
┌──────────────────────────────────┐
│  eq-memory::secure_buffer        │
│  ┌────────────────────────────┐  │
│  │ Raw text loaded into       │  │
│  │ heap-allocated, locked,    │  │
│  │ non-swappable memory       │  │
│  └────────────┬───────────────┘  │
└───────────────┼──────────────────┘
                │
                ▼
┌──────────────────────────────────┐
│  eq-engine-core::pipeline        │
│  ┌────────────────────────────┐  │
│  │ 1. Tokenize (local         │  │
│  │    tokenizer, NO network)  │  │
│  │ 2. SLM Inference (1B-3B    │  │
│  │    quantized model)        │  │
│  │ 3. Logit → classification  │  │
│  └────────────┬───────────────┘  │
└───────────────┼──────────────────┘
                │
                ▼
┌──────────────────────────────────┐
│  eq-privacy-filter               │
│  ┌────────────────────────────┐  │
│  │ 1. PII scan (regex + model │  │
│  │    -assisted detection)    │  │
│  │ 2. Redact matched patterns │  │
│  │ 3. Tier enforcement        │  │
│  │    (Tier 0-3 escalation)   │  │
│  └────────────┬───────────────┘  │
└───────────────┼──────────────────┘
                │
                ▼
┌──────────────────────────────────┐
│  eq-state-compiler                │
│  ┌────────────────────────────┐  │
│  │ Assembles validated EQ     │  │
│  │ State JSON from:           │  │
│  │  - AffectState             │  │
│  │  - IntentState             │  │
│  │  - RiskState               │  │
│  │  - PrivacyState            │  │
│  │  - ResponsePolicy          │  │
│  │  - ContextState            │  │
│  └────────────┬───────────────┘  │
└───────────────┼──────────────────┘
                │
                ▼
┌──────────────────────────────────┐
│  eq-memory::secure_buffer        │
│  ┌────────────────────────────┐  │
│  │ ZEROIZE: Overwrite raw     │  │
│  │ text buffer with zeros,    │  │
│  │ then deallocate.           │  │
│  │ Log: "Buffer wiped:        │  │
│  │   session_id=xxx,          │  │
│  │   size=NNN bytes"          │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────┐
│  eq-ffi                           │
│  ┌────────────────────────────┐  │
│  │ EQ State JSON →            │  │
│  │ UniFFI-generated struct    │  │
│  │ → Kotlin/Swift caller      │  │
│  │                            │  │
│  │ NO RAW TEXT CROSSES        │  │
│  │ THIS BOUNDARY.             │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

---

## 4. Secure Buffer Design

This is the most critical component for privacy guarantees.

```rust
// eq-memory/src/secure_buffer.rs

/// A heap-allocated buffer for sensitive string data.
///
/// Guarantees:
/// 1. Memory is zeroed on `drop` (compile-time enforced via `Zeroize`).
/// 2. Memory is mlocked to prevent swapping to disk (OS-dependent).
/// 3. No interior `Clone` — only move semantics.
/// 4. Length and capacity are also zeroed on drop.
pub struct SecureBuffer {
    /// The raw byte storage for sensitive text.
    /// Access is tightly controlled through `with_raw` closures.
    data: Vec<u8>,
}

impl SecureBuffer {
    /// Creates a new secure buffer from a string.
    /// The input string's heap is also overwritten after copy.
    pub fn from_string(mut source: String) -> Self { ... }

    /// Provides temporary, scoped access to the raw bytes.
    /// The closure MUST NOT leak the reference outside its scope.
    pub fn with_raw<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    { ... }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // 1. Overwrite data bytes with zeros
        self.data.as_mut_slice().zeroize();
        // 2. Overwrite length and capacity
        self.data.zeroize();
        // 3. Log the wipe event (metadata only, no content)
        log::info!("SecureBuffer wiped: {} bytes", self.data.len());
    }
}
```

### Key Properties:

| Property | Implementation |
|:---|:---|
| **No Clone** | `SecureBuffer` does not implement `Clone`. Move semantics only. |
| **Bounds Checked** | All slice access uses safe Rust — no raw pointer deref. |
| **Swap Protected** | On Android: `mlock()` via `libc` binding (behind `#[cfg(target_os = "android")]`). |
| **Audit Trail** | `auditor.rs` logs creation, access, and wipe events (timestamps + sizes only — NO content). |

---

## 5. Inference Runtime Integration

### 5.1 Backend Abstraction — SLMAdapter Trait

The inference backend is abstracted behind the `SLMAdapter` trait, currently implemented by `LlamaCppAdapter` (HTTP client to `llama-server-mini`) and `MockAdapter` (deterministic test double).

```rust
// eq-engine-core/src/slm_adapter.rs

/// Abstraction over SLM inference backends.
/// Currently backed by llama-server-mini HTTP API (TcpStream-based).
/// Future: MLC-LLM, Android NNAPI, Core ML.
pub trait SLMAdapter {
    /// Classify user input and return structured inference result.
    /// Opens a fresh TCP connection per call (Connection: close).
    fn classify(&self, text: &str, session_id: &str) -> Result<InferenceResult, EngineError>;

    /// Quick health check against the backend.
    fn health_check(&self) -> Result<bool, EngineError>;

    /// Human-readable status string (for diagnostics).
    fn status(&self) -> String;
}
```

### 5.2 LlamaCppAdapter — TcpStream HTTP/1.1 Client

The `LlamaCppAdapter` connects to `llama-server-mini` via **direct TcpStream HTTP/1.1** (replaced ureq 3.3.0 due to a connection abort bug against llama-server-mini). Key design:

- **Fresh TCP connection per request** with `Connection: close` — avoids context overflow
- **JSON-envelope prompt format** — wraps user text in `{"instruction","schema","user_message"}` JSON to force Phi-4 into structured output reliably
- **Dual-schema parsing** — handles both flat (`affect_primary`) and nested (`affect.primary`) output schemas with model vocabulary mapping and substring fallback

```rust
pub struct LlamaCppAdapter {
    profile: ModelProfile,
}

pub struct ModelProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub max_tokens: u32,
    pub temperature: f64,
}
```

### 5.3 Live Backend

- **Model:** Phi-4-mini-instruct Q4_K_M (4.55 GB)  
- **Server:** `llama-server-mini` on `127.0.0.1:9120`  
- **Router:** llama.cpp router on `127.0.0.1:8080`  
- **Inference latency:** ~2.4s per request on RX 570 (fresh TCP connection per call)  
- **Parser reliability:** 5/5 benchmark iterations succeed; handles markdown fences, missing fields, substring variant matching

### 5.4 Quantization Requirements

Model weights should be pre-quantized before bundling:

| Parameter | Target |
|:---|:---|
| **Weight quantization** | 4-bit GGUF or MLC format |
| **KV cache quantization** | INT8 or FP8 (required for 4K-8K context) |
| **Model size** | 1B-3B parameters (targeting 500MB-1.5GB on disk) |
| **Inference target** | < 500ms per EQ State generation on Snapdragon 8 Elite |

---

## 6. FFI Boundary (UniFFI)

### 6.1 Exported Functions

```rust
// eq-ffi/src/lib.rs

/// Initialize the EQ Engine with a model path and device configuration.
/// Must be called once at app startup.
#[uniffi::export]
fn initialize(config: EngineConfig) -> Result<(), EngineError>;

/// Process raw user text through the full pipeline:
///   1. Load into SecureBuffer
///   2. Run SLM inference
///   3. Apply privacy filters
///   4. Compile EQ State
///   5. Wipe SecureBuffer
///   6. Return EQ State JSON (string)
///
/// Raw text NEVER leaves this function — only the EQ State crosses the FFI.
#[uniffi::export]
fn process_user_input(
    raw_text: String,
    session_id: String,
    detail_level: String,  // "minimal" | "standard" | "extended"
) -> Result<String, EngineError>;

/// Trigger a secure memory wipe of all active session buffers.
/// Called when the app backgrounds or the session ends.
#[uniffi::export]
fn wipe_all_sessions() -> Result<(), EngineError>;

/// Get the current engine health/status (for diagnostics dashboard).
#[uniffi::export]
fn engine_status() -> EngineStatus;
```

### 6.2 FFI-Safe Types

```rust
// eq-ffi/src/types.rs

/// FFI-safe version of the EQ State.
/// All fields are plain Rust types that UniFFI can translate
/// to Kotlin data classes / Swift structs automatically.
#[derive(uniffi::Record)]
pub struct FfiEQState {
    pub schema_version: String,
    pub session_id: String,
    pub captured_at: String,          // ISO-8601
    pub affect_primary: String,       // AffectPrimary enum variant name
    pub affect_valence: f64,
    pub affect_arousal: f64,
    pub affect_confidence: f64,
    pub intent_category: String,      // IntentCategory enum variant name
    pub intent_confidence: f64,
    pub risk_level: String,           // RiskLevel enum variant name
    pub risk_confidence: f64,
    pub risk_signals: Vec<String>,
    pub privacy_sensitivity: String,  // PrivacySensitivity enum variant name
    pub privacy_pii_removed: bool,
    pub privacy_redaction_confidence: f64,
    pub response_tone: String,        // ResponseTone enum variant name
    pub response_warmth: f64,
    pub response_directness: f64,
    pub context_summary: String,
}
```

---

## 7. Error Handling Strategy

| Error Variant | Description | Recovery |
|:---|:---|:---|
| `ModelNotFound` | Model file missing at configured path | App shows download/retry UI |
| `InferenceTimeout` | SLM inference exceeded timeout | Retry with reduced context |
| `MemoryLockFailed` | `mlock()` failed on Android | Log warning, continue without |
| `PiiScanOverflow` | PII scan exceeded complexity budget | Fall back to regex-only mode |
| `BufferPoisoned` | SecureBuffer integrity check failed | Hard crash (panic) — data cannot be trusted |

---

## 8. Testing Strategy

### 8.1 Unit Tests (cargo test — 78 passing)

| Crate | Test Count | Test Focus |
|:---|:---:|:---|
| `eq-memory` | 11 | Buffer creation, zeroize verification, drop semantics, no-clone compile check |
| `eq-privacy-filter` | 16 | PII patterns (Canadian SIN, passport, email, health card, etc.), redaction |
| `eq-state-compiler` | 11 | All 46 enum variants, serialization, JSON output format |
| `eq-engine-core` | 32 | Pipeline orchestration, SLM adapter (flat+nested parsing, vocabulary mapping) |
| `eq-ffi` | 8 | FFI bridge, process_user_input via UniFFI, wipe sessions |
| **Integration** | 4 (ignored*) | Live backend health check, classify, status + 5-iteration benchmark |

*Integration tests require a running llama-server-mini instance and are `#[ignore]`d in CI.

### 8.2 Integration Tests (Live Backend)

```rust
// tests/live_backend_test.rs

/// Tests against a running llama-server-mini (127.0.0.1:9120 preferred).
/// Auto-discovers backend via TCP connect probe; skips if unreachable.
#[test]
#[ignore = "requires running llama.cpp backend"]
fn backend_health_check() { ... }

#[test]
#[ignore = "requires running llama.cpp backend"]
fn backend_classify_returns_response() {
    // "I'm really frustrated with this bug" → affect=Frustrated, intent=TechnicalHelp, risk=Low
    // Inference time: ~2.4s on RX 570
}

#[test]
#[ignore = "requires running llama.cpp backend"]
fn backend_status_reports_connection() { ... }

// tests/inference_benchmark.rs — 5 iterations, mean 2473ms, median 2466ms
#[test]
#[ignore = "requires running llama.cpp backend"]
fn inference_latency_benchmark() { ... }
```

### 8.3 Security Audit Checklist

- [ ] `SecureBuffer::from_string()` leaves no copies on the caller's heap.
- [ ] `Drop` is called deterministically (no `mem::forget` on SecureBuffer).
- [ ] No `unsafe` code in `eq-memory` except the `mlock` syscall.
- [ ] UniFFI bindings do not expose raw text fields.
- [ ] All async operations use bounded thread pools (no unbounded spawn).

---

## 9. Build & Deployment

### 9.1 Android Integration

```
eq-gateway/
├── rust/                  # This entire Rust workspace
│   ├── Cargo.toml
│   └── crates/
├── android/               # Android Gradle project
│   ├── app/
│   │   └── src/main/
│   │       ├── kotlin/    # Jetpack Compose UI + KMP logic
│   │       └── jniLibs/   # Prebuilt .so (arm64-v8a, armeabi-v7a)
│   ├── build.gradle.kts   # Includes cargo-ndk build step
│   └── rust-bridge/       # UniFFI-generated Kotlin bindings
```

### 9.2 iOS Integration

```
eq-gateway/
├── rust/
├── ios/
│   ├── EQGateway.xcodeproj
│   ├── EQGateway/
│   │   └── RustBridge/    # UniFFI-generated Swift bindings
│   └── Frameworks/
│       └── libeq_ffi.a    # Prebuilt static lib (arm64)
```

### 9.3 CI/CD Requirements

- **Build step**: `cargo ndk` for Android targets + `cargo lipo` (via `lipo`) for iOS universal binary.
- **Test step**: `cargo test` on all crates + integration test suite.
- **Security step**: Automated memory leak scan via `valgrind` or `address-sanitizer` on CI.

---

## 10. Key Dependencies (Cargo.toml)

```toml
[workspace]
members = [
    "crates/eq-engine-core",
    "crates/eq-privacy-filter",
    "crates/eq-memory",
    "crates/eq-state-compiler",
    "crates/eq-ffi",
]

# eq-memory
[dependencies]
zeroize = { version = "1", features = ["zeroize_derive"] }
libc = "0.2"           # For mlock/munlock on Android
tracing = "0.1"        # Structured logging (no data, only metadata)

# eq-privacy-filter
[dependencies]
regex = "1"            # Primary PII pattern matching
serde_json = "1"       # For parsing model-assisted PII signals

# eq-engine-core
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"       # JSON parsing for SLM output + HTTP response handling
# HTTP client: direct TcpStream HTTP/1.1 (replaced ureq 3.3.0)
# Inference: llama-server-mini HTTP API at 127.0.0.1:9120

# eq-ffi
[dependencies]
uniffi = { version = "0.31", features = ["cli"] }
thiserror = "2"        # Error type derivation for FFI safety
```

---

*Confidential — EQ Gateway Project*
