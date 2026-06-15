# Sprint 2 Plan — Agent Gateway, Android Integration & MCP Bridge

**Sprint:** 2 of 4  
**Duration:** 2 weeks (10 working days)  
**Theme:** Agent Gateway Service, Mobile Integration & MCP Bridge  
**Goal:** Working agentic pipeline with HITL gates, observability, Android FFI integration, and MCP tools for cloud AI.

---

## Overview

Sprint 2 runs three parallel tracks:

| Track | Description | Dependencies |
|:---|:---|:---:|
| **Option A: Agent Gateway** | FastAPI + LangGraph service wrapping the EQ engine with model routing, HITL gates, and receipts | Rust engine (Sprint 1) |
| **Option B: Android Integration** | Wire UniFFI/JNA bridge, Compose chat UI, on-device inference test | Rust FFI (Sprint 1), Android scaffold (S1-001) |
| **Option C: MCP Bridge** | Expose EQ State as MCP tools for cloud AI consumption | FastAPI service foundations |

---

## Option A — Agent Gateway Service (Priority 1)

### Goal
Build a Python FastAPI service that wraps the Rust EQ Gateway engine with:
- Model routing (local-first vs cloud)
- Human-in-the-loop approval gates for sensitive content
- Structured observability receipts for every request

### Architecture

```
┌─────────────┐     ┌──────────────────────────────────────┐
│   Client    │     │         Agent Gateway (:8090)         │
│  (curl/UI)  │────▶│                                      │
└─────────────┘     │  POST /chat                          │
                    │    ┌──────────────┐                   │
                    │    │  EQ Client   │──▶ llama-server   │
                    │    │  (Python)    │◀── :9120          │
                    │    └──────┬───────┘                   │
                    │           ▼                           │
                    │    ┌──────────────┐                   │
                    │    │  Model       │── local/cloud     │
                    │    │  Router      │── or HITL gate    │
                    │    └──────┬───────┘                   │
                    │           ▼                           │
                    │    ┌──────────────┐                   │
                    │    │  HITL Gate   │── approve/reject  │
                    │    │  (LangGraph) │── human pause     │
                    │    └──────┬───────┘                   │
                    │           ▼                           │
                    │    ┌──────────────┐                   │
                    │    │  Response    │── final response  │
                    │    │  + Receipt   │── structured log  │
                    │    └──────────────┘                   │
                    └──────────────────────────────────────┘
```

### Tickets

| ID | Title | Effort | Description |
|:---|:---|:---:|:---|
| S2-A01 | Scaffold agent-gateway service | 2h | `pyproject.toml`, FastAPI app, config, directory structure |
| S2-A02 | Build EQ Client (Python) | 6h | HTTP client to llama-server-mini :9120, parses structured JSON output, handles errors |
| S2-A03 | Build POST /analyze endpoint | 3h | Accept user input → call EQ Client → return EQ State JSON |
| S2-A04 | Build Model Router | 4h | Rules engine: risk level → routing decision (local/cloud/HITL), sensitivity thresholds |
| S2-A05 | Build HITL Gate (LangGraph) | 8h | State machine with interrupt/resume, approve/reject/edit flow, timeout handling |
| S2-A06 | Build Observability Receipts | 4h | Structured JSON receipt per request: request_id, pii_detected, model_route, human_approval, final_status |
| S2-A07 | Integration tests | 4h | Test each endpoint, router rules, HITL flow, receipts (with MockAdapter) |
| S2-A08 | CLI demo script | 2h | `demo.py` that exercises the full pipeline interactively |

**Total: 33h**

---

## Option B — Android Integration (Priority 2)

### Goal
Wire the existing Rust FFI into the Android app and run inference on-device.

### Tickets

| ID | Title | Effort | Description |
|:---|:---|:---:|:---|
| S2-B01 | Build native .so for Android (arm64) | 4h | Cross-compile eq-ffi via cargo-ndk, copy to jniLibs |
| S2-B02 | Wire JNA bridge in Kotlin | 6h | Call Rust `process_user_input()` from Kotlin via generated UniFFI bindings |
| S2-B03 | Build Compose chat UI shell | 6h | Text input, message list, send button, debug overlay showing EQ State |
| S2-B04 | Wire full pipeline: UI → Rust → display | 4h | Type text → JNA → Rust → EQ State JSON → parsed → displayed |
| S2-B05 | On-device inference test | 3h | Run phi-4-mini on Android emulator or device, measure latency |
| S2-B06 | Integration test (instrumented) | 3h | Inject text, assert EQ State returned, validate schema |

**Total: 26h**

---

## Option C — MCP Bridge (Priority 3)

### Goal
Expose EQ State as MCP tools so cloud AI can consume emotional context privately.

### Tickets

| ID | Title | Effort | Description |
|:---|:---|:---:|:---|
| S2-C01 | Define MCP tool schemas | 3h | `get_eq_state`, `get_response_policy`, `get_anonymized_context` — JSON Schema per tool |
| S2-C02 | Build MCP server | 6h | FastMCP or stdio-based MCP server exposing EQ analysis tools |
| S2-C03 | Wire to Agent Gateway | 2h | MCP server calls the same backend as the gateway, shares state |
| S2-C04 | Test with MCP inspector | 3h | Validate tool calls return schema-compliant output |

**Total: 14h**

---

## Sprint Summary

| Track | Tickets | Hours |
|:---|:---:|:---:|
| Option A — Agent Gateway | 8 | 33h |
| Option B — Android Integration | 6 | 26h |
| Option C — MCP Bridge | 4 | 14h |
| **Total** | **18** | **73h** |

---

## Definition of Done (Sprint 2)

- [ ] FastAPI agent gateway runs on `:8090`, `POST /chat` returns EQ State + response
- [ ] HITL gate pauses on high-risk payloads, resumes on approval
- [ ] Observability receipts emitted per request (validated JSON)
- [ ] Android app calls Rust engine and displays EQ State overlay
- [ ] MCP server exposes EQ State tools, validated with MCP inspector
- [ ] All Python tests pass (pytest)
- [ ] All Rust tests pass (78+)
- [ ] Sprint review doc written
