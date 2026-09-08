# VIA System Overview

## Responsibility boundary

```text
User
  ↓
Voice / Text Interaction
  ↓
VIA
  ├─ Context grounding
  ├─ Intent refinement
  ├─ Downstream Agent routing
  ├─ Context sharing / consent interaction
  ├─ Conversation / Task lifecycle
  ├─ Progress / status / cancel / follow-up
  └─ Result interaction
        ↓
Downstream Agent
  ├─ Domain reasoning
  ├─ Planning
  ├─ Tool selection
  ├─ Tool execution
  └─ Domain result
        ↓
OS / App / Web / External Service
```

## Current logical flow

```text
Voice/Text Interaction
→ Context Engine
→ Intent Refiner
→ Agent Router
→ Task / Workflow Manager
→ Agent Harness Port
→ Downstream Agent
```

Cross-cutting services:

- Session & Conversation State
- Memory Manager
- Policy / Consent / Identity
- Model Gateway
- Agent Registry
- Observability / Audit / Evaluation
- Notification

## Key architectural invariants

- VIA is Voice-first, not Voice-only.
- Conversation/task state is independent from media connection.
- Downstream Agent owns domain reasoning and tool execution.
- VIA owns delegation, user interaction and task lifecycle.
- User context is local-first and purpose-scoped when shared externally.
- Provider-specific model/runtime details do not leak into VIA canonical contracts.
