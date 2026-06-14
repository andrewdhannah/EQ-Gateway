# Design Document — **EQ Gateway**

**Status:** Concept / Technical Design Draft (Updated with Convergent Tech Stack)  
**Owner:** Placeholder  
**Date:** June 04, 2026  
**Version:** 0.2  
**Working Title:** EQ Gateway  
**Product Category:** On-device SLM + privacy-preserving emotional context middleware + MCP bridge

---

## 1. Executive Summary

**EQ Gateway** is a privacy-preserving emotional context system that uses a small language model (SLM) running locally on a mobile device to infer a user’s mood, communication needs, risk state, and contextual preferences. Instead of sending raw personal data to a cloud model, the local SLM produces a constrained, anonymized, structured emotional-state payload (**EQ State**). This payload is passed via an MCP interface to a larger AI, allowing the cloud model to adapt its tone and strategy without accessing private user history.

The product thesis:
> Keep the emotionally sensitive layer local. Let larger AI reason over anonymized emotional metadata rather than raw private context.

---

## 2. Core Concept

The system implements a "Dual-Layer" architecture:

1.  **On-device SLM (The EQ Engine):** Classifies, redacts, summarizes, and routes. Acts as a privacy firewall.
2.  **Large AI (The Reasoning Engine):** Synthesizes, writes, and plans based on the metadata provided.

### The Privacy Pivot
Unlike traditional assistants that scale personalization via data collection, EQ Gateway scales via **structured metadata**.
*   *Traditional:* More data $\rightarrow$ More personalization $\rightarrow$ Higher risk.
*   *EQ Gateway:* Less raw data $\rightarrow$ More useful emotional structure $\rightarrow$ Lower risk.

---

## 3. System Architecture & Tech Stack (Convergent Model)

To ensure privacy integrity and cross-platform scalability, the project utilizes a convergent technology stack.

### 3.1 Hardware Strategy
*   **Tier 3 (Flagship):** Targeting Snapdragon 8 Elite (optimized for high-throughput NPU/GPU inference).
*   **Tier 1 (Baseline):** Targeting mid-range chipsets (ensuring broad market compatibility).

### 3.2 The Convergent Tech Stack

| Layer | Technology | Role | Rationale |
| :--- | :--- | :--- | :--- |
| **UI Layer** | Jetpack Compose (Android) / SwiftUI (iOS) | User Interaction | Platform-native premium experience. |
| **Logic Layer** | **Kotlin Multiplatform (KMP)** | Librarian, Privacy Rules, EQ State Compiler | Single source of truth for business logic across mobile. |
| **Engine Layer** | **Rust** | Inference Orchestration, Secure Memory/PII handling | **Compile-time memory safety; prevents data leakage.** |
| **Inference Backend**| llama.cpp / MLC-LLM | Mathematical kernels | Optimized for mobile NPUs/GPUs. |
| **Research Layer**| **Python** | Model tuning, quantization, and evaluation | Industry standard for AI science and training. |
| **Contract Layer**| **TypeScript** | Schema definition & MCP simulation | High-fidelity typing for structured payloads. |

### 3.3 Communication Flow
1.  **Input:** User provides raw text to the Native UI.
2.  **Processing:** Text is passed to the **Rust Engine**. Rust manages the sensitive memory buffer and executes the SLM inference.
3.  **Transformation:** The SLM produces an **EQ State** payload (JSON). Rust ensures the original text is securely wiped from memory once the payload is generated.
4.  **Handoff:** The payload is passed through the **KMP Logic Layer** to the **MCP Bridge**.
5.  **Cloud Interaction:** The Cloud AI receives only the structured metadata.

---

## 4. SLM Role & Strategy

### 4.1 SLM Specialization
The SLM is not a generalist; it is a specialist in:
*   Mood/Intent/Risk classification.
*   PII-aware summarization.
*   Response-policy recommendation.

### 4.2 Model Optimization
To meet mobile constraints, models must implement:
*   **Aggressive Quantization:** 4-bit or 8-bit weights.
*   **Quantized KV Cache:** Critical for supporting target context windows (4K–8K) within mobile RAM limits without performance degradation.

---

## 5. Privacy & Security Model

### 5.1 The "Rust Guard" Principle
The use of Rust for the engine layer provides a mathematical guarantee of memory safety. This ensures that raw PII handled during the "Local Capture" phase cannot be accessed by unauthorized memory reads, reinforcing the "Privacy Firewall" brand promise.

### 5.2 Privacy Tiers
*   **Tier 0 (Fully Local):** No data leaves the device.
*   **Tier 1 (Metadata Only):** Only emotional/intent metadata is shared.
*   **Tier 2 (Anonymized Summary):** Sanitized context summary is shared.
*   **Tier 3 (Approved Excerpt):** User-approved specific text is shared.

---

## 6. Implementation Roadmap (High Level)

*   **Phase 0: Prototype (Python/TS):** Validate JSON output stability and PII redaction accuracy using Python for modeling and TypeScript for schema testing.
*   **Phase 1: Foundation (Rust/KMP/Android):** Build the secure Rust engine and the KMP business logic layer on an Android baseline.
*   **Phase 2: MCP Bridge:** Implement the governed tool interface between the device and cloud models.
*   **Phase 3: Scaling:** Port to iOS via KMP/SwiftUI and optimize for Apple Silicon.

---

## 7. Final Product Thesis

EQ Gateway is a **Local Emotional Firewall**. It enables high-utility, emotionally adaptive AI experiences by ensuring that the most sensitive data—human emotion and private context—never leaves the owner's device in its raw form.
