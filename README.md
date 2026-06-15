# EQ Gateway

**Privacy-preserving emotional context middleware for AI systems.**

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-dea584)](https://www.rust-lang.org)
[![Android](https://img.shields.io/badge/Android-SDK%2036-3ddc84)](https://developer.android.com)
[![Python](https://img.shields.io/badge/Python-3.11%2B-3776A9)](https://python.org)
[![License](https://img.shields.io/badge/License-MIT-blue)](#license)

---

## Overview

EQ Gateway is an on-device privacy layer for emotionally intelligent AI. It uses a small language model (SLM) running locally on a mobile device to infer a user's mood, intent, risk state, and privacy sensitivity — then emits a **constrained, anonymized, structured metadata payload** called **EQ State**. Cloud AI receives only this metadata, never the raw user text.

### The Core Thesis

> Keep the emotionally sensitive layer local. Let larger AI reason over anonymized emotional metadata rather than raw private context.

### Key Differentiator

EQ Gateway is not an AI assistant — it is a **privacy firewall for AI interactions**. It competes on trust, not intelligence. For users subject to privacy regulations (PIPEDA, PHIPA, GDPR) and increasingly wary of data extraction, this is a defensible market position.

---

## Architecture

```
┌────────────────────────────────────────────────┐
│                 Mobile Device                    │
│                                                  │
│  ┌──────────┐   ┌──────────────┐   ┌─────────┐ │
│  │  Native   │   │  Rust Engine │   │  llama  │ │
│  │    UI     │──▶│   (Secure    │──▶│ .cpp /  │ │
│  │(Compose / │   │   Memory,    │   │  SLM    │ │
│  │  SwiftUI) │   │    PII,      │   │         │ │
│  └──────────┘   │  Orchestrate) │   └────┬────┘ │
│                 └──────┬────────┘        │      │
│                        │                 │      │
│                        ▼                 │      │
│                 ┌──────────────┐         │      │
│                 │  EQ State    │◀────────┘      │
│                 │   Compiler   │                │
│                 └──────┬───────┘                │
│                        │                        │
│                        ▼                        │
│                 ┌──────────────┐                │
│                 │  MCP Bridge  │                │
│                 │  (KMP Logic) │                │
│                 └──────┬───────┘                │
└────────────────────────┼────────────────────────┘
                         │ EQ State (JSON) — no raw text
                         ▼
                 ┌──────────────┐
                 │   Cloud AI   │
                 │  (Adapted    │
                 │   Response)  │
                 └──────────────┘
```

### Layer Stack

| Layer | Technology | Role |
|:---|:---|:---|
| **UI** | Jetpack Compose (Android) / SwiftUI (iOS) | User interaction |
| **Logic** | Kotlin Multiplatform (KMP) | Librarian, privacy rules, EQ State compiler |
| **Engine** | Rust | Secure memory, PII detection, orchestration |
| **Inference** | llama.cpp / MLC-LLM | SLM mathematical kernels (GPU/NPU) |
| **Research** | Python | Model tuning, quantization, evaluation |
| **Contract** | TypeScript / JSON Schema | Schema definition & MCP simulation |

---

## Project Structure

```
EQ-Gateway/
├── crates/                          # Rust workspace (engine crates)
│   ├── eq-memory/                   #   SecureBuffer: zeroize-on-drop + mlock
│   ├── eq-privacy-filter/           #   PII detection (Canadian patterns)
│   ├── eq-state-compiler/           #   EQState struct (mirrors Zod schema)
│   ├── eq-engine-core/              #   Pipeline orchestration
│   └── eq-ffi/                      #   UniFFI FFI bridge → Kotlin
│
├── android/                         # Android project (Jetpack Compose)
│   └── app/
│       ├── src/main/java/
│       │   ├── com/example/eqgateway/   # App code
│       │   └── uniffi/eq_ffi/           # Generated Kotlin bindings
│       └── src/main/jniLibs/            # Cross-compiled .so files
│           ├── arm64-v8a/
│           ├── armeabi-v7a/
│           ├── x86_64/
│           └── x86/
│
├── evaluation/                      # Python evaluation harness
│   ├── schema/eq_state_schema.json  #   JSON Schema (derived from Zod)
│   ├── fixtures/                    #   Test fixtures (valid + invalid)
│   └── tests/                       #   19 tests, all passing
│
├── docs/                            # Design documentation
│   ├── PRD_v0.2.md                  #   Product Requirements Document
│   ├── Design_Document_v0.2.md      #   Full architecture design
│   ├── Sprint_1_Plan.md             #   Sprint 1 detailed plan
│   └── schemas/                     #   TypeScript Zod schemas
│
├── schemas/                         # TypeScript/JSON schema sources
├── Cargo.toml                       # Rust workspace manifest
└── .cargo/config.toml               # Android NDK cross-compilation config
```

---

## Prerequisites

### Rust

| Tool | Version | Purpose |
|:---|:---:|:---|
| Rust toolchain | 1.85+ | Engine development |
| `uniffi-bindgen` | 0.31+ | Kotlin binding generation |

```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android
```

### Android

| Tool | Version | Location |
|:---|:---:|:---|
| JDK | 17 (Zulu) | `C:\Program Files\Zulu\zulu-17\` |
| Android SDK | 36 | `C:\Android\` |
| Build Tools | 36.0.0 | `C:\Android\build-tools\` |
| NDK | 27.0.12077973 | `C:\Android\ndk\` |

### Python (evaluation)

```bash
cd evaluation
pip install -r requirements.txt
```

---

## Building

### Rust (Desktop)

```bash
cargo build
cargo test
```

All 5 crates compile with zero warnings. **78 Rust tests pass** (up from 19).

### Android APK

```bash
cd android
./gradlew assembleDebug
```

**36 tasks, BUILD SUCCESSFUL.** Produces `app/build/outputs/apk/debug/app-debug.apk`.

### Cross-Compile for Android (all 4 ABIs)

```bash
# Build native libraries
cargo build -p eq-ffi --target aarch64-linux-android
cargo build -p eq-ffi --target armv7-linux-androideabi
cargo build -p eq-ffi --target x86_64-linux-android
cargo build -p eq-ffi --target i686-linux-android

# Copy to jniLibs (automated via script)
```

### Regenerate Kotlin Bindings

```bash
uniffi-bindgen generate --library target/debug/eq_ffi.dll \
  --language kotlin --out-dir android/app/src/main/java
```

### Python Evaluation

```bash
cd evaluation
pytest -v
```

**19 Python tests pass** — JSON Schema validation against the Zod contract with both valid and intentionally invalid fixtures.

---

## EQ State Payload

The EQ State is the core output — a structured JSON object capturing emotional, intent, risk, and privacy metadata. No raw user text is ever included.

```json
{
  "schema_version": "v0.1",
  "session": {
    "ephemeral_session_id": "a1b2c3d4-...",
    "timestamp_local": "2026-06-14T15:30:00",
    "device_processing_only": true
  },
  "affect": {
    "mood": "neutral",
    "valence": 0.2,
    "arousal": -0.1,
    "urgency": "low"
  },
  "intent": {
    "primary": "inquire",
    "secondary": "clarify",
    "task_category": "information_request",
    "confidence": 0.85
  },
  "risk": {
    "self_harm_risk": "none_detected",
    "escalation_risk": "low",
    "distress_level": "low",
    "contains_crisis_keywords": false
  },
  "privacy": {
    "pii_detected": false,
    "redacted_count": 0,
    "redacted_categories": [],
    "contains_sensitive_data": false
  },
  "response_policy": {
    "requires_careful_tone": false,
    "suggested_tone": "neutral",
    "recommended_action": "answer_directly",
    "requires_human_escalation": false
  },
  "context": {
    "turn_count": 5,
    "session_duration_seconds": 120,
    "input_length_chars": 87
  }
}
```

Full schema: [`evaluation/schema/eq_state_schema.json`](evaluation/schema/eq_state_schema.json) and [`schemas/eq-state.schema.ts`](schemas/eq-state.schema.ts) (TypeScript Zod source).

---

## Privacy & Security

EQ Gateway's security model is enforced at the architecture level:

| Mechanism | Implementation | Status |
|:---|:---|:---:|
| **Zeroize-on-drop** | `SecureBuffer` uses `zeroize` crate + `std::mem::forget` guard | ✅ |
| **Memory locking** | `mlock` / `VirtualLock` prevents secrets from being paged to disk | ✅ |
| **No-Clone buffers** | `SecureBuffer` explicitly forbids `Clone` and `Copy` | ✅ |
| **PII redaction** | Canadian patterns: SIN, passport, email, phone, health card, postal code | ✅ |
| **On-device only** | EQ State payload declares `device_processing_only: true` | ✅ |
| **All-or-nothing wipe** | `wipe_all_sessions()` FFI call clears all ephemeral state | ✅ |
| **MCP bridge** | Structured metadata only — no raw text crosses the device boundary | 🚧 |

---

## Sprint 1 Progress

| Ticket | Description | Status |
|:---|:---|:---:|
| S1-001 | Android project scaffold (Compose + AGP 9.0.1) | ✅ |
| S1-002 | Rust workspace (5 crates, 78 tests) | ✅ |
| S1-003 | UniFFI bridge + .so cross-compilation | ✅ |
| S1-004 | Python evaluation harness | ✅ |
| S1-005 | GitHub Actions CI | ✅ |
| S1-006 | SecureBuffer with zeroize | ✅ |
| S1-007 | mlock/munlock support | ✅ |
| S1-008 | Canadian PII patterns | ✅ |
| S1-009 | PiiScanner::scan() | ✅ |
| S1-010 | Security audit tests | ✅ |
| S1-011 | SLMAdapter trait (classify, health_check, status) | ✅ |
| S1-012 | JSON-envelope prompt format for structured output | ✅ |
| S1-013 | Phi-4-mini-instruct integration via llama-server-mini | ✅ |
| S1-014 | Dual-schema parser (flat + nested) with vocabulary mapping | ✅ |
| S1-015 | End-to-end live inference verified (~2.4s on RX 570) | ✅ |
| S1-016 | Inference benchmark (5/5 success, mean 2473ms) | ✅ |

---

## License

MIT — see [LICENSE](LICENSE) (to be added).

---

## Documentation

Full design documentation is available in the [`docs/`](docs/) directory:

| Document | Description |
|:---|:---|
| [PRD v0.2](docs/PRD_v0.2.md) | Product Requirements Document |
| [Design Document v0.2](docs/Design_Document_v0.2.md) | Full architecture and design |
| [Sprint 1 Plan](docs/Sprint_1_Plan.md) | Detailed sprint breakdown |
| [Market Research](docs/Market_Research_v0.1.md) | Competitive landscape & positioning |
| [MCP Data Flow](docs/MCP_Data_Flow.md) | Security model & data flow |
| [EQ State Schema (TS)](schemas/eq-state.schema.ts) | TypeScript Zod contract |
| [EQ State Schema (JSON)](evaluation/schema/eq_state_schema.json) | JSON Schema derived from Zod |
