# Market Research and Product Placement — Placeholder: **EQ Gateway**

**Status:** Research / Positioning Draft  
**Date:** May 21, 2026  
**Version:** 0.1  
**Companion Document:** `EQ_Gateway_Design_Document_v0.1.md`  
**Working Product Name:** EQ Gateway  
**Working Category:** Privacy-preserving affective middleware for AI systems

---

## 1. Executive Summary

EQ Gateway sits at the intersection of five current AI categories:

```text
1. Local/on-device small language models
2. Privacy-preserving prompt mediation
3. PII redaction/anonymization for LLMs
4. AI companions, journaling, and mood-tracking tools
5. MCP-based tool/context brokering
```

Search results show multiple adjacent systems, but no exact equivalent to the proposed architecture:

```text
phone-local SLM
+ EQ/mood/intent/risk inference
+ privacy-preserving anonymization
+ structured emotional-state payload
+ MCP bridge
+ larger cloud AI receives only constrained emotional metadata
```

The closest existing work focuses on **local PII redaction before cloud LLM use**. Other adjacent products focus on **fully local AI companions** or **mood tracking**, while MCP provides the tool interface but not the affective privacy layer.

The strongest product placement is therefore:

> EQ Gateway is not another AI companion. It is a local emotional privacy layer that lets larger AI systems adapt to a user’s state without receiving the user’s raw private context.

---

## 2. Core Market Hypothesis

AI assistants are becoming more personal, emotionally adaptive, and context-aware. That creates a tension:

```text
Better personalization usually requires more personal data.
More personal data increases privacy, trust, compliance, and safety risk.
```

EQ Gateway resolves this tension by moving the most sensitive inference layer onto the user’s device.

The product hypothesis:

> A phone-local SLM can translate raw private human context into structured, anonymized emotional metadata, allowing larger AI systems to respond better without centralizing the user’s emotional history.

This creates a product category around **private emotional context brokering**.

---

## 3. Category Map

### 3.1 Existing Categories

| Category | Description | Relation to EQ Gateway |
|---|---|---|
| Local AI companion apps | Full companion experiences running partly or fully on-device. | Emotional UX overlap, but not primarily middleware. |
| Mood tracking apps | User-entered or inferred mood logs, often wellbeing-focused. | Mood-domain overlap, but not LLM gateway architecture. |
| PII redaction proxies | Detect/redact private information before sending prompts to cloud LLMs. | Strong technical overlap, but usually not EQ/affective. |
| Privacy-preserving prompt rewriting | Local or private models rewrite prompts for safer cloud use. | Strong privacy overlap, but not structured EQ signal. |
| MCP servers/tools | Standardized way to expose context/tools to AI apps. | Infrastructure overlap; EQ Gateway uses MCP as the bridge. |
| Affective computing research | Emotion detection, sentiment analysis, affect modeling. | Technique overlap, not full product architecture. |

### 3.2 Proposed Category

```text
Privacy-preserving affective middleware
```

Alternative category labels:

```text
Local EQ gateway
On-device emotional context broker
Affective privacy firewall
Human-context mediation layer
Private affective AI infrastructure
```

The most precise technical category:

> On-device affective context mediation for cloud-connected LLM systems.

---

## 4. Closest Adjacent Systems

## 4.1 Local AI Proxy for PII Redaction

A close architectural cousin is the local AI proxy pattern. LogRocket describes a proxy that sits between an application and a cloud LLM, detects sensitive values, replaces them with placeholders, sends the sanitized prompt to the cloud model, and restores original values afterward.

Source:  
https://blog.logrocket.com/build-local-ai-proxy-redact-pii-before-llms/

### Similarity

```text
Local layer sits between user/app and cloud model.
Sensitive data is processed before the cloud call.
The system avoids choosing between weak local-only AI and powerful cloud-only AI.
```

### Difference

```text
PII proxy focuses on redaction and rehydration.
EQ Gateway focuses on emotional-state extraction, privacy-tiered context sharing, and response-policy metadata.
```

### Product Implication

This category validates the need for an intermediary privacy layer. EQ Gateway should not compete head-on as a generic PII scrubber. It should position as a higher-level **emotional context gateway** that includes redaction as one subsystem.

---

## 4.2 Anonymizer SLM / PAPILLON-Style Local Prompt Rewriting

Hugging Face’s Anonymizer SLM writeup discusses lightweight on-device models for privacy-first PII replacement. It references PAPILLON, which rewrites user queries locally into privacy-preserving prompts before sending them to stronger API models. The article notes that local rewriting can preserve privacy but may still leak private information or reduce response quality.

Source:  
https://huggingface.co/blog/pratyushrt/anonymizerslm

### Similarity

```text
Local SLM protects user privacy before cloud model interaction.
The large model receives a transformed prompt.
The small model acts as a privacy buffer rather than a full assistant.
```

### Difference

```text
Anonymizer SLM focuses on PII replacement.
EQ Gateway adds affect, intent, risk, tone, privacy tier, and MCP contract.
```

### Product Implication

This is one of the strongest pieces of prior art. EQ Gateway should acknowledge the anonymization-SLM pattern and differentiate around:

```text
structured EQ State
MCP tool exposure
emotional response policy
user-visible privacy tiers
local affective memory
risk-aware routing
```

---

## 4.3 Whistledown — Client-Side Privacy for Emotionally Nuanced Conversations

Whistledown is highly relevant because it explicitly addresses emotionally charged and socially sensitive LLM conversations. It focuses on protecting PII while preserving conversational coherence through a client-side privacy system.

Source:  
https://arxiv.org/html/2511.13319v1  
PDF: https://arxiv.org/pdf/2511.13319

### Similarity

```text
Recognizes that emotionally charged LLM conversations create privacy risks.
Operates client-side.
Protects PII before prompts reach cloud-hosted LLMs.
Attempts to preserve conversational utility.
```

### Difference

```text
Whistledown is primarily a privacy/coherence layer.
EQ Gateway treats emotional state as a structured first-class output.
EQ Gateway uses MCP as a governed interface.
EQ Gateway turns mood/intent/risk into response-control metadata.
```

### Product Implication

Whistledown validates the market need: users disclose sensitive information in emotional/social conversations. EQ Gateway’s differentiation is to produce a **usable emotional state interface**, not only a privacy-preserved transformed prompt.

---

## 4.4 Contextual Privacy in Conversational Agents

A 2025 ACL Findings paper, “Safeguarding Contextual Privacy in Interactions with Conversational Agents,” argues that privacy is not merely about sensitive terms, but about whether information is appropriate and necessary in context. It proposes privacy mediation between users and conversational agents.

Source:  
https://aclanthology.org/2025.findings-acl.1343.pdf

### Similarity

```text
Middleware layer manages what context reaches the AI.
Privacy is contextual rather than just keyword-based.
Sensitive information should be disclosed only when task-relevant.
```

### Difference

```text
The research is broader contextual privacy mediation.
EQ Gateway is a concrete product architecture centered on local SLM EQ inference and MCP.
```

### Product Implication

This supports the “Librarian” and “context compiler” design. EQ Gateway should avoid claiming that privacy is solved by redaction alone. The better claim is:

```text
The system minimizes, redacts, generalizes, and contextually gates what leaves the device.
```

---

## 4.5 MCP — Model Context Protocol

MCP is an open standard for connecting AI applications to tools and external systems. Official MCP materials describe it as a way for AI applications to access data sources, tools, and workflows through standardized interfaces.

Sources:  
https://modelcontextprotocol.io/docs/getting-started/intro  
https://anthropic.com/news/model-context-protocol  
https://modelcontextprotocol.io/specification/2025-06-18/server/resources

### Similarity

```text
EQ Gateway uses MCP as the bridge between local state and larger AI systems.
MCP supports controlled resources/tools.
MCP makes the architecture model-provider independent.
```

### Difference

```text
Most MCP servers expose files, databases, APIs, local tools, or workflows.
EQ Gateway would expose privacy-preserving emotional state.
```

### Product Implication

MCP gives EQ Gateway a credible integration path. The product should define itself as:

```text
an MCP-compatible local emotional context server
```

That makes the idea immediately legible to AI developers.

---

## 4.6 Local AI Companions and Mood Apps

Cassie markets itself as an on-device AI companion with private memory storage, journaling, reminders, mood/tone tracking, no account, and no data tracking.

Source:  
https://apps.apple.com/ca/app/cassie-your-ai-companion/id6746135777

Mood Tracker / AI companion apps on mobile stores offer mood logging, journaling, and AI companion experiences.

Example source:  
https://play.google.com/store/apps/details?id=com.brilliant.moodtracker.ai.journal.daily.app

### Similarity

```text
Emotional-support use case.
Mood and journaling domain.
Mobile-first interaction.
Privacy can be part of the pitch.
```

### Difference

```text
These apps are generally end-user companions.
EQ Gateway is an infrastructure layer and privacy-preserving gateway.
```

### Product Implication

Consumer companion apps are both competition and potential customers. EQ Gateway could be:

```text
A standalone companion app
A privacy layer inside a companion app
An SDK for companion apps
An MCP server used by companion workflows
```

The stronger differentiated play is probably SDK/MCP/infrastructure first, then consumer app as a demo.

---

## 4.7 Enterprise PII Filtering for AI

Enterprise tools increasingly add PII filtering before LLM prompts. Gravitee, for example, describes a PII Filtering Policy intended to detect, redact, or block sensitive data before it reaches an LLM.

Source:  
https://www.gravitee.io/blog/how-to-prevent-pii-leaks-in-ai-systems-automated-data-redaction-for-llm-prompt

### Similarity

```text
Pre-LLM privacy filtering.
Data governance angle.
Enterprise compliance concern.
```

### Difference

```text
Enterprise filters are usually gateway/API-layer security controls.
EQ Gateway is personal-device-first and affect-aware.
```

### Product Implication

Enterprise buyers may understand the privacy problem quickly, but the emotional/EQ angle must be positioned carefully. For enterprise, the pitch should be:

```text
private human-context mediation for AI copilots
```

Not:

```text
employee mood surveillance
```

This distinction is critical.

---

## 5. Competitive Landscape Summary

| Category | Examples / Sources | Overlap | Threat Level | EQ Gateway Differentiation |
|---|---|---:|---:|---|
| Local PII proxy | LogRocket local proxy, enterprise PII filtering | High | High | Adds EQ State, risk, tone policy, MCP, local affective memory. |
| Anonymizer SLM | Hugging Face Anonymizer SLM, PAPILLON-style rewriting | High | High | Uses structured emotional metadata, not just rewritten prompt. |
| Client-side privacy layer | Whistledown | High | Medium-High | Treats affect as first-class interface; MCP-native. |
| Contextual privacy research | ACL contextual privacy work | Medium-High | Medium | Turns concept into mobile/MCP product architecture. |
| Local AI companions | Cassie, mood/journal companions | Medium | Medium | Middleware/SDK rather than only companion UX. |
| Mood tracking apps | AI mood journals, wellness trackers | Medium | Medium | Routes mood as privacy-safe AI control signal. |
| MCP ecosystem | MCP servers broadly | Medium | Low-Medium | Specializes MCP around local emotional context. |
| Enterprise PII/security gateways | Gravitee-style filters | Medium | Medium | Personal device layer; emotional context rather than generic compliance. |

---

## 6. What Appears Novel

The individual components are not novel by themselves.

Existing market/research already includes:

```text
on-device AI
local prompt anonymization
PII redaction
mood tracking
AI companions
MCP servers
privacy-preserving LLM workflows
```

The novel product architecture is the combination:

```text
1. Local SLM as an EQ engine, not a full assistant.
2. Emotional context converted into strict structured metadata.
3. Raw emotional history remains local by default.
4. Larger AI receives mood/intent/risk/response-policy payloads.
5. MCP exposes governed emotional context tools.
6. User can inspect what was shared.
7. Privacy tiers decide whether metadata, summary, excerpt, or raw text leaves the device.
```

The most defensible phrase:

> EQ Gateway is an on-device emotional context broker for AI systems.

The sharper technical phrase:

> A phone-local SLM that transforms private user input into privacy-preserving affective metadata exposed through MCP.

---

## 7. Product Placement

## 7.1 Placement Option A — Consumer Companion

### Description

A mobile app that helps users journal, reflect, and get assistance from larger AI models while keeping raw emotional context local.

### Pitch

```text
A private AI companion that keeps your emotional context on your phone.
```

### Pros

```text
Easy to demo
Emotionally intuitive
Clear user value
Can prove the trust model directly
```

### Cons

```text
Crowded companion/wellness market
Risk of being compared to therapy apps
App-store policy and safety concerns
Harder to monetize without strong brand trust
```

### Recommended if

```text
You need a clear public demo.
You want to prove user-facing trust controls.
You want to show “what was shared” as a visible differentiator.
```

---

## 7.2 Placement Option B — Developer SDK

### Description

A mobile SDK that AI app developers can embed to infer local EQ state, anonymize context, and pass structured emotional metadata to cloud models.

### Pitch

```text
Add privacy-preserving emotional context to your AI app without collecting raw user history.
```

### Pros

```text
Stronger technical differentiation
Avoids direct competition with companion apps
Developers understand API/schema value
Can become infrastructure
```

### Cons

```text
Requires documentation and support
Harder to demo emotionally
Must prove runtime reliability across devices
```

### Recommended if

```text
The goal is defensible infrastructure.
The product is aimed at AI builders.
The MCP angle is central.
```

---

## 7.3 Placement Option C — MCP Server / Local Context Service

### Description

A local MCP server that exposes EQ State, response policy, anonymized context, and privacy-tier tools to compatible AI clients.

### Pitch

```text
An MCP server for local, privacy-preserving emotional context.
```

### Pros

```text
Clear fit for current AI developer workflows
Fast to prototype
Model/provider independent
Differentiates from normal MCP file/database tools
```

### Cons

```text
MCP ecosystem still evolving
Security model must be rigorous
Consumer users may not understand MCP
```

### Recommended if

```text
You want the cleanest technical proof of concept.
You want to plug into Claude Desktop, local agents, or other MCP-compatible clients.
```

---

## 7.4 Placement Option D — Enterprise Human-Context Privacy Layer

### Description

An enterprise product that allows AI copilots to adapt to user state without sending raw employee context to cloud AI systems.

### Pitch

```text
Give AI copilots better human-context signals without exposing sensitive employee data.
```

### Pros

```text
Privacy/compliance budget exists
Clear concern around AI data leakage
Potentially high-value buyer
```

### Cons

```text
EQ/mood inference can sound like surveillance
Requires serious governance, opt-in, auditability, and legal review
Long sales cycles
```

### Recommended if

```text
Positioned as user-controlled privacy middleware, not employer mood analytics.
```

---

## 8. Recommended Product Path

The strongest path is staged:

```text
1. Technical MVP as local MCP EQ server
2. Demo app showing privacy tiers and “what was shared”
3. Developer SDK around EQ State schema
4. Optional consumer companion experience
5. Enterprise offering only after governance is mature
```

### Why this order

The MCP server proves the architecture.

The demo app proves the human trust story.

The SDK proves portability.

The consumer companion provides a visible product.

Enterprise can come later, but only with careful positioning.

---

## 9. Positioning Statement

### 9.1 Developer Positioning

> EQ Gateway is a local SLM-powered emotional context broker that converts private user input into anonymized affect, intent, risk, and response-policy metadata for use by larger AI systems through MCP.

### 9.2 Consumer Positioning

> EQ Gateway keeps your emotional context on your device and shares only privacy-safe signals when outside AI help is needed.

### 9.3 Enterprise Positioning

> EQ Gateway gives AI systems useful human-context signals while minimizing exposure of sensitive employee or customer data.

### 9.4 Research Positioning

> EQ Gateway explores local affective mediation as an alternative to centralized emotional-data collection in LLM systems.

---

## 10. Messaging Pillars

### Pillar 1 — Local Emotional Privacy

```text
The emotionally sensitive layer stays on the device by default.
```

### Pillar 2 — Structured EQ, Not Raw Exposure

```text
The cloud model receives mood, intent, risk, and response-policy metadata—not raw emotional history.
```

### Pillar 3 — User-Controlled Sharing

```text
Users can see and control what leaves the device.
```

### Pillar 4 — Model-Swappable Infrastructure

```text
The architecture is not tied to one local model or one cloud provider.
```

### Pillar 5 — MCP-Native Integration

```text
AI systems can consume EQ state through a governed tool interface.
```

---

## 11. Product Claims: Safe vs Risky

## 11.1 Safer Claims

```text
privacy-preserving
local-first
raw text stays on device by default
data-minimized
user-controlled
anonymized emotional metadata
structured EQ State
MCP-compatible
local SLM-powered
```

## 11.2 Risky Claims

Avoid or heavily qualify:

```text
fully anonymous
impossible to re-identify
emotionally accurate
therapeutic
clinically validated
mental health treatment
detects your true feelings
understands you completely
guaranteed private
```

### Recommended Claim Discipline

Use:

```text
privacy-preserving emotional context
```

Not:

```text
fully anonymous emotional AI
```

Use:

```text
infers likely mood signals
```

Not:

```text
knows how you feel
```

Use:

```text
supports reflective and conversational use
```

Not:

```text
provides therapy
```

---

## 12. Target Segments

## 12.1 Primary Early Segment — AI Power Users / Local AI Users

### Why

They understand:

```text
local models
MCP
privacy risks
cloud/local tradeoffs
context management
```

### Messaging

```text
Your phone becomes the emotional firewall for cloud AI.
```

### Channels

```text
GitHub
Hacker News
Reddit local AI communities
MCP directories
AI builder Discords
LocalLLaMA-style communities
```

---

## 12.2 Secondary Segment — AI App Developers

### Why

They need emotional adaptation but do not want to own raw emotional data.

### Messaging

```text
Add EQ-aware personalization without collecting raw user history.
```

### Channels

```text
SDK docs
example apps
MCP server registry
developer blog posts
open-source demo
```

---

## 12.3 Tertiary Segment — Privacy-Conscious Consumers

### Why

They want AI help but are wary of sending private emotional content to cloud models.

### Messaging

```text
Get better AI help while keeping sensitive context on your device.
```

### Channels

```text
App Store / Play Store
privacy-focused blogs
personal AI communities
journaling/wellbeing audiences
```

---

## 12.4 Later Segment — Enterprise AI Governance

### Why

Enterprises are concerned about sensitive data entering LLM workflows.

### Messaging

```text
Human-context aware AI without unnecessary sensitive-data exposure.
```

### Caution

Do not position as employee mood monitoring. It must be opt-in, user-controlled, and privacy-preserving.

---

## 13. Competitive Strategy

### 13.1 Do Not Compete as Another Chatbot

The market already has many AI companions. Competing as “another emotionally aware chatbot” is weak.

Better:

```text
the privacy layer underneath emotionally adaptive AI
```

### 13.2 Do Not Compete as a Generic PII Redactor

PII redaction is becoming a feature in gateways, API management, and enterprise security tools.

Better:

```text
PII redaction + emotional state schema + MCP response policy
```

### 13.3 Own the EQ State Contract

The most defensible artifact is the schema and tool contract.

Possible moat:

```text
EQ State schema
privacy-tier framework
evaluation harness
local emotional memory model
MCP tool definitions
developer adoption
```

### 13.4 Open the Protocol, Monetize the Runtime

A strong strategy:

```text
Open-source:
  schema
  MCP tool definitions
  test suite
  basic local server

Commercialize:
  mobile SDK
  optimized models
  evaluation harness
  enterprise policy controls
  polished app
```

---

## 14. SWOT Analysis

## 14.1 Strengths

```text
Clear privacy-first differentiation
Fits emerging MCP ecosystem
Uses local SLMs for a bounded task
Avoids forcing phone model to be full assistant
Can work with many cloud AI providers
User-visible “what was shared” trust mechanism
Strong conceptual framing: emotional firewall
```

## 14.2 Weaknesses

```text
Mood inference may be unreliable
Anonymization can leak rare details
Cross-platform inference is complex
Consumer messaging can be misunderstood as therapy
MCP security concerns require careful implementation
May be hard to explain quickly to nontechnical users
```

## 14.3 Opportunities

```text
Growing local AI capability on phones
Rising concern over AI privacy
MCP adoption by major AI tools
Need for better AI personalization without raw data centralization
Potential SDK market for AI companion and journaling apps
Potential enterprise privacy/compliance market
```

## 14.4 Threats

```text
Large AI platforms may add native privacy/EQ features
Mobile OS vendors may offer local AI personalization APIs
PII gateway vendors may expand into affective metadata
Regulatory scrutiny around emotional inference
App Store policy concerns if framed as mental health product
MCP vulnerabilities or ecosystem instability
```

---

## 15. Regulatory and Trust Considerations

### 15.1 Sensitive Category Risk

Emotional state inference can become sensitive quickly, especially when connected to:

```text
mental health
relationships
workplace stress
medical situations
children/family
legal conflict
financial distress
```

The product should default to:

```text
local-only raw data
user inspection
explicit consent for sharing
no diagnosis
no clinical claims
```

### 15.2 Enterprise Risk

Enterprise use is attractive but dangerous if mishandled. A workplace version must not become:

```text
mood surveillance
productivity emotion scoring
employee risk ranking
```

Acceptable enterprise framing:

```text
user-controlled privacy mediation for AI copilots
```

Not acceptable framing:

```text
management insight into employee emotion
```

### 15.3 User Trust Requirements

The app must be able to answer:

```text
What did you infer?
What left my device?
Why did it leave?
Which model/service received it?
Can I delete it?
Can I turn this off?
Can I use local-only mode?
```

---

## 16. MCP Security Placement

MCP is useful but should not be treated as automatically safe.

Recent security commentary has raised concerns about MCP server implementations and tool execution risk. Even without relying on any single report, EQ Gateway should assume MCP is a privileged boundary and design conservatively.

Practical requirements:

```text
No arbitrary shell execution
No broad filesystem access
No raw local memory export tool
Strict tool allowlist
Schema-validated inputs/outputs
Explicit consent for sensitive escalation
Local audit log
Per-client permissions
Rate limits
Transport hardening
```

MCP is the interface. It should not be the trust boundary by itself.

---

## 17. Go-To-Market Narrative

### 17.1 Developer Launch Narrative

Title:

```text
Your local EQ engine for MCP-enabled AI.
```

Subtitle:

```text
Convert private user context into structured, anonymized emotional metadata before larger models respond.
```

Demo:

```text
Input: raw emotional message
Local output: EQ State JSON
Shared output: anonymized summary + response policy
Cloud output: better answer without raw private text
```

### 17.2 Blog Post Angles

```text
1. The small model should not be the assistant — it should be the emotional firewall.
2. Why redacting PII is not enough for emotionally adaptive AI.
3. MCP needs human-context servers, not just file and database connectors.
4. The case for local affective middleware.
5. How to pass mood to a large AI without passing the person.
```

### 17.3 Demo Video Storyboard

```text
Scene 1:
  User writes a sensitive workplace message.

Scene 2:
  Phone-local EQ Engine extracts mood/intent/risk.

Scene 3:
  Privacy panel shows:
    - stayed on device
    - shared with larger AI
    - privacy tier

Scene 4:
  Larger AI receives only anonymized EQ State.

Scene 5:
  Response is calmer, more useful, and context-aware.

Scene 6:
  User opens audit log and sees exactly what was sent.
```

---

## 18. Pricing / Business Model Options

## 18.1 Open-Core Developer Product

```text
Free:
  EQ State schema
  basic MCP server
  test prompts
  simple local runtime

Paid:
  mobile SDK
  optimized inference
  privacy evaluation suite
  enterprise policy controls
  dashboard
  support
```

## 18.2 Consumer App

```text
Free:
  local-only basic companion
  small model
  limited cloud handoff

Paid:
  larger model support
  advanced privacy dashboard
  encrypted local memory
  multiple top hats
  cross-device sync if ever implemented
```

## 18.3 Enterprise

```text
Per-seat or per-device licensing
Governance dashboard
Policy templates
Audit exports
Private deployment support
Security review support
```

Recommended initial business model:

```text
open-source protocol + paid SDK/tooling
```

This aligns with developer adoption and avoids needing to win the consumer companion market immediately.

---

## 19. Product Roadmap Placement

## 19.1 Version 0.1 — Proof of Concept

```text
Local SLM produces valid EQ State JSON
Basic anonymizer
MCP tool exposes state
Manual cloud prompt uses state
Demo panel shows shared vs local data
```

## 19.2 Version 0.2 — Developer Demo

```text
Installable MCP server
Schema package
Example prompts
PII leakage tests
Basic docs
One Android runtime path
```

## 19.3 Version 0.3 — Mobile Prototype

```text
Android app
Local model manager
Privacy tier selector
“What was shared” panel
Local audit log
Cloud handoff demo
```

## 19.4 Version 0.4 — Cross-Platform

```text
iOS runtime path
Shared schema/context compiler
Device-tier fallback
MCP bridge hardening
```

## 19.5 Version 1.0 — Product Candidate

```text
Public SDK or app
Evaluation harness
Privacy dashboard
Stable EQ State schema
Model-swappable runtime
MCP server release
Clear non-clinical positioning
```

---

## 20. Differentiation Checklist

For every demo, document, or pitch, make sure these points are visible:

```text
The local model is not trying to replace the cloud model.
The local model protects and structures emotional context.
The cloud model receives metadata, not raw emotional history.
MCP is used as the governed bridge.
The user can inspect what was shared.
The schema is stable and model-agnostic.
The product is not therapy or diagnosis.
The privacy layer is more than PII redaction.
```

If those points are not visible, the product will be mistaken for:

```text
another mood tracker
another AI companion
another PII redactor
another local chatbot
```

---

## 21. Final Placement Recommendation

The best market placement is:

```text
Developer-first, MCP-native, privacy-preserving affective middleware.
```

A consumer app can exist, but it should be treated as a demonstration of the infrastructure rather than the only product.

Best short pitch:

> EQ Gateway lets a phone-local SLM convert private emotional context into anonymized EQ metadata, so larger AI systems can respond with the right tone without seeing the user’s raw private data.

Best technical pitch:

> An MCP-compatible local affective context broker with privacy-tiered disclosure, structured EQ State, and on-device SLM inference.

Best strategic framing:

> The future of emotionally adaptive AI should not require centralized emotional surveillance.

---

## 22. Source List

The following sources informed this market-placement draft:

1. LogRocket — local AI proxy and PII redaction before LLM calls  
   https://blog.logrocket.com/build-local-ai-proxy-redact-pii-before-llms/

2. Hugging Face — Anonymizer SLM and PAPILLON-style local prompt rewriting  
   https://huggingface.co/blog/pratyushrt/anonymizerslm

3. Whistledown — client-side privacy system for emotionally nuanced LLM conversations  
   https://arxiv.org/html/2511.13319v1  
   https://arxiv.org/pdf/2511.13319

4. ACL Findings 2025 — Safeguarding contextual privacy in conversational agents  
   https://aclanthology.org/2025.findings-acl.1343.pdf

5. Model Context Protocol official documentation  
   https://modelcontextprotocol.io/docs/getting-started/intro  
   https://modelcontextprotocol.io/specification/2025-06-18/server/resources

6. Anthropic announcement of MCP  
   https://anthropic.com/news/model-context-protocol

7. Cassie — on-device AI companion with private memory and mood/tone tracking  
   https://apps.apple.com/ca/app/cassie-your-ai-companion/id6746135777

8. Mood Journal / AI mood tracker app example  
   https://play.google.com/store/apps/details?id=com.brilliant.moodtracker.ai.journal.daily.app

9. Gravitee — PII filtering policy before LLM prompts  
   https://www.gravitee.io/blog/how-to-prevent-pii-leaks-in-ai-systems-automated-data-redaction-for-llm-prompt
