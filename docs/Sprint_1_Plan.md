# Sprint 1 Plan — Foundation & Core Pipeline

**Sprint:** 1 of 4  
**Duration:** 2 weeks (10 working days)  
**Theme:** Foundation & Core Pipeline  
**Goal:** Working end-to-end prototype on Android emulator — raw text in, EQ State JSON out, validated against schema.

---

## 1. Sprint Scope

### In Scope
- Android project scaffold (Jetpack Compose + KMP shared module)
- Rust crate scaffold (all 6 crates, compilable)
- Python evaluation harness (schema validation working)
- Librarian local SQLite database operational
- Single end-to-end pipeline: Capture → Infer → Redact → Compile → Store → Validate

### Out of Scope (Future Sprints)
- iOS port
- MCP network bridge to cloud AI
- Privacy dashboard UI
- Mood trend visualization
- User onboarding flow
- Production model fine-tuning (using placeholder/test model)

### Leveraged Existing Work
The following assets from [vulkan-polaris-llama](https://github.com/andrewdhannah/vulkan-polaris-llama) are reused directly:
- **llama.cpp Vulkan build** (known-good, Polaris-fixed) — no need to build llama.cpp from scratch
- **server-mini.cpp** (843-line HTTP server embedding llama.cpp) — adapted as the inference runtime
- **model_manager.ps1** — model download and switching logic adapted for mobile
- **Benchmark results** for Phi-4-mini, Llama 3.2, and Gemma 3 across 5 test categories
- **Known-good state** documented; saves weeks of Vulkan debugging on mobile GPUs

---

## 2. Ticket Breakdown

### Epic 1: Project Scaffold & Toolchain (Days 1-2)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-001 | Initialize Android project with KMP shared module | 4h | None | `./gradlew assembleDebug` succeeds; shared module compiles; Jetpack Compose renders blank screen |
| S1-002 | Initialize Rust workspace with all 6 crates | 4h | None | `cargo build` succeeds across all crates; `cargo test` on empty tests passes |
| S1-003 | Set up UniFFI bridge skeleton | 4h | S1-002 | `cargo build` produces `.so` for Android; UniFFI generates Kotlin bindings |
| S1-004 | Initialize Python eval harness project | 2h | None | `pip install -r requirements.txt` succeeds; `pytest` discovers tests |
| S1-005 | Configure CI (GitHub Actions) for all three stacks | 4h | S1-001, S1-002, S1-004 | Push to `main` triggers Rust build, Android build, Python tests; all green |

**Epic 1 Total: 18h**

---

### Epic 2: Secure Memory & PII Filter (Days 2-4)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-006 | Implement `SecureBuffer` with zeroize-on-drop | 6h | S1-002 | `SecureBuffer::from_string()` wipes source; `Drop` zeroes heap; `Clone` is NOT implemented; test verifies memory content after drop is all zeros |
| S1-007 | Add `mlock` support for Android (`#[cfg(target_os = "android")]`) | 3h | S1-006 | `mlock` is called on buffer creation on Android; gracefully no-ops on other platforms |
| S1-008 | Implement Canadian PII pattern library | 4h | S1-002 | All 6 pattern categories (SIN, health, postal code, email, phone, passport) compiled as regex; test corpus of 20 examples per category all detected |
| S1-009 | Implement `PiiScanner::scan()` with tier enforcement | 6h | S1-008 | Scanner returns `clean: bool`, `matches[]`, `critical_count`; level 0 redaction blocks everything; level 3 allows specified categories |
| S1-010 | Write `SecureBuffer` security audit tests | 3h | S1-006 | "Zero Leak" integration test: feed PII string → process → dump memory → assert PII NOT present |

**Epic 2 Total: 22h**

---

### Epic 3: Inference Pipeline (Days 3-6)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-011 | Integrate llama.cpp via vulkan-polaris-llama fork (C FFI) | 6h | S1-002 | Fork/cherry-pick the known-good Vulkan Polaris fix; `LlamaCppBackend::load()` loads a Q4_K_M GGUF; `infer()` returns tokenized output. **Reuses proven server-mini.cpp** from existing repo |
| S1-012 | Implement `EngineConfig` with model path, GPU layers, context size | 3h | S1-011 | Config parsed from JSON; invalid configs return `EngineError::ConfigError` |
| S1-013 | Build `pipeline::process()` — the core inference pipeline | 8h | S1-009, S1-011 | Full pipeline: tokenize → infer → extract logits → classify → compile → wipe |
| S1-014 | Implement affect/intent extraction from SLM logits | 6h | S1-011 | Logit vectors mapped to `AffectPrimary` variant; confidence score calculated via softmax |
| S1-015 | Add inference timeout and retry logic | 3h | S1-013 | Default timeout of 5s; configurable; returns `EngineError::InferenceTimeout` if exceeded |
| S1-016 | Benchmark inference speed on Snapdragon 8 Elite emulator | 3h | S1-013 | `process()` completes under 2s for 1B model; results logged to structured output |

**Epic 3 Total: 31h**

---

### Epic 4: EQ State Compiler (Days 5-7)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-017 | Build `AffectState` compiler module | 4h | S1-014 | Raw logits → `{primary, secondary[], valence, arousal, confidence}` |
| S1-018 | Build `IntentState` compiler module | 3h | S1-014 | Raw logits → `{category, subtype, confidence}` |
| S1-019 | Build `RiskState` compiler module | 3h | S1-009 | PII scan results + affect signals → `{level, signals[], confidence, requiresLocalEscalation}` |
| S1-020 | Build `PrivacyState` compiler module | 3h | S1-009, S1-019 | Redaction results + tier rules → `{sensitivityLevel, rawTextShared, piiRemoved, redactionConfidence}` |
| S1-021 | Build `ResponsePolicy` compiler module | 3h | S1-017, S1-018 | Affect + Intent → `{tone, warmth, directness, length, maxFollowupQuestions}` |
| S1-022 | Build `EQState` master compiler (assembles all sub-states) | 4h | S1-017 through S1-021 | JSON output matches `EQStateSchema` (validated against TS schema); session UUID generated; timestamp ISO-8601 |
| S1-023 | Compile-time validation test: output JSON against Zod schema | 4h | S1-022, S1-004 | Python harness loads TS schema, sends Rust output, validates — 100% pass rate required |

**Epic 4 Total: 24h**

---

### Epic 5: Librarian Local Store (Days 6-8)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-024 | Scaffold SQLDelight database with schema | 4h | S1-001 | All 3 tables created; `./gradlew` generates coroutine queries |
| S1-025 | Implement `UserPreferences` save/load | 3h | S1-024 | Default values returned on first launch; updates persisted across app restarts |
| S1-026 | Implement `EQStateHistory` time-series storage | 4h | S1-024 | Records appended; queryable by date range; indexed on `captured_at` |
| S1-027 | Implement `ConversationHistory` CRUD | 4h | S1-024 | Insert, query by session, query by date range; auto-purge not yet active |
| S1-028 | Implement `Librarian.recordTurn()` — the main write path | 3h | S1-026, S1-027 | Takes EQStateSummary + anonymized content; writes atomically to both tables |
| S1-029 | Implement `Librarian.assembleContext()` — the main read path | 5h | S1-028 | Returns recent N turns + compressed older summary; respects token budget |
| S1-030 | Implement retention policy placeholder (stub, not active) | 2h | S1-028 | Policy object exists and loads from config; `applyRetentionPolicy()` logs but does not delete yet |

**Epic 5 Total: 25h**

---

### Epic 6: Python Validation Harness (Days 7-10)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-031 | Port EQ State schemas to Python (pydantic models) | 4h | S1-004 | All enums + sub-models + master `EQState` model; field constraints match TS schema |
| S1-032 | Port MCP tool schemas to Python | 2h | S1-031 | `GetEqStateOutput`, `GetAnonymizedContextOutput`, etc. — matching field-level constraints |
| S1-033 | Implement `PiiScanner` in Python (mirrors Rust version) | 4h | S1-031 | Same 6 Canadian pattern categories; returns same `{clean, matches[], criticalCount}` structure |
| S1-034 | Build `SchemaValidator` — validates JSON strings against pydantic models | 3h | S1-031 | Takes JSON string + model class; returns `(is_valid, errors[])` |
| S1-035 | Create synthetic test corpus (20 Canadian PII examples) | 3h | None | 5 SIN examples, 5 health card, 5 address+postal, 5 mixed; each in a realistic sentence |
| S1-036 | Build `run_full_eval.py` pipeline script | 6h | S1-033, S1-034, S1-035 | End-to-end: load JSON outputs from Rust → validate schema → scan for PII → generate report |
| S1-037 | Smoke test: validate Rust pipeline output through Python harness | 4h | S1-023, S1-036 | Rust compiler output → Python validator → 100% schema pass + 0% PII leak |
| S1-038 | Generate Model Card v0.1 for the test model | 2h | S1-037 | JSON card created; `approved: true` if all criteria met; saved to `output/model_cards/` |
| S1-039 | Document Python harness README and usage | 2h | S1-036 | README with install, run, interpret results; example output in doc |

**Epic 6 Total: 30h**

---

### Epic 7: Android UI Shell & Integration (Days 8-10)

| Ticket ID | Title | Effort | Dependencies | Acceptance Criteria |
|:---|:---|:---:|:---|:---|
| S1-040 | Build main chat screen shell (Jetpack Compose) | 6h | S1-001 | Text input field + message list + send button; all styled with Material 3 |
| S1-041 | Wire JNI bridge — Kotlin calls Rust `process_user_input()` | 6h | S1-003, S1-022 | Typing text → JNI → Rust → EQ State JSON returned → displayed in debug overlay |
| S1-042 | Integrate Librarian into app lifecycle | 4h | S1-028, S1-040 | App start initializes DB; each turn recorded via `recordTurn()`; context loaded for subsequent turns |
| S1-043 | Build debug overlay showing EQ State in real-time | 4h | S1-041 | Expandable panel under each message shows `affect_primary`, `risk_level`, `intent_category`, `privacy_sensitivity` |
| S1-044 | Test full pipeline on Android emulator (API 35, arm64) | 4h | S1-041, S1-042 | Text typed → EQ State generated → stored in Librarian → displayed in debug overlay — all within 3s |
| S1-045 | Write integration test: full pipeline E2E | 4h | S1-041 | Instrumented test: inject text → assert EQ State returned → assert matches schema → assert no PII in stored summary |
| S1-046 | Sprint demo preparation and documentation | 4h | S1-044 | Demo script written; known limitations documented; handoff notes for Sprint 2 |

**Epic 7 Total: 32h**

---

## 3. Sprint Summary

| Epic | Hours | Tickets | Owner |
|:---|:---:|:---:|:---|
| 1. Project Scaffold & Toolchain | 18h | 5 | DevOps |
| 2. Secure Memory & PII Filter | 22h | 5 | Rust |
| 3. Inference Pipeline | 31h | 6 | Rust |
| 4. EQ State Compiler | 24h | 7 | Rust |
| 5. Librarian Local Store | 25h | 7 | KMP |
| 6. Python Validation Harness | 30h | 9 | Python |
| 7. Android UI & Integration | 32h | 7 | Android |
| **Total** | **182h** | **46** | — |

### Capacity Planning

| Role | Available Hours | Assigned | Utilization |
|:---|:---:|:---:|:---:|
| Rust Engineer | 80h | 77h | 96% |
| KMP/Android Engineer | 80h | 57h | 71% |
| Python/ML Engineer | 80h | 30h | 38% |
| DevOps | 40h | 18h | 45% |

> **Note:** Python/ML engineer has spare capacity in this sprint because model fine-tuning is a Sprint 2 activity. They can assist with Rust crate testing.

---

## 4. Key Milestones

| Day | Milestone | Verification |
|:---:|:---|:---|
| 2 | All three projects compile | `cargo build`, `./gradlew assembleDebug`, `pytest` all green |
| 4 | SecureBuffer passes "Zero Leak" test | Integration test confirms PII string is not recoverable from memory |
| 6 | Full inference pipeline functional | Raw text in, EQ State JSON out, validated against TS schema |
| 8 | Librarian stores and retrieves context | `assembleContext()` returns correct window with compression |
| 10 | E2E demo on Android emulator | Type text → see EQ State overlay → stored in Librarian → all green |

---

## 5. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|:---|:---:|:---:|:---|
| llama.cpp JNI integration issues on arm64 | Medium | High | Start with x86_64 emulator; have MLC-LLM as fallback backend |
| SQLDelight schema migrations in KMP | Low | Medium | Keep schema v0.1 simple; migration manager is a placeholder |
| 1B model too slow on emulator (no GPU) | High | Medium | Use smaller test model (phi-2, 2.7B quantized); set realistic latency expectations for sprint demo |
| UniFFI binding generation complexity | Medium | High | Manual JNI bridge as fallback; UniFFI as stretch goal |
| Team member unavailability | Low | Medium | Cross-train Python engineer on Rust crate basics in Epic 2 |

---

## 6. Definition of Done (Sprint 1)

- [ ] All 46 tickets closed (or moved to Sprint 2 with explicit reason).
- [ ] Android app runs on emulator (API 35, arm64).
- [ ] User can type text → EQ State is generated and displayed.
- [ ] EQ State JSON passes schema validation (100%).
- [ ] Anonymized summaries contain zero PII (verified by Python harness).
- [ ] Librarian persists data across app restarts.
- [ ] CI pipeline green for all three stacks.
- [ ] Known limitations documented in `SPRINT-1-REVIEW.md`.

---

*Confidential — EQ Gateway Project*
