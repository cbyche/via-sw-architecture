//! The nine frontend tools, their instructions, and the handler that answers
//! them.
//!
//! [`catalog`] owns the model-visible names, descriptions and JSON Schemas;
//! [`instructions`] owns the per-response instructions attached to a tool
//! result; [`handler`] owns the dispatch and every catalogued output shape; and
//! [`transcripts`] owns the per-turn ASR text a delegation is pinned with.

pub mod catalog;
pub mod handler;
pub mod instructions;
pub mod transcripts;

pub use catalog::{
    ALL_TOOL_NAMES, ALWAYS_DECLARED, CANCEL_AGENT_TASK, ENTER_SLEEP, GET_AGENT_TASK_STATUS,
    GET_CURRENT_TIME, MAX_INPUT_REFS, MAX_NOTES_ITEMS, MEMORY, MEMORY_ACTIONS, NOTES,
    NOTES_ACTIONS, PERMISSION_DECISIONS, REMINDER_RECURRENCES, REMINDER_TYPES,
    RESPOND_AGENT_PERMISSION, SCHEDULE_REMINDER, SLEEPING_CLIENT_STATE, SPAWN_THINKING,
    frontend_tools, tools,
};
pub use handler::{
    ACTIVITY_DETAIL_CLIP, AssumeAvailable, BackendAvailability, CANCELLABLE_STATUSES,
    ClientContext, CommittedTurn, DelegationRequest, DelegationRunners, LIST_ALL_CAP,
    MAX_DEFERRED_RESPONSES, MAX_PROCESSED_CALLS, MAX_TURN_TASKS, OBJECTIVE_CLIP,
    PermissionResponder, QUERY_OBJECTIVE_CLIP, RESULT_CLIP, STATUS_QUERY_PRIORITY,
    StatusQueryRequest, THINKING_MARKER, ToolCall, ToolCallHandler, ToolCallHandlerConfig,
    ToolCallOutcome,
};
pub use instructions::{
    accepted_instructions, backend_disconnected_instructions, backend_not_configured_instructions,
    duplicate_submission_instructions, permission_pending_instructions,
    permission_response_instructions, permission_submitted_always_instructions,
    permission_submitted_reject_instructions, reminder_confirmation_instructions,
    result_response_instructions, speak_response_instructions,
};
pub use transcripts::{
    DEFAULT_MAX_TURNS, DEFAULT_WAIT_MS, ResolvedDelegation, TurnTranscripts, TurnTranscriptsBuilder,
};
