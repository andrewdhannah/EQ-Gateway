# Product Requirements Document — EQ Gateway

**Document Version:** 0.2  
**Date:** June 08, 2026  
**Status:** Draft  
**Confidentiality:** Confidential — EQ Gateway Project

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Product Vision](#2-product-vision)
3. [Target Market & Personas](#3-target-market--personas)
4. [User Stories](#4-user-stories)
5. [Functional Requirements](#5-functional-requirements)
6. [Non-Functional Requirements](#6-non-functional-requirements)
7. [Technical Architecture](#7-technical-architecture)
8. [Platform Strategy](#8-platform-strategy)
9. [Data Model](#9-data-model)
10. [MCP Interface Specification](#10-mcp-interface-specification)
11. [Privacy & Security Model](#11-privacy--security-model)
12. [Compliance & Regulation](#12-compliance--regulation)
13. [Monetization Strategy](#13-monetization-strategy)
14. [Release Criteria](#14-release-criteria)
15. [Roadmap & Milestones](#15-roadmap--milestones)
16. [Risks & Mitigations](#16-risks--mitigations)
17. [Competitive Landscape](#17-competitive-landscape)
18. [Glossary](#18-glossary)

---

## 1. Executive Summary

**EQ Gateway** is a privacy-preserving emotional context middleware layer for AI systems. It uses a small language model (SLM) running locally on a mobile device to infer a user's emotional state, intent, risk level, and privacy sensitivity. Instead of sending raw personal data to a cloud AI, the local SLM produces a constrained, anonymized, structured **EQ State** payload. This payload is passed via an MCP (Model Context Protocol) bridge to a larger cloud AI, allowing it to adapt its tone and response strategy **without ever accessing the user's private text**.

### The Core Thesis

> Keep the emotionally sensitive layer local. Let larger AI reason over anonymized emotional metadata rather than raw private context.

### Key Differentiator

EQ Gateway is not an AI assistant — it is a **privacy firewall for AI interactions**. It competes on trust, not intelligence. For Canadian users subject to PIPEDA and increasingly wary of data extraction, this is a defensible market position.

---

## 2. Product Vision

### 2.1 Vision Statement

A world where emotionally adaptive AI is **private by default** — where no user ever has to trade their personal data for a more understanding digital assistant.

### 2.2 Mission

To build the standard privacy layer for emotionally intelligent AI, starting with mobile devices and expanding to all surfaces where humans interact with AI.

### 2.3 Core Values

| Value | How It Manifests |
|:---|:---|
| **Privacy by Design** | No raw user text ever leaves the device without explicit user approval |
| **Emotional Sovereignty** | Users own their emotional data; it is never sold, shared, or analyzed externally |
| **Transparency** | Every data flow is visible in the Privacy Dashboard — no hidden telemetry |
| **Accessibility** | The privacy layer works for everyone, not just technical users |

---

## 3. Target Market & Personas

### 3.1 Target Audiences

| Segment | Description | Size Indicator | Willingness to Pay |
|:---|:---|:---:|:---:|
| **Privacy-Conscious Consumers** | Users who avoid cloud AI due to privacy concerns | Large (growing with each data breach) | Medium |
| **Mental Health App Users** | Users who want AI support but fear data exposure | Medium | High |
| **Enterprise Knowledge Workers** | Professionals handling sensitive client data | Medium | Very High |
| **Canadian Healthcare Adjacent** | Providers subject to PHIPA who want AI assistance | Small (regulated) | Very High |

### 3.2 Primary Persona: "Private Patricia"

- **Age:** 34
- **Occupation:** Therapist (self-employed)
- **Location:** Toronto, Ontario
- **Devices:** Pixel 8 Pro (Android), iPad
- **Pain Points:** Wants to use AI for session notes and client follow-ups, but cannot risk client confidentiality being exposed. Current AI tools are black boxes with unclear data handling.
- **EQ Gateway Value:** Can use AI assistance knowing that client names, addresses, and session details never leave her device. The AI only sees anonymized emotional context.
- **Willingness to Pay:** $8-15/month

### 3.3 Secondary Persona: "Overwhelmed Owen"

- **Age:** 22
- **Occupation:** University Student
- **Location:** Vancouver, BC
- **Devices:** Samsung Galaxy S23
- **Pain Points:** Struggles with anxiety and academic stress. Talks to ChatGPT for support but worries about his personal struggles being stored and analyzed. Has read too many privacy policies.
- **EQ Gateway Value:** Gets emotionally adaptive support without exposing his mental health data. The app learns his mood patterns locally and adjusts its tone accordingly.
- **Willingness to Pay:** $0-5/month (student discount)

### 3.4 Tertiary Persona: "Compliance Carl"

- **Age:** 45
- **Occupation:** IT Director, mid-sized law firm
- **Location:** Calgary, Alberta
- **Devices:** Work-managed Samsung Galaxy S24, work laptop
- **Pain Points:** Firm wants to deploy an AI assistant for lawyers but cannot risk client-solicitor privilege violations. Needs audit trails and compliance documentation.
- **EQ Gateway Value:** Full privacy audit trail, PII redaction guarantees, local-only processing. Can pass PIPEDA/GDPR compliance review.
- **Willingness to Pay:** Enterprise licensing

---

## 4. User Stories

### 4.1 Core Interaction Flow

```
Story ID: US-001
Title: User sends a message privately
As a: Privacy-conscious user
I want: To type a sensitive message and have the AI understand my emotional state
So that: I get a helpful response without exposing my raw personal data
Acceptance Criteria:
- Message text is processed entirely on-device
- Only EQ State (metadata) is available for the AI's reasoning
- User can see the EQ State in the debug overlay
- Response time is under 3 seconds total
```

```
Story ID: US-002
Title: User reviews their privacy audit trail
As a: Privacy-conscious user
I want: To see exactly what data was shared with the AI and what was kept local
So that: I can trust the system is working as promised
Acceptance Criteria:
- Privacy Dashboard shows every EQ State generated
- Tier 3 events (raw text shared) are highlighted with user approval timestamps
- User can export their data as JSON
- User can delete all local data
```

```
Story ID: US-003
Title: User adjusts their privacy tier
As a: Privacy-conscious user
I want: To choose how much context the AI can see
So that: I can balance personalization with privacy based on the situation
Acceptance Criteria:
- Tier selector is accessible from the main screen
- Tiers range from "Fully Local" (Tier 0) to "Approved Excerpt" (Tier 3)
- Changing tier takes effect immediately for the next message
```

### 4.2 Privacy & Control

```
Story ID: US-004
Title: AI requests raw text approval
As a: User
I want: To be prompted when the AI wants to see my actual text
So that: I can make an informed decision about sharing
Acceptance Criteria:
- Prompt shows the reason and a preview of the text
- User can approve, deny, or let it timeout
- All decisions are logged in the Privacy Dashboard
```

```
Story ID: US-005
Title: System detects escalating risk
As a: User in distress
I want: The system to detect when I'm in crisis
So that: It can offer appropriate resources entirely locally
Acceptance Criteria:
- Crisis keywords are detected on-device
- System offers local crisis resources without sending data to cloud
- Escalation is logged locally
```

### 4.3 Personalization

```
Story ID: US-006
Title: System adapts tone based on my mood history
As a: Regular user
I want: The AI to notice when I've been frustrated for several days and adjust
So that: I feel understood without having to explain my mood each time
Acceptance Criteria:
- Mood trending runs entirely on-device
- Response policy adjusts warmth/directness based on trend
- User can override or disable mood-adaptive responses
```

---

## 5. Functional Requirements

### 5.1 On-Device SLM Engine

| ID | Requirement | Priority | Epic |
|:---|:---|:---:|:---:|
| FR-001 | SLM must run entirely on-device with no network calls for inference | P0 | Engine |
| FR-002 | Model must be downloadable post-install from HuggingFace | P1 | Engine |
| FR-003 | Engine must support Q4 and Q8 quantized models (1B-3B params) | P0 | Engine |
| FR-004 | Inference must complete within 2 seconds on target hardware | P1 | Engine |
| FR-005 | Engine must support quantized KV cache for 4K-8K context window | P1 | Engine |
| FR-006 | Engine must output structured JSON matching the EQ State schema | P0 | Compiler |
| FR-007 | Engine must support graceful fallback if model fails to load | P1 | Engine |

### 5.2 PII Redaction

| ID | Requirement | Priority | Epic |
|:---|:---|:---:|:---:|
| FR-010 | Must detect and redact Canadian SIN numbers | P0 | Privacy |
| FR-011 | Must detect and redact Canadian postal codes | P0 | Privacy |
| FR-012 | Must detect and redact email addresses | P0 | Privacy |
| FR-013 | Must detect and redact Canadian phone numbers | P0 | Privacy |
| FR-014 | Must detect and redact Canadian health card numbers | P0 | Privacy |
| FR-015 | Must detect and redact passport numbers | P0 | Privacy |
| FR-016 | Must support regex-based detection as baseline | P0 | Privacy |
| FR-017 | Must support model-assisted detection for adversarial evasion | P1 | Privacy |
| FR-018 | Redaction confidence score must be reported in EQ State | P1 | Privacy |

### 5.3 EQ State Generation

| ID | Requirement | Priority | Epic |
|:---|:---|:---:|:---:|
| FR-020 | Must classify primary affect from controlled vocabulary (18 labels) | P0 | Compiler |
| FR-021 | Must detect secondary affects (0 or more from same vocabulary) | P1 | Compiler |
| FR-022 | Must compute valence score (-1.0 to 1.0) | P0 | Compiler |
| FR-023 | Must compute arousal score (0.0 to 1.0) | P0 | Compiler |
| FR-024 | Must classify user intent from controlled vocabulary (13 labels) | P0 | Compiler |
| FR-025 | Must compute risk level from controlled vocabulary (6 levels) | P0 | Compiler |
| FR-026 | Must determine privacy sensitivity level from controlled vocabulary (6 levels) | P0 | Compiler |
| FR-027 | Must derive response policy (tone, warmth, directness, length) from EQ state | P0 | Compiler |
| FR-028 | Must generate anonymized context summary (PII-free) | P0 | Compiler |
| FR-029 | Every EQ State must include a UUID session identifier and ISO-8601 timestamp | P0 | Compiler |

### 5.4 Local Storage (Librarian)

| ID | Requirement | Priority | Epic |
|:---|:---|:---:|:---:|
| FR-030 | User preferences stored locally; never synced to cloud | P0 | Librarian |
| FR-031 | EQ State history stored in time-series format (30-day rolling window) | P0 | Librarian |
| FR-032 | Conversation turns stored with anonymized content only | P0 | Librarian |
| FR-033 | Database encrypted at rest (AES-256 via SQLCipher) | P0 | Librarian |
| FR-034 | User can export all data as JSON from Privacy Dashboard | P1 | Librarian |
| FR-035 | User can delete all local data (factory reset) | P0 | Librarian |
| FR-036 | Context window assembly must respect token budget parameter | P1 | Librarian |
| FR-037 | Mood trending computed locally; no data leaves device | P1 | Librarian |

### 5.5 MCP Bridge

| ID | Requirement | Priority | Epic |
|:---|:---|:---:|:---:|
| FR-040 | Must expose `get_eq_state` tool with detail_level parameter | P0 | MCP |
| FR-041 | Must expose `get_anonymized_context` tool with task_scope and max_tokens | P0 | MCP |
| FR-042 | Must expose `get_response_policy` tool (no input required) | P0 | MCP |
| FR-043 | Must expose `request_user_approval_for_raw_excerpt` tool | P0 | MCP |
| FR-044 | Tool input and output must be validated against Zod schemas | P0 | MCP |
| FR-045 | Tool registry must be serializable for LLM initialization | P1 | MCP |

### 5.6 Android UI

| ID | Requirement | Priority | Epic |
|:---|:---|:---:|:---:|
| FR-050 | Main chat interface with text input and message list | P0 | UI |
| FR-051 | Debug overlay showing EQ State for each message | P1 | UI |
| FR-052 | Privacy Dashboard with full audit trail | P1 | UI |
| FR-053 | Tier selector (Tier 0-3) accessible from main screen | P1 | UI |
| FR-054 | Settings screen for user preferences | P1 | UI |
| FR-055 | Onboarding flow explaining the privacy model | P2 | UI |
| FR-056 | Dark mode support | P2 | UI |

---

## 6. Non-Functional Requirements

### 6.1 Performance Budgets

| Metric | Target | Hard Limit | Measurement |
|:---|:---:|:---:|:---:|
| EQ State generation latency (1B model, SD8 Elite) | < 1s | 2s | `process_user_input()` wall-clock |
| EQ State generation latency (1B model, SD7-series) | < 2s | 4s | `process_user_input()` wall-clock |
| App cold start to ready | < 3s | 5s | First frame to interactive |
| Model download (1B Q4, ~600MB) | < 30s | 60s | On 5GHz WiFi |
| Librarian context assembly (10 turns) | < 50ms | 100ms | `assembleContext()` wall-clock |
| PII scan (1KB text) | < 10ms | 50ms | `PiiScanner::scan()` wall-clock |
| Memory footprint (engine loaded, idle) | < 200MB | 350MB | RSS as reported by OS |
| Memory footprint (engine inferencing) | < 600MB | 1GB | Peak RSS during inference |
| Database size (30 days, 100 turns/day) | < 5MB | 10MB | SQLite file size |

### 6.2 Privacy Requirements

| ID | Requirement | Verification |
|:---|:---|:---|
| NFR-001 | No raw user text is ever transmitted off-device without explicit user approval | Code audit + network traffic analysis |
| NFR-002 | All locally stored data is encrypted at rest (AES-256) | Encryption key management audit |
| NFR-003 | No analytics, telemetry, or crash reporting includes user text | Code audit |
| NFR-004 | User can delete all their data with a single action | E2E test |
| NFR-005 | Memory containing PII is zeroed immediately after processing | Memory dump test |
| NFR-006 | Privacy Dashboard accurately reflects every data flow | E2E audit test |

### 6.3 Security Requirements

| ID | Requirement | Verification |
|:---|:---|:---|
| NFR-010 | Rust engine must have zero `unsafe` blocks (except OS syscalls) | `#![forbid(unsafe_code)]` lint |
| NFR-011 | Database encryption key must use platform keychain (Android Keystore / iOS Keychain) | Integration test |
| NFR-012 | Model file integrity verified via SHA-256 hash before loading | Integrity check in CI |
| NFR-013 | All network communication to cloud AI uses TLS 1.3 | Network policy enforcement |
| NFR-014 | Crash reporter must be configured to exclude all user data fields | Crash report sample review |

### 6.4 Reliability Requirements

| ID | Requirement | Target |
|:---|:---|:---:|
| NFR-020 | App crash rate | < 0.1% of sessions |
| NFR-021 | EQ State generation success rate | > 99.5% |
| NFR-022 | Model load success rate | > 99% |
| NFR-023 | PII redaction accuracy (recall) | > 99.9% on known patterns |
| NFR-024 | Database operation success rate | > 99.99% |

### 6.5 Compatibility Requirements

| ID | Requirement | Target |
|:---|:---|:---:|
| NFR-030 | Minimum Android API level | API 31 (Android 12) |
| NFR-031 | Target Android API level | API 35 (Android 15) |
| NFR-032 | Minimum iOS version | iOS 17 (for Core ML / Apple Intelligence) |
| NFR-033 | Target chipset (Android) | Snapdragon 8 Elite (Tier 3), Snapdragon 7-series (Tier 1) |
| NFR-034 | Minimum RAM | 6GB (Tier 3), 4GB (Tier 1 on-device only) |

---

## 7. Technical Architecture

### 7.1 System Context Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     MOBILE DEVICE                           │
│                                                             │
│  ┌──────────┐   ┌──────────────┐   ┌──────────────────┐    │
│  │  Native  │──▶│   Librarian  │   │   Rust Engine     │    │
│  │  UI      │   │  (KMP/Store) │   │  (Secure/Infer)   │    │
│  │ (Compose)│   │              │   │                   │    │
│  └──────────┘   └──────┬───────┘   └────────┬──────────┘    │
│                        │                    │               │
│                        └─────────┬──────────┘               │
│                                  │                          │
│                          ┌───────▼────────┐                │
│                          │  MCP Bridge    │                │
│                          │  (KMP/Rust)    │                │
│                          └───────┬────────┘                │
│                                  │                          │
│                                  │ (EQ State JSON only)    │
│                                  │                          │
└──────────────────────────────────┼──────────────────────────┘
                                   │
                           ┌───────▼────────┐
                           │   Cloud AI     │
                           │  (GPT-4,       │
                           │   Claude, etc.)│
                           └────────────────┘
```

### 7.2 Technology Stack

| Layer | Technology | Version | Purpose |
|:---|:---|:---:|:---|
| **Mobile UI (Android)** | Jetpack Compose | 1.6+ | Declarative UI |
| **Mobile UI (iOS)** | SwiftUI | iOS 17+ | Declarative UI |
| **Shared Logic** | Kotlin Multiplatform (KMP) | 1.9+ | Business logic, Librarian, networking |
| **Secure Engine** | Rust | 1.78+ | Inference orchestration, memory safety |
| **Inference Backend** | llama.cpp (via llama-cpp-sys) | Latest | SLM execution on NPU/GPU |
| **Database** | SQLDelight + SQLCipher | 2.0+ | Encrypted local storage |
| **Schema Validation** | Zod + pydantic | TS 3.22 / Py 2.0 | Contract enforcement |
| **FFI Bridge** | UniFFI | 1.0+ | Rust → Kotlin/Swift bindings |
| **Cloud AI** | Provider-agnostic (OpenAI, Anthropic, etc.) | — | Large-scale reasoning |

### 7.3 Data Flow (Single Turn)

```
Step 1: User types message in Jetpack Compose UI
    │
    ▼
Step 2: Kotlin UI sends text to Rust Engine via JNI
    │  (Text crosses into Rust memory — SecureBuffer)
    ▼
Step 3: Rust Engine:
    ├── 3a. Load text into SecureBuffer (mlock, no swap)
    ├── 3b. Run SLM inference (tokenize → predict → logits)
    ├── 3c. Classify: affect, intent, risk from logits
    ├── 3d. Scan for PII, apply redaction per tier rules
    ├── 3e. Compile EQ State JSON
    └── 3f. ZERO the SecureBuffer (raw text gone)
    │
    ▼
Step 4: EQ State JSON returned to Kotlin via JNI
    │  (NO RAW TEXT crosses back)
    ▼
Step 5: Librarian stores:
    ├── EQStateSummary in time-series table
    └── ConversationTurn (anonymized content only)
    │
    ▼
Step 6: MCP Bridge sends EQ State to Cloud AI
    │  (if Tier > 0 and network available)
    │
    ▼
Step 7: Cloud AI responds based on EQ State metadata
    │
    ▼
Step 8: AI response displayed in UI, stored in Librarian
```

---

## 8. Platform Strategy

### 8.1 Android First Rationale

| Factor | Android Advantage |
|:---|:---|
| **Model sovereignty** | Download GGUF/Safetensors from HuggingFace at runtime; no app store review for model updates |
| **Inference runtime** | Mature llama.cpp ecosystem with NPU optimizations via Qualcomm SNPE |
| **Prototyping speed** | Sideload APKs instantly; iterate on PII redaction without review queues |
| **Hardware diversity** | Can validate Tier 3 (SD8 Elite) vs Tier 1 (SD7-series) on real devices |
| **Developer tooling** | Android Studio + GPU Profiler + Memory Profiler for engine optimization |

### 8.2 Cross-Platform Strategy

| Component | Sharing Strategy |
|:---|:---|
| Rust Engine | 100% shared — same `eq-engine-core` compiled for both platforms via cargo-ndk (Android) and cargo-lipo (iOS) |
| Librarian Logic | 100% shared via KMP commonMain — same SQLDelight queries, same preference management |
| EQ State Schemas | 100% shared — TypeScript Schema of Truth compiled to both Kotlin and Swift |
| MCP Bridge | 100% shared via KMP — same tool definitions, same validation logic |
| UI Layer | 0% shared — Jetpack Compose (Android) and SwiftUI (iOS) for platform-native feel |
| Platform Encryption | 0% shared — Android Keystore vs iOS Keychain implementations differ |

### 8.3 iOS Port Strategy

iOS development begins in **Sprint 3** after the Android MVP is validated:

1. Port KMP shared logic (zero code changes expected).
2. Create SwiftUI views matching the Compose screens.
3. Create iOS Rust bindings via UniFFI (same `.udl` file).
4. Optimize for Core ML / Apple Neural Engine.
5. Test on iPhone 15 Pro and above (Apple Intelligence capable).

---

## 9. Data Model

### 9.1 EQ State Schema (Abstract)

```
EQState {
    schema_version: string           // e.g., "0.2"
    session {
        ephemeral_session_id: uuid   // Random UUID, no user identity
        timestamp_local: datetime    // ISO-8601
        device_processing_only: bool // True if Tier 0
    }
    affect {
        primary: enum[18]            // "neutral" | "calm" | "frustrated" | ...
        secondary: enum[18][ ]       // Zero or more
        valence: float [-1.0, 1.0]
        arousal: float [0.0, 1.0]
        confidence: float [0.0, 1.0]
        evidence_type: string
    }
    intent {
        category: enum[13]           // "emotional_support" | "venting" | ...
        subtype: string
        confidence: float [0.0, 1.0]
    }
    risk {
        level: enum[6]               // "none" | "low" | "crisis" | ...
        signals: string[ ]
        confidence: float [0.0, 1.0]
        requires_local_escalation: bool
    }
    privacy {
        sensitivity_level: enum[6]   // "public" | "restricted" | ...
        raw_text_shared: bool
        pii_removed: bool
        sensitive_domains_detected: string[ ]
        redaction_confidence: float [0.0, 1.0]
    }
    response_policy {
        tone: enum[9]                // "calm_direct" | "warm_brief" | ...
        warmth: float [0.0, 1.0]
        directness: float [0.0, 1.0]
        length: "short" | "medium" | "long"
        pace: string
        max_followup_questions: int [0+]
        format: string
    }
    context {
        anonymized_summary: string
        included_raw_excerpt: bool
        retrieval_notes_included: bool
    }
}
```

Full TypeScript/Zod implementation in `schemas/eq-state.schema.ts`.

### 9.2 Local Database Schema

See `10_KMP_Librarian_Architecture.md` for complete SQL schema.

Tables:
- `UserPreferences` — single-row key-value store
- `EQStateHistory` — time-series EQ State records
- `ConversationHistory` — anonymized turn-by-turn log

### 9.3 MCP Tool Definitions

See `schemas/mcp-tools.schema.ts` for complete Zod definitions.

Tools:
- `get_eq_state` — Returns EQ State at specified detail level
- `get_anonymized_context` — Returns sanitized text summary
- `get_response_policy` — Returns tone/length/directness constraints
- `request_user_approval_for_raw_excerpt` — Prompts user for data sharing approval

---

## 10. MCP Interface Specification

### 10.1 Protocol

The MCP Bridge exposes a governed set of tools to the Cloud AI. The bridge ensures:

1. **Only these tools** are available to the Cloud AI (no raw data access).
2. **Tool inputs and outputs** are validated against Zod schemas.
3. **Rate limits** are enforced to prevent excessive context extraction.
4. **Audit logs** record every tool invocation (no content, only metadata).

### 10.2 Tool Definitions

| Tool | Input | Output | Frequency |
|:---|:---|:---|:---:|
| `get_eq_state` | `{detail_level: "minimal"|"standard"|"extended"}` | `{eq_state: EQState, captured_at: datetime}` | Every turn |
| `get_anonymized_context` | `{task_scope: string, max_tokens: int(100-1500)}` | `{anonymized_summary: string, metadata: {...}}` | Once per topic change |
| `get_response_policy` | `{}` | `{policy: {tone, warmth, directness, length, max_followup_questions}}` | Every turn |
| `request_user_approval_for_raw_excerpt` | `{reason: string, excerpt_preview: string}` | `{status: "approved"|"denied"|"timeout", authorized_excerpt?: string}` | Rare (user must opt in) |

### 10.3 Tool Invocation Flow

```
Cloud AI initiates turn
    │
    ├──▶ get_eq_state("standard")
    │       ◀── EQState{affect, intent, risk, privacy, response_policy, context}
    │
    ├──▶ get_response_policy()
    │       ◀── Policy{tone: "gentle_direct", warmth: 0.7, directness: 0.4, ...}
    │
    ├──▶ get_anonymized_context("The user is discussing workplace stress", 500)
    │       ◀── Summary + metadata
    │
    └──▶ [Optional] request_user_approval_for_raw_excerpt(
    │       "The user mentioned a specific colleague name that provides essential context",
    │       "I'm having issues with [---]..."
    │   )
            ◀── {status: "approved", authorized_excerpt: "I'm having issues with Sarah..."}
```

---

## 11. Privacy & Security Model

### 11.1 Privacy Tiers

| Tier | Name | Data Sent to Cloud | Use Case |
|:---:|:---|:---|:---|
| 0 | **Fully Local** | Nothing | Highly sensitive topics (health, legal, relationship) |
| 1 | **Metadata Only** | EQ State (affect, intent, risk, policy) | General conversation with privacy emphasis |
| 2 | **Anonymized Summary** | EQ State + PII-free context summary | Most day-to-day use |
| 3 | **Approved Excerpt** | EQ State + user-approved raw text | Complex topics where nuance is essential |

### 11.2 Memory Safety (Rust Guard)

The Rust engine's `SecureBuffer` provides:

1. **Zeroize-on-drop**: When the buffer goes out of scope, all bytes are overwritten with zeros before deallocation.
2. **mlock**: On Android, the buffer's physical pages are locked to prevent swapping to disk.
3. **No Clone**: The buffer cannot be duplicated; only move semantics.
4. **Audit Logging**: Every buffer creation, access, and wipe is logged (metadata only — never content).

### 11.3 Data Flow Enforcement

```
Boundary 1: UI → Rust Engine
    - Raw text crosses via JNI
    - Immediately moved to SecureBuffer
    - Original JNI string heap is zeroed

Boundary 2: Rust Engine → KMP Logic (Librarian)
    - Only EQ State JSON crosses
    - anonymized_summary is PII-free by this point
    - NO raw text ever crosses this boundary

Boundary 3: KMP Logic → Cloud AI (via MCP)
    - Only EQ State JSON crosses (respecting current tier)
    - Tier 3 requires explicit user approval dialog
```

---

## 12. Compliance & Regulation

### 12.1 Applicable Regulations

| Regulation | Jurisdiction | Relevance | Risk Level |
|:---|:---|:---|:---:|
| **PIPEDA** | Canada (federal) | Consent, data minimization, individual access | High |
| **PHIPA** | Ontario (health) | Health data handling requirements | High (if health-adjacent) |
| **GDPR** | EU / UK | Data protection, right to erasure, DPIA | Medium (if serving EU users) |
| **CCPA** | California, USA | Consumer privacy, opt-out rights | Low (initial focus is Canada) |
| **Bill 64 / Law 25** | Quebec | Additional consent and privacy obligations | Medium (if serving Quebec) |

### 12.2 Compliance Strategy

| Requirement | EQ Gateway Approach |
|:---|:---|
| **Data Minimization** | Only EQ State metadata is shared; no raw text by default |
| **Purpose Limitation** | Data is used only for the specific AI interaction; no secondary analytics |
| **Individual Access** | Privacy Dashboard allows users to view and export all their data |
| **Right to Erasure** | Single-button wipe of all local data |
| **Consent** | Tier 3 requires explicit opt-in via UI prompt |
| **Privacy Impact Assessment** | Architecture built for DPIA submission; documentation exists for all data flows |
| **Data Portability** | JSON export of all stored data available in Privacy Dashboard |

### 12.3 Recommended Legal Positioning

EQ Gateway should be positioned as:

> **A local computation layer, not a data processor.**

The argument: EQ Gateway does not *collect*, *store*, or *transmit* personal data. It transforms text into metadata at the edge. The metadata (EQ State) is an anonymized derivative that cannot be reversed to reconstruct the original text. This creates a defensible privacy position:

- **PIPEDA compliance**: No collection of personal information without consent (because no PI is collected by default).
- **PHIPA avoidance**: Health data is processed locally and never transmitted; redacted at Tier 0.
- **GDPR data processor status**: The device owner controls all processing; EQ Gateway is a tool, not a controller.

---

## 13. Monetization Strategy

### 13.1 Pricing Tiers

| Tier | Price | Target | Features |
|:---|:---:|:---|:---|
| **Free** | $0 | Students, casual users | Tier 0-2 processing, 50 turns/day, limited mood history (7 days) |
| **Pro** | $9.99/mo | Privacy-conscious consumers | All tiers, unlimited turns, 90-day mood history, full export, priority model updates |
| **Enterprise** | Custom | Law firms, healthcare, regulated industries | On-prem deployment option, audit logging, SLA, dedicated model fine-tuning, SSO |

### 13.2 Value Proposition by Tier

| Stakeholder | Value | Price Sensitivity |
|:---|:---|:---:|
| **End user** | Private AI interactions that understand emotions | Low-moderate ($5-15/mo) |
| **Enterprise** | PIPEDA/GDPR-compliant AI assistant for sensitive workflows | Low ($50-500/seat/mo) |
| **AI platform** | Privacy layer as a differentiation feature | N/A (partnership/licensing) |

### 13.3 Potential Revenue Streams

1. **Direct subscription** (Pro users) — primary revenue.
2. **Enterprise licensing** — per-seat pricing for regulated industries.
3. **OEM licensing** — embed EQ Gateway as a privacy layer in existing AI assistant apps.
4. **Model marketplace** — allow fine-tuned domain models (e.g., "Canadian healthcare EQ model") to be sold through the app.

---

## 14. Release Criteria

### 14.1 MVP Release Criteria (v1.0)

- [ ] Android app installable from Google Play Store.
- [ ] Model downloads from HuggingFace on first launch.
- [ ] User can type a message and receive a response adapted to their EQ State.
- [ ] EQ State JSON validated against schema (100% compliance).
- [ ] PII redaction achieves > 99.9% recall on Canadian pattern test suite.
- [ ] Zero PII leaks in anonymized summaries (verified by eval harness).
- [ ] Librarian persists data across app restarts (encrypted).
- [ ] Privacy Dashboard shows audit trail for at least the current session.
- [ ] Full pipeline completes in under 3 seconds on Snapdragon 8 Elite.
- [ ] Open-source evaluation harness available for community audit.

### 14.2 v1.1 Release Criteria

- [ ] iOS app (iPhone 15 Pro+) available on App Store.
- [ ] Mood trending visualization in Privacy Dashboard.
- [ ] User feedback mechanism (thumbs up/down on AI responses).
- [ ] Pro subscription tier active.
- [ ] Canadian French language support.

### 14.3 v2.0 Release Criteria

- [ ] Enterprise on-prem deployment option.
- [ ] Custom model fine-tuning service.
- [ ] Multi-platform support (desktop, web via WASM).
- [ ] Third-party app SDK for embedding EQ Gateway.
- [ ] SOC 2 Type II audit completed.

---

## 15. Roadmap & Milestones

### 15.1 Phase 0: Prototype (Weeks 1-4)

| Milestone | Date | Deliverable |
|:---|:---:|:---|
| Sprint 1 complete | Week 2 | E2E Android prototype, EQ State pipeline, schema validation |
| Sprint 2 complete | Week 4 | MCP bridge active, Privacy Dashboard v1, PII test suite passing |
| **Phase 0 Gate** | **Week 4** | **Decision: proceed to MVP or pivot** |

### 15.2 Phase 1: MVP (Weeks 5-12)

| Milestone | Date | Deliverable |
|:---|:---:|:---|
| Sprint 3 complete | Week 6 | Mood trending, user feedback, iOS port begins |
| Sprint 4 complete | Week 8 | iOS MVP, onboarding flow, subscription integration |
| Sprint 5 complete | Week 10 | Performance optimization, battery testing, polish |
| Sprint 6 complete | Week 12 | Security audit, compliance review, Play Store submission |
| **v1.0 Launch** | **Week 12** | **Android MVP on Google Play** |

### 15.3 Phase 2: Growth (Weeks 13-24)

| Milestone | Date | Deliverable |
|:---|:---:|:---|
| iOS launch | Week 16 | v1.0 on App Store |
| Enterprise beta | Week 20 | 3 design partners onboarded |
| v2.0 launch | Week 24 | Enterprise tier, SDK, on-prem |

---

## 16. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|:---|:---:|:---:|:---|
| **SLM model quality insufficient** for reliable affect classification | Medium | High | Invest in evaluation harness early; test multiple base models (Phi-3, Llama-3.2, Gemma-2) in Phase 0 |
| **Inference latency too high** on mid-range devices | Medium | High | Aggressive quantization (Q4); KV cache quantization; target SD7-series as minimum; dynamic model size selection |
| **PII redaction fails** on adversarial input (leetspeak, split words) | Medium | Critical | Two-layer approach: regex (speed) + model-assisted (accuracy); adversarial test corpus |
| **User adoption low** due to complexity of privacy model | Medium | Medium | Clear onboarding with tier examples; progressive disclosure — start at Tier 2, let users explore |
| **Regulatory challenge** — regulator decides EQ State is personal data | Low | High | Legal review of anonymization strength; commission third-party privacy audit; prepare DPIA |
| **iOS App Store rejection** for on-device AI model downloading | Low | Medium | Bundle a small starter model; download larger models as "optional assets" via App Store guidelines |
| **UniFFI binding generation** too complex for cross-compilation | Low | Medium | Manual JNI bridge as fallback; allocate buffer time in Sprint 1 for UniFFI spike |

---

## 17. Competitive Landscape

### 17.1 Direct Competitors

| Competitor | Approach | EQ Gateway Advantage |
|:---|:---|:---|
| **Apple Intelligence** | On-device processing, but opaque to user and developer | Open architecture; user-visible EQ State; cross-platform (not Apple-only) |
| ** Google AICore** | On-device foundation models, Android-only, no explicit privacy tier system | Granular tier control; explicit PII redaction layer; Canadian compliance focus |
| **Anthropic / Claude** | Privacy-first cloud AI (trust boundary) | EQ Gateway is a *complement*, not competitor — Claude could be the cloud AI using EQ State |
| **Local AI apps (PrivateGPT, Ollama)** | Fully local, but no EQ State metadata layer | Metadata enables adaptive cloud AI while keeping data local; best of both worlds |

### 17.2 Competitive Moat

1. **The dual-layer architecture** is defensible — combining local SLM classification with cloud AI reasoning in a governed, auditable way.
2. **Canadian privacy compliance focus** — PIPEDA/PHIPA expertise is a concrete market advantage for Canadian healthcare and legal sectors.
3. **Open-source evaluation harness** — community trust through transparency.
4. **Adversarial PII test corpus** — verifiable redaction guarantees that competitors lack.
5. **First-mover in the "emotional privacy" niche** — no major competitor is explicitly solving this problem.

---

## 18. Glossary

| Term | Definition |
|:---|:---|
| **EQ State** | Structured metadata payload containing affect, intent, risk, privacy, response policy, and context. The only data that leaves the device. |
| **EQ Engine** | The on-device SLM that classifies user text into an EQ State. Runs in the Rust layer. |
| **The Top Hat** | The structured prompt contract governing what the local SLM can and cannot output. Prevents the model from generating anything outside the EQ State schema. |
| **MCP Bridge** | The Model Context Protocol interface that exposes governed tools to the Cloud AI. |
| **Librarian** | The local, encrypted, on-device data store for user preferences, EQ State history, and conversation turns. Never syncs to the cloud. |
| **SecureBuffer** | Rust memory management structure that zeroes PII on drop, mlock'd against swap, and non-cloneable. |
| **Privacy Tier** | A user-selectable setting (0-3) that controls how much data is shared with the cloud AI. |
| **Tier 0** | Fully local processing. No data leaves the device. |
| **Tier 1** | Metadata only. Only EQ State (affect, intent, risk, policy) is shared. |
| **Tier 2** | Anonymized summary. EQ State + PII-free context summary. |
| **Tier 3** | Approved excerpt. EQ State + user-approved raw text excerpt. |
| **SLM** | Small Language Model (1B-3B parameters) running on-device for classification and redaction. |
| **Large AI** | The cloud-based LLM (e.g., Claude, GPT-4) that receives only the EQ State and generates the final response. |
| **PII** | Personally Identifiable Information — any data that could identify an individual (SIN, health card, address, etc.). |
| **PIPEDA** | Canada's Personal Information Protection and Electronic Documents Act — federal privacy law. |
| **PHIPA** | Ontario's Personal Health Information Protection Act — provincial health privacy law. |
| **Model Card** | JSON artifact generated by the evaluation harness that gates model deployment in CI/CD. |
| **Convergent Tech Stack** | Multi-language architecture where TypeScript defines contracts, Python trains models, Rust secures data, and KMP bridges to mobile. |

---

*Confidential — EQ Gateway Project*  
*This document contains proprietary information intended for internal use only.*
