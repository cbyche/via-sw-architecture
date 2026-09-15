//! The JSON-RPC method names this client speaks.
//!
//! **External contract** — ten `json-rpc-method` records in
//! `docs/reference/contracts.json`, sourced from
//! `server/src/agent/acp-process-client.mjs` and
//! `server/src/agent/permission-broker.mjs`. These strings are on the wire
//! between the Gateway and a third-party agent process; renaming one is not a
//! refactor, it is a protocol break. `docs/rebrand.md` classifies every ACP
//! wire method as **KEEP**.
//!
//! # Why restate what the SDK already has
//!
//! `agent-client-protocol` exports the same names as
//! `schema::v1::AGENT_METHOD_NAMES` / `schema::v1::CLIENT_METHOD_NAMES`, and
//! this module's own test asserts they agree, name by name. The point of
//! restating them is that the contract is on *VIA*, not on the SDK: if a future
//! SDK release renamed a method, that test fails loudly here rather than
//! silently changing what a backend receives.
//!
//! [`SESSION_SET_MODEL`] is the one name the SDK does **not** have, which is
//! exactly why upstream spells it as a literal too.

/// `initialize` — the first bytes a backend agent process ever sees.
pub const INITIALIZE: &str = "initialize";

/// `session/new` — creates the coordinator session and every project session.
pub const SESSION_NEW: &str = "session/new";

/// `session/resume` — continues an existing session, preferred over
/// [`SESSION_LOAD`] whenever `agentCapabilities.sessionCapabilities.resume` is
/// declared. The fixed-coordinator-identity model depends on it.
pub const SESSION_RESUME: &str = "session/resume";

/// `session/load` — the legacy continuation path, used only when `resume` is
/// absent and `agentCapabilities.loadSession` is declared.
pub const SESSION_LOAD: &str = "session/load";

/// `session/list` — paginated session enumeration. The page limit travels
/// inside `_meta`, not at the top level.
pub const SESSION_LIST: &str = "session/list";

/// `session/prompt` — the only method that carries user content.
pub const SESSION_PROMPT: &str = "session/prompt";

/// `session/cancel` — a **notification**. No id, no response, errors swallowed:
/// cancellation must never block.
pub const SESSION_CANCEL: &str = "session/cancel";

/// `session/close` — releases a session, when the agent declares it.
pub const SESSION_CLOSE: &str = "session/close";

/// `session/set_config_option` — coordinator mode, profile session options and
/// the model override all go through this one method.
pub const SESSION_SET_CONFIG_OPTION: &str = "session/set_config_option";

/// `session/set_model` — the legacy model path, for agents that returned
/// `models.availableModels` instead of a `category: "model"` config option.
///
/// **A hard-coded literal, not an SDK constant** — `acp-process-client.mjs:533`
/// spells it out in source, so it is version-pinned regardless of what the SDK
/// does. `agent-client-protocol` 2.0.0 has no such method.
pub const SESSION_SET_MODEL: &str = "session/set_model";

/// `session/update` — the **inbound** notification carrying the whole
/// progress/result stream.
pub const SESSION_UPDATE: &str = "session/update";

/// `session/request_permission` — the **inbound** request the Gateway answers.
pub const SESSION_REQUEST_PERMISSION: &str = "session/request_permission";

/// `fs/read_text_file` — a client obligation VIA does **not** accept.
///
/// Named here only so the refusal is explicit and testable: the Gateway
/// declares an empty `clientCapabilities`, so an agent that calls this gets a
/// method-not-found rather than a filesystem read. See
/// [`crate::client::CLIENT_CAPABILITIES_ARE_EMPTY`].
pub const FS_READ_TEXT_FILE: &str = "fs/read_text_file";

/// `fs/write_text_file` — the other unaccepted client obligation.
pub const FS_WRITE_TEXT_FILE: &str = "fs/write_text_file";

/// Every method this client sends or answers, in catalogue order.
///
/// The `fs/*` pair is deliberately absent: VIA names them
/// ([`FS_READ_TEXT_FILE`], [`FS_WRITE_TEXT_FILE`]) in order to refuse them.
pub const ACP_METHODS: &[&str] = &[
    INITIALIZE,
    SESSION_NEW,
    SESSION_RESUME,
    SESSION_LOAD,
    SESSION_LIST,
    SESSION_PROMPT,
    SESSION_CANCEL,
    SESSION_CLOSE,
    SESSION_SET_CONFIG_OPTION,
    SESSION_SET_MODEL,
    SESSION_UPDATE,
    SESSION_REQUEST_PERMISSION,
];

#[cfg(test)]
mod tests {
    use agent_client_protocol::schema::v1::{AGENT_METHOD_NAMES, CLIENT_METHOD_NAMES};

    use super::*;

    #[test]
    fn every_name_matches_the_sdk() {
        assert_eq!(INITIALIZE, AGENT_METHOD_NAMES.initialize);
        assert_eq!(SESSION_NEW, AGENT_METHOD_NAMES.session_new);
        assert_eq!(SESSION_RESUME, AGENT_METHOD_NAMES.session_resume);
        assert_eq!(SESSION_LOAD, AGENT_METHOD_NAMES.session_load);
        assert_eq!(SESSION_LIST, AGENT_METHOD_NAMES.session_list);
        assert_eq!(SESSION_PROMPT, AGENT_METHOD_NAMES.session_prompt);
        assert_eq!(SESSION_CANCEL, AGENT_METHOD_NAMES.session_cancel);
        assert_eq!(SESSION_CLOSE, AGENT_METHOD_NAMES.session_close);
        assert_eq!(
            SESSION_SET_CONFIG_OPTION,
            AGENT_METHOD_NAMES.session_set_config_option
        );
        assert_eq!(SESSION_UPDATE, CLIENT_METHOD_NAMES.session_update);
        assert_eq!(
            SESSION_REQUEST_PERMISSION,
            CLIENT_METHOD_NAMES.session_request_permission
        );
        assert_eq!(FS_READ_TEXT_FILE, CLIENT_METHOD_NAMES.fs_read_text_file);
        assert_eq!(FS_WRITE_TEXT_FILE, CLIENT_METHOD_NAMES.fs_write_text_file);
    }

    #[test]
    fn set_model_is_ours_because_the_sdk_has_no_such_method() {
        // Upstream hard-codes the literal for exactly this reason. If a future
        // SDK grows the method, this assertion is where the decision to switch
        // gets made deliberately.
        let sdk_names = [
            AGENT_METHOD_NAMES.session_new,
            AGENT_METHOD_NAMES.session_load,
            AGENT_METHOD_NAMES.session_resume,
            AGENT_METHOD_NAMES.session_list,
            AGENT_METHOD_NAMES.session_prompt,
            AGENT_METHOD_NAMES.session_cancel,
            AGENT_METHOD_NAMES.session_close,
            AGENT_METHOD_NAMES.session_set_config_option,
            AGENT_METHOD_NAMES.session_set_mode,
            AGENT_METHOD_NAMES.session_delete,
        ];
        assert!(!sdk_names.contains(&SESSION_SET_MODEL));
    }

    #[test]
    fn the_method_list_has_no_duplicates() {
        let mut sorted = ACP_METHODS.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), before);
    }
}
