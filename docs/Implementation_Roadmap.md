# 05 — Implementation Roadmap

**Project:** EQ Gateway  
**Status:** Draft v0.1  
**Date:** May 21, 2026

---

## 1. Roadmap Summary

EQ Gateway should be built in staged layers:

```text
0. Research prototype
1. Local EQ Engine
2. Privacy transform
3. MCP bridge
4. Mobile product shell
5. Evaluation harness
6. SDK / developer release
7. Consumer or enterprise productization
```

The product should not start as a full companion app. It should start as a narrow proof:

> Raw private text goes in. Privacy-safe EQ State comes out. A larger AI responds better without receiving raw private context.

---

## 2. Phase 0 — Concept Prototype

## 2.1 Goal

Prove that a small local model can produce useful structured EQ State JSON.

## 2.2 Build

```text
desktop or Android test harness
one local 1B–3B model
static Top Hat prompt
sample emotional messages
strict JSON parser
manual comparison of outputs
```

## 2.3 Deliverables

```text
eq_state_v0.1.json examples
Top Hat v0.1
test prompt set
schema validator
before/after sample cloud prompts
```

## 2.4 Success Criteria

```text
90–95% valid JSON
reasonable affect/intent classification
no obvious raw PII in anonymized summary
latency acceptable on target device
```

## 2.5 Do Not Build Yet

```text
full app UI
long-term memory
cloud relay
enterprise dashboard
multi-model selection
background processing
```

---

## 3. Phase 1 — Local EQ Engine

## 3.1 Goal

Run the EQ inference pipeline fully on-device.

## 3.2 Build

```text
Android app shell
local model runtime
model loading/unloading
prompt compiler
Top Hat injection
EQ State generator
JSON validator
retry/fallback logic
```

## 3.3 Runtime Candidates

```text
LiteRT / Google AI Edge
llama.cpp / GGUF
MLC-LLM
native platform runtime where useful
```

## 3.4 Device Targets

Initial:

```text
Galaxy S22 Ultra or similar Android flagship
```

Later:

```text
modern iPhone Pro
midrange Android
older fallback device
```

## 3.5 Success Criteria

```text
stable first-token latency
stable generation rate
valid structured output
controlled context growth
no thermal collapse during short sessions
```

---

## 4. Phase 2 — Privacy Transform

## 4.1 Goal

Ensure the local output is safe to expose externally.

## 4.2 Build

```text
PII detection
redaction pass
rare-detail generalization
privacy-tier classifier
summary regeneration if leakage found
local audit record
```

## 4.3 Redaction Targets

```text
names
locations
organizations
emails
phone numbers
addresses
exact dates
project names
client names
medical identifiers
legal identifiers
financial account details
rare combinations of facts
```

## 4.4 Success Criteria

```text
PII leakage tests pass
anonymized summary remains useful
privacy tier assigned consistently
raw text never shared by default
```

---

## 5. Phase 3 — MCP Bridge

## 5.1 Goal

Expose EQ State to a larger AI through governed MCP tools.

## 5.2 Build

```text
MCP server or facade
get_eq_state tool
get_response_policy tool
get_anonymized_context tool
request_more_context tool
short-lived payload storage
payload TTL
tool audit log
```

## 5.3 Architecture Choice

For MVP:

```text
Phone computes payload.
Bridge exposes temporary MCP tools.
Bridge stores no raw history.
```

Avoid:

```text
cloud model connects directly to phone
phone exposes full memory
raw data stored on relay
```

## 5.4 Success Criteria

```text
large AI can call tools
tools return schema-valid output
payload expires correctly
forbidden requests are denied
user can see what was shared
```

---

## 6. Phase 4 — Demo App

## 6.1 Goal

Make the concept understandable to a user or investor.

## 6.2 Three-Panel Demo

```text
Panel 1: Original local message
Panel 2: What stays on device / what is shared
Panel 3: Large AI response adapted by EQ State
```

## 6.3 Required UI

```text
privacy tier selector
EQ State preview
anonymized summary preview
send/approve button
audit log
local-only mode
```

## 6.4 Success Criteria

```text
user understands privacy flow in under one minute
demo shows clear difference between raw prompt and EQ-mediated prompt
no therapy/diagnosis positioning
```

---

## 7. Phase 5 — Evaluation Harness

## 7.1 Goal

Make the system testable and defensible.

## 7.2 Build

```text
PII leakage tests
schema validity tests
mood classification tests
intent classification tests
risk detection tests
prompt injection tests
latency benchmarks
battery/thermal benchmarks
response-policy adherence tests
```

## 7.3 Success Criteria

```text
automated regression suite
release gates
baseline metrics
model/prompt comparison reports
```

---

## 8. Phase 6 — Cross-Platform

## 8.1 Goal

Move from Android prototype to Android/iOS architecture.

## 8.2 Build

```text
runtime adapter interface
Android runtime adapter
iOS runtime adapter
shared schema package
shared Top Hat package
shared privacy policy package
device-tier detection
model download manager
```

## 8.3 Success Criteria

```text
same EQ State schema on both platforms
device-specific runtime behind common interface
fallback model for weaker devices
consistent privacy behavior
```

---

## 9. Phase 7 — SDK / Developer Release

## 9.1 Goal

Make EQ Gateway embeddable in other AI apps.

## 9.2 Build

```text
developer docs
schema package
MCP server package
sample app
SDK wrapper
test harness
example integrations
```

## 9.3 SDK API Sketch

```ts
const eq = await EQGateway.initialize({
  model: "local-3b-q4",
  privacyDefault: "anonymized_summary",
  mcp: true
});

const state = await eq.analyze({
  message: userMessage,
  task: "assistant_response"
});

const payload = await eq.buildMcpPayload({
  privacyTier: "tier_2_anonymized_summary"
});
```

## 9.4 Success Criteria

```text
developer can integrate in one afternoon
sample app works
schemas are documented
privacy guarantees are explicit
```

---

## 10. Phase 8 — Productization

## 10.1 Consumer Option

```text
private reflective companion
local mood/EQ engine
cloud handoff by consent
local-only journaling
what-was-shared panel
```

## 10.2 Developer Option

```text
MCP-compatible EQ server
mobile SDK
test harness
optimized local model packages
```

## 10.3 Enterprise Option

```text
privacy-preserving human-context layer
policy controls
audit dashboard
no employee surveillance positioning
```

---

## 11. Suggested Milestones

## Milestone A — Working Local EQ JSON

```text
Input message → local model → valid EQ State JSON
```

## Milestone B — Safe Anonymized Summary

```text
Input with PII → anonymized summary → leakage test pass
```

## Milestone C — MCP Tool Demo

```text
Large AI calls get_eq_state() and get_anonymized_context()
```

## Milestone D — User-Visible Privacy Demo

```text
User sees what stays local and what is shared
```

## Milestone E — Cross-Platform Runtime

```text
Same schema on Android and iOS
```

## Milestone F — Evaluation Gate

```text
Automated tests block unsafe prompt/model changes
```

---

## 12. Technical Debt to Avoid

```text
hardcoding one model
putting privacy rules only in prompts
allowing raw export for convenience
building a giant system prompt
using cloud to perform anonymization
storing relay payloads permanently
making MCP tools too broad
skipping evaluation tests
claiming full anonymity
positioning as therapy
```

---

## 13. First 30 Days

```text
Write EQ State schema v0.1.
Write Top Hat v0.1.
Build local structured-output test harness.
Create 50–100 test messages.
Run local model on device.
Measure context, latency, and JSON validity.
Add simple PII scanner.
Produce first anonymized summaries.
Create before/after demo prompts.
```

---

## 14. First 60 Days

```text
Build Android prototype.
Add privacy-tier selector.
Add local audit log.
Build MCP server/facade.
Expose get_eq_state and get_anonymized_context.
Run first cloud-model integration.
Create demo video/screenshots.
Start evaluation harness.
```

---

## 15. First 90 Days

```text
Stabilize local runtime.
Add response policy tool.
Add request_more_context flow.
Implement post-filter.
Add iOS feasibility prototype.
Publish design docs.
Package developer demo.
Start open-source schema/MCP contract.
```

---

## 16. Final Roadmap Principle

Build the smallest system that proves this:

```text
A small local model can protect, structure, and transmit emotional context better than sending raw user text to a large model.
```
