# EQ Gateway: Unified Development Workflow

This document defines the multi-language strategy for the EQ Gateway project. To maximize developer velocity and maintain the highest levels of privacy and performance, we utilize a "Convergent Tech Stack."

## 1. Language Responsibility Matrix

| Language | Role | Primary Domain | Key Libraries/Tools |
| :--- | :--- | :--- | :--- |
| **TypeScript** | **The Architect** | Contracts, Schemas, MCP simulation, Web/Dev tooling. | `Zod` (Schema), `Vitest` (Testing), `TypeChat`. |
| **Python** | **The Scientist** | Model training, fine-tuning, quantization research, and metric evaluation. | `PyTorch`, `Transformers`, `NumPy`, `HuggingFace`. |
| **Rust** | **The Guard** | High-performance inference orchestration, secure memory management (PII handling). | `UniFFI` (Bindings), `Tokio` (Async), `ndarray`. |
| **Kotlin** | **The Bridge** | Android UI, KMP Business Logic, Lifecycle management. | `Jetpack Compose`, `Ktor`, `Kotlin Multiplatform`. |
| **Swift** | **The Bridge** | iOS UI, high-level OS integration. | `SwiftUI`, `Combine`. |

## 2. The "Source of Truth" Workflow

To prevent drift between the "Brain" (Python), the "Guard" (Rust), and the "App" (Kotlin/Swift), we follow this sequence:

### Step 1: Define the Contract (TypeScript)
Every new feature begins with a TypeScript schema definition using `Zod`.
* **Action:** Define the `EQ_State` or a new tool payload in a central `.ts` file.
* **Output:** A strictly typed JSON contract that acts as the "Source of Truth."

### Step 2: Model Validation (Python)
The AI researchers use the TypeScript schema to validate that the models they are tuning actually produce the expected output.
* **Action:** Run Python evaluation scripts that ingest sample prompts and verify the resulting JSON matches the TS schema.

### Step 3: Secure Implementation (Rust)
The engineers implement the heavy lifting in Rust, ensuring that the logic adheres to the memory safety required for PII handling.
* **Action:** Write the Rust modules that ingest raw text $\rightarrow$ apply redaction $\rightarrow$ output the JSON defined in Step 1. Use `UniFFI` to generate bindings for the mobile layers.

### Step 4: Mobile Integration (KMP/Native)
The mobile developers consume the high-level logic via KMP and the high-performance engine via the Rust bindings.
* **Action:** Implement the UI and local orchestration, calling the pre-validated Rust/KMP logic.

## 3. Summary of Workflow Cycles

* **Feature Cycle:** TS (Define) $\rightarrow$ Python (Verify Model) $\rightarrow$ Rust (Implement Security) $\rightarrow$ KMP/Native (Deploy UI).
* **Bug Fix Cycle (Logic):** TS (Reproduce via Mock) $\rightarrow$ Rust (Fix in Engine) $\rightarrow$ Mobile (Update).
* **Bug Fix Cycle (Model):** Python (Retune/Quantize) $\rightarrow$ Rust (Update Weights/Runtime) $\rightarrow$ Mobile (Update).

---
*Confidential - EQ Gateway Project*
