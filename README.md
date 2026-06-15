# EQ Gateway — Local Context Firewall for Agentic AI

**On-device privacy firewall for agentic AI. Converts raw user input into structured EQ State metadata — no raw text leaves the device.**

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-dea584)](https://www.rust-lang.org)
[![Android](https://img.shields.io/badge/Android-SDK%2036-3ddc84)](https://developer.android.com)
[![Python](https://img.shields.io/badge/Python-3.11%2B-3776A9)](https://python.org)
[![License](https://img.shields.io/badge/License-MIT-blue)](#license)

---

## Overview

EQ Gateway is an on-device privacy layer for emotionally intelligent AI. It converts sensitive raw user input into a **constrained, anonymized, structured metadata payload** called **EQ State**. Cloud AI agents receive only this metadata, never the raw user text, ensuring that the most sensitive context never leaves the device.

### The Core Thesis

> Keep the context boundary local. Let larger AI reason over structured EQ State metadata rather than raw private context.

### Key Differentiator

EQ Gateway is not an AI assistant — it is a **privacy firewall for AI interactions**. It computesS, approves, and transports privacy-safe state objects, acting as a secure buffer between the human and the cloud.

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
│       │   ├── com.example.eqgateway/   # App code
│       │   └── uniffi/eq_ffi/           # Generated Kotlin bindings
│       └── src/main/jniLibs/            # Cross-compiled .so files
│
├── services/                        # Agent Infrastructure (Python)
│   ├── agent-gateway/                #  FastAPI Service, Routing, HITL Gate, Receipts
│   └── mcp-bridge/                   #  FastMCP Bridge to Cloud Agents
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
│   ├── Sprint_2_Plan.md             #   Sprint 2 detailed plan
│   └── schemas/                     #   TypeScript Zod schemas
│
├── schemas/                         # TypeScript/JSON schema sources
├── Cargo.toml                       # Rust workspace manifest
└── .cargo/config.toml               # Android NDK cross-compilation config
```

---

## Implementation Status

| Component | Status | Proof of Work |
|:---|:---:|:---|
| **Rust Engine** | ✅ | 5 crates, 78 tests, Zeroize-on-drop, mlock |
| **PII Scanning** | ✅ | Canadian patterns implemented in `eq-privacy-filter` |
| **Android App** | ✅ | Compose UI with Rust FFI integration (debug APK) |
| **EQ State Schema** | ✅ | JSON Schema validated via Python evaluation harness |
| **Agent Gateway** | ✅ | FastAPI, Deterministic Routing, HITL Gate, Audit Receipts |
| **MCP Bridge** | ✅ | FastMCP relay for cloud-agent tool access |
| **Inference** | ✅ | Phi-4-mini-instruct via llama-server-mini |
| **CI/CD** | ✅ | GitHub Actions for Rust build and test |
| **Security Audit** | 🚧 | Manual review of FFI boundaries |

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
| **MCP bridge** | Structured metadata only — no raw text crosses the device boundary | ✅ |

---

## Building

### Rust (Desktop)
```bash
cargo build
cargo test
```
All 5 crates compile with zero warnings. **78 Rust tests pass**.

### Android APK
```bash
cd android
./gradlew assembleDebug
```
**BUILD SUCCESSFUL.** Produces `app/build/outputs/apk/debug/app-debug.apk`.

### Python (Gateway & Bridge)
```bash
# Agent Gateway
cd services/agent-gateway
pip install -r requirements.txt
uvicorn agent_gateway.main:app --reload

# MCP Bridge
cd services/mcp-bridge
pip install -r requirements.txt
python src/mcp_bridge/server.py
```

---

## Related Proofs
This project is part of a broader agentic AI safety architecture:
- [EQ Gateway](https://github.com/andrewdhannah/EQ-Gateway) — Context Boundary / Privacy Firewall (This project)
- [Work Packet Compiler](https://github.com/andrewdhannah/work-packet-compiler) — Action Boundary / Governed Delegation
- [Agentic OS Proof](https://github.com/andrewdhannah/agentic-os-proof) — Integrated Governance Runtime

---

## Documentation
Full design documentation is available in the [`docs/`](docs/) directory:

| Document | Description |
|:---|:---|
| [PRD v0.2](docs/PRD_v0.2.md) | Product Requirements Document |
| [Design Document v0.2](docs/Design_Document_v0.2.md) | Full architecture and design |
| [Sprint 1 Plan](docs/Sprint_1_Plan.md) | Detailed sprint breakdown |
| [Sprint 2 Plan](docs/Sprint_2_Plan.md) | Detailed sprint breakdown |
| [Market Research](docs/Market_Research_v0.1.md) | Competitive landscape & positioning |
| [MCP Data Flow](docs/MCP_Data_Flow.md) | Security model & data flow |
