# VIA System Definition and Evaluation Baseline

> Status: Architecture Rebaseline Working Baseline  
> Branch: `architecture-rebaseline-20260918`  
> Principle: This document defines the system first. Quality attributes, evaluation scenarios, and architecture decision points are derived from this definition rather than assumed in advance.

# 1. System Mission & Boundary

## 1.1 System Mission

VIA is **software installed and executed on the user's PC that manages voice, text, and on-screen interaction as one continuous user interaction flow; answers requests directly when VIA can complete them without an external work-performing agent; delegates requests that require actual work execution to an appropriate Downstream Agent; and delivers progress and results back to the user.**

"Installed and executed on the user's PC" means that the VIA software itself runs on the user's PC. It does **not** mean that every AI model or Downstream Agent used by VIA must run locally.

VIA has the following responsibilities.

1. **Receive Voice and Text input**
   - Accept user requests through both Voice and Text.
   - Treat both modalities as part of the same user interaction and conversation state.

2. **Provide real-time Voice interaction**
   - Voice processing is part of the VIA architecture scope.
   - VIA includes a Voice Runtime that uses an S2S (Speech-to-Speech) Model as the primary voice model.
   - VIA manages the start, end, interruption, and continuation of voice interaction.
   - Even when the S2S Model answers directly from its own knowledge, the user turn and response remain part of VIA-managed conversation state.

3. **Collect and retrieve Context**
   - Collect current PC interaction context such as screen information, pointer activity, selection, focused window, and foreground application.
   - When required to understand or answer a request, VIA may perform policy-allowed read-only retrieval from sources such as local files, mail, calendar, and browser context.

4. **Understand and complete the user's request representation**
   - Interpret natural, incomplete, and conversational user expressions using Voice/Text input together with available Context.
   - Resolve expressions such as "this", "here", "this part", or "what we were doing before" using interaction and conversation context.

5. **Associate the request with ongoing work**
   - Determine whether the current user request starts new work or continues, modifies, queries, or cancels existing work.
   - Connect the request to the appropriate VIA-managed work state.

6. **Provide a VIA Direct Response when external work execution is unnecessary**
   - VIA may answer directly when the request can be completed without a Downstream Agent performing an external action or domain workflow.
   - Examples include a simple question that the S2S Model can answer from its own knowledge and an informational request that can be answered using VIA's read-only Context access.
   - Direct responses remain in the same VIA-managed conversation history and can be referenced by later follow-up turns.

7. **Select and delegate to a Downstream Agent when actual work execution is required**
   - Select an appropriate Downstream Agent.
   - Provide the request and the Context required for execution.
   - The Downstream Agent performs the actual domain reasoning, planning, tool selection, and work execution.

8. **Manage user-facing work interaction**
   - Connect progress, clarification requests, consent requests, follow-up instructions, corrections, cancellation, completion, and failure between the user and the relevant work.

9. **Provide both Text and Voice responses**
   - Every user-visible response is recorded and shown as Text in the Chat UI.
   - When Voice interaction is active, VIA also provides a short Voice response containing the key information the user needs immediately.
   - Text may contain additional details that are unnecessary to read aloud.
   - For long-running work or requests requiring later user attention, VIA may additionally use UI or OS notifications.

In short:

> **VIA owns user Interaction and Agent Orchestration. A Downstream Agent owns actual work Reasoning, Planning, and Execution.**

---

## 1.2 System Boundary

The primary responsibility boundary is:

```text
User
 │
 │ Voice / Text / Screen Interaction
 ▼
VIA — software running on the user's PC
 │
 ├─ Voice Runtime
 │    └─ S2S Model
 │
 ├─ Voice / Text interaction handling
 ├─ Conversation state management
 ├─ Context collection and read-only retrieval
 ├─ Interaction grounding
 ├─ Request understanding and refinement
 ├─ New-work / existing-work association
 ├─ VIA Direct Response
 ├─ Downstream Agent selection and delegation
 ├─ Progress / clarification / consent interaction
 ├─ Follow-up / correction / cancel / result handling
 └─ Voice / Text response delivery
 │
 ▼
Downstream Agent
 │
 ├─ Domain reasoning
 ├─ Planning
 ├─ Tool selection
 ├─ Tool execution
 └─ Actual work completion
 │
 ▼
OS / Application / Web / External Service
```

### VIA may directly perform

- Voice and Text interaction handling
- Voice S2S processing and Direct Response
- Conversation-state management
- Screen and user-interaction Context collection
- Policy-allowed read-only search and retrieval of local file, mail, calendar, browser, and similar Context
- Grounding user expressions to screen or interaction Context
- Request understanding and refinement
- New-work / existing-work association
- Downstream Agent selection and delegation
- Progress, clarification, consent, follow-up, correction, cancellation, and result interaction
- Voice and Text response delivery

### Downstream Agent performs

- Domain-specific reasoning required to perform the requested work
- Work planning
- Tool selection
- Tool execution
- Actual state-changing work in the OS, application, web, or external service
- Agent-internal workflow and sub-task management

### Read-only Context versus Action

The boundary is intentionally simple:

> **VIA may Read, Search, and Understand information needed for interaction.  
> A Downstream Agent performs state-changing Actions needed to complete actual work.**

Examples:

| Operation | Owner |
| --- | --- |
| Read the current screen | VIA |
| Inspect pointer / selection / focused window | VIA |
| Search file names or read file metadata/content when policy allows | VIA |
| Search or read mail/calendar/browser Context when policy allows | VIA |
| Move or delete a file | Downstream Agent |
| Send an email | Downstream Agent |
| Create or modify a calendar event | Downstream Agent |
| Operate an application to perform user work | Downstream Agent |
| Execute a web transaction | Downstream Agent |

A VIA Direct Response is allowed only when no Downstream Agent work execution is required. Read-only information retrieval does not by itself require delegation.

---

## 1.3 AI Model Boundary

AI used around VIA is divided into three scopes so that responsibility is unambiguous.

### 1. Voice S2S Model — inside VIA architecture scope

- VIA Voice Runtime uses an S2S Model as the primary Voice AI model.
- The S2S Model handles real-time voice interaction and may directly answer simple requests from its own knowledge.
- All S2S user turns and responses remain under VIA conversation-state management.
- The Voice Runtime, its interfaces, state handling, and its integration with the rest of VIA are part of the architecture design scope.
- Additional ASR, VAD, TTS, or helper models may be introduced only as part of an explicit architecture design; the S2S Model remains the mandatory primary Voice Runtime model.

### 2. VIA semantic inference — inside VIA architecture scope

VIA must support semantic decisions required for its own responsibilities, including:

- request refinement
- interaction grounding
- new-work / existing-work association
- Downstream Agent selection
- deciding whether a request can be answered directly or requires delegation
- constructing the user-facing response

These decisions may use generative models, deterministic logic, or a combination of both. The architecture must explicitly define where such inference occurs and which VIA element owns each decision.

The model used for VIA semantic inference may be local/on-device or remote/cloud. Model size, provider, and deployment location are not treated as part of the Downstream Agent and remain relevant to VIA architecture.

### 3. Downstream Agent Model — outside VIA architecture scope

A model used internally by a Downstream Agent for domain reasoning, planning, tool selection, or tool execution belongs to the Downstream Agent implementation and is outside the VIA architecture design and evaluation scope.

---

## 1.4 Boundary Principles

The following principles are fixed for this project.

1. **VIA owns user Interaction and Agent Orchestration.**
2. **Downstream Agents own actual work Reasoning, Planning, and Execution.**
3. **VIA itself runs on the user's PC; VIA dependencies may run locally or remotely.**
4. **The Voice Runtime is part of VIA and uses an S2S Model as its primary Voice Model.**
5. **VIA may perform policy-allowed read-only Context access required to understand or directly answer a request.**
6. **State-changing work Actions are delegated to a Downstream Agent.**
7. **Requests that do not require Downstream Agent work execution may be answered directly by VIA.**
8. **Direct Response and Agent-delegated Response are both managed in the same VIA conversation state.**
9. **Every user-visible response is retained as Text in the Chat UI; active Voice interaction additionally receives a short spoken response containing the key information.**
10. **The architecture must explicitly assign ownership for semantic decisions inside VIA rather than assuming that every decision is performed by one fixed model or one fixed component.**
11. **Downstream Agent internal models, reasoning, planning, tool selection, tool execution, and execution performance are outside the VIA architecture evaluation boundary.**

This section deliberately does not define the final Architecture Significant Requirements. Those requirements are derived after the system capabilities, representative use cases, fixed assumptions, and intentional change scenarios have been defined.
