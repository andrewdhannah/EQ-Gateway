# 07 — Pitch and Positioning

**Project:** EQ Gateway  
**Status:** Draft v0.1  
**Date:** May 21, 2026

---

## 1. One-Sentence Pitch

EQ Gateway lets a phone-local SLM convert private emotional context into anonymized EQ metadata, so larger AI systems can respond with the right tone without seeing the user’s raw private data.

---

## 2. Short Pitch

Most AI personalization requires sending more personal data to bigger models.

EQ Gateway takes the opposite approach.

It runs a small language model locally on the phone to infer mood, intent, risk, and response preference. The phone then sends only structured, privacy-safe emotional metadata through MCP to a larger AI model.

The large model gets enough context to respond well. It does not get the user’s raw emotional history by default.

The phone becomes the emotional privacy gateway.

---

## 3. Technical Pitch

EQ Gateway is an MCP-compatible local affective context broker.

It uses an on-device SLM to transform raw user input into a strict EQ State schema containing affect, intent, risk, privacy tier, and response policy. The local privacy layer redacts and generalizes sensitive details before exposing the state through narrow MCP tools.

The larger AI receives structured emotional metadata and an anonymized context summary, not open access to local memory.

Core architecture:

```text
Phone-local SLM
+ Privacy transform
+ EQ State schema
+ MCP bridge
+ Cloud reasoning model
+ Local post-filter
```

---

## 4. Founder / Product Narrative

Large AI models are powerful, but they create a problem when used for personal, emotional, or reflective conversations. The more helpful the assistant becomes, the more personal context it wants.

That creates a bad tradeoff:

```text
Better help requires more disclosure.
More disclosure creates more privacy risk.
```

EQ Gateway changes the shape of that problem.

Instead of sending raw emotional context to the cloud, the phone runs a small local model that acts as an EQ engine. It detects the likely mood, intent, risk level, and preferred response style. Then it converts that into a compact structured payload.

The larger model does not need to know the names, places, relationships, or private details. It only needs to know how to respond.

The key idea:

> Pass the mood, not the person.

---

## 5. Product Category

Recommended category:

```text
Privacy-preserving affective middleware
```

Alternative category names:

```text
on-device EQ gateway
local emotional context broker
AI emotional privacy layer
MCP affective context server
private human-context middleware
```

Avoid positioning as:

```text
therapy app
mental health treatment
emotion detector
surveillance tool
generic chatbot
```

---

## 6. Core Messaging Pillars

## 6.1 Keep Emotional Context Local

```text
The most sensitive part of the AI interaction stays on the phone by default.
```

## 6.2 Share Signals, Not Raw History

```text
The larger AI receives structured mood, intent, risk, and response-policy metadata.
```

## 6.3 User Controls the Boundary

```text
The user can see what was inferred and what was shared.
```

## 6.4 MCP-Native

```text
AI systems consume EQ context through governed tools, not raw memory access.
```

## 6.5 Model-Swappable

```text
The product is not tied to one local model, one cloud model, or one vendor.
```

---

## 7. Taglines

```text
Pass the mood, not the person.
```

```text
The emotional firewall for AI.
```

```text
Private context. Better responses.
```

```text
Your phone decides what AI gets to know.
```

```text
Local EQ for cloud AI.
```

```text
Personalized AI without personal-data exposure.
```

```text
A privacy layer for emotionally adaptive AI.
```

Best technical tagline:

```text
An MCP-native EQ gateway for privacy-preserving AI personalization.
```

Best consumer tagline:

```text
Better AI help without sending your private context.
```

---

## 8. The Problem

AI systems are becoming more emotionally adaptive, but emotional adaptation usually requires access to private user context.

That creates problems:

```text
raw emotional disclosure
cloud memory risk
PII leakage
over-personalization
loss of user trust
unclear consent
long-context bloat
```

Existing approaches often solve only part of the problem:

```text
PII redaction removes names but not emotional sensitivity.
Local companions protect privacy but lack larger-model reasoning.
Cloud models reason well but centralize sensitive context.
Mood trackers record state but do not broker AI responses.
```

---

## 9. The Solution

EQ Gateway splits the assistant into two roles:

```text
Local SLM:
  protect, classify, compress, anonymize, route

Large AI:
  reason, draft, plan, explain, generate
```

The bridge is a structured schema:

```text
EQ State = affect + intent + risk + privacy tier + response policy
```

The larger AI adapts its answer using that state.

---

## 10. Why Now

Several trends make this possible:

```text
small open-weight models are now usable on phones
mobile inference is improving
MCP gives AI systems a standard tool interface
users are more aware of AI privacy risks
AI companions are becoming emotionally personal
companies need safer AI context handling
```

The market is moving toward more personalized AI. EQ Gateway argues that personalization should not require centralized emotional surveillance.

---

## 11. Differentiation

EQ Gateway is not:

```text
just a chatbot
just a mood tracker
just a PII redactor
just a local LLM app
just an MCP server
```

It is:

```text
a local emotional privacy gateway
```

The distinctive combination:

```text
on-device SLM
structured EQ State
privacy-tier disclosure
MCP bridge
local emotional memory
cloud reasoning
user-visible audit
```

---

## 12. Audience-Specific Pitches

## 12.1 AI Developer

You are building an AI app that needs to respond with better tone and emotional awareness, but you do not want to collect raw user history.

EQ Gateway gives you a local EQ layer that emits structured affect, intent, risk, and response-policy metadata through MCP.

You get better adaptation with less sensitive data.

## 12.2 Privacy-Conscious User

You want AI help with personal situations, but you do not want to send everything you feel to a cloud model.

EQ Gateway keeps the raw emotional context on your phone and shares only privacy-safe signals when outside AI help is needed.

## 12.3 Enterprise Buyer

Your organization wants AI copilots to be more context-aware without increasing sensitive-data exposure.

EQ Gateway provides a user-controlled local context mediation layer that can minimize what reaches external AI systems.

## 12.4 Research Audience

EQ Gateway explores a local affective mediation architecture where emotional state is converted into structured, privacy-preserving metadata for downstream AI systems.

---

## 13. Demo Script

## 13.1 Setup

Show a user writing a sensitive message:

```text
I am so tired of being blamed for a project that keeps changing. I need help figuring out what to say without making it worse.
```

## 13.2 Local Processing

Show local EQ State:

```json
{
  "affect": "frustrated",
  "intent": "conflict_navigation",
  "risk": "none",
  "response_policy": "calm_direct",
  "privacy_tier": "anonymized_summary"
}
```

## 13.3 Privacy Panel

Show:

```text
Stayed on device:
  original message
  names
  full emotional history

Shared:
  anonymized summary
  mood/intent/risk
  response policy
```

## 13.4 Cloud Response

The larger AI produces a calm practical draft.

## 13.5 Close

Show audit log:

```text
Raw text shared: No
Privacy tier: Anonymized summary
Tools used: get_eq_state, get_response_policy, get_anonymized_context
```

End line:

```text
The larger AI helped without seeing the raw private message.
```

---

## 14. Blog Post Angles

## 14.1 The Small Model Should Not Be the Assistant

Thesis:

```text
The phone-local SLM is most valuable as a control layer, not a weaker chatbot.
```

## 14.2 Redacting PII Is Not Enough

Thesis:

```text
Emotional privacy is not only about names and addresses. It is about what context is appropriate to share.
```

## 14.3 Pass the Mood, Not the Person

Thesis:

```text
Larger AI systems can adapt tone and framing from structured emotional metadata without seeing raw user history.
```

## 14.4 MCP Needs Human-Context Servers

Thesis:

```text
MCP should not only connect AI to files and databases. It can also connect AI to consented, privacy-preserving human context.
```

## 14.5 The Emotional Firewall for AI

Thesis:

```text
The next AI privacy layer should sit where the most sensitive context begins: on the user’s device.
```

---

## 15. Safe Claim Language

Use:

```text
privacy-preserving
local-first
raw text stays on device by default
structured emotional metadata
mood/intent inference
user-controlled sharing
data minimization
MCP-compatible
```

Avoid:

```text
fully anonymous
guaranteed private
knows how you feel
therapeutic
clinical
diagnostic
mental health treatment
emotion surveillance
```

---

## 16. Landing Page Draft

## Hero

```text
Local EQ for cloud AI.
```

## Subhead

```text
EQ Gateway runs a small model on your phone to turn private emotional context into anonymized signals, so larger AI systems can respond better without seeing your raw personal data.
```

## Three Bullets

```text
Keep raw emotional context on-device by default.
Share structured mood, intent, risk, and response-policy metadata.
Connect to larger AI systems through a governed MCP bridge.
```

## CTA Options

```text
View the demo
Read the schema
Try the MCP server
Join the developer preview
```

---

## 17. Investor / Strategic Framing

AI assistants are moving toward deeper personalization. The likely default path is centralized user memory and more cloud-side emotional profiling.

EQ Gateway offers an alternative infrastructure layer:

```text
personalization without raw personal-data exposure
```

The product can become:

```text
a protocol
an SDK
a local mobile runtime
an MCP server
a trust layer for AI companion apps
```

The long-term opportunity is not just one app. It is becoming the privacy boundary between human emotional context and generalized AI reasoning systems.

---

## 18. Final Positioning

Best final phrase:

```text
EQ Gateway is a local emotional privacy gateway for AI.
```

Best technical phrase:

```text
An MCP-compatible on-device affective context broker.
```

Best strategic phrase:

```text
The context boundary should not live in the cloud by default.
```
