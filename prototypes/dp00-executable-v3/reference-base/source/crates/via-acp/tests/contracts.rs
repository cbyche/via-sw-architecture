//! The catalogued contracts this crate owns, asserted against the catalogue.
//!
//! Every value here is **parsed from `docs/reference/contracts.json`**, never
//! retyped. That is the point: a test that restated the expected string would
//! agree with itself forever, including after somebody edited the catalogue.
//! Reading the catalogue means a reworded or renumbered contract fails here
//! rather than quietly stopping being checked.
//!
//! `via-conformance` is the crate that eventually owns the whole 707-row
//! acceptance suite; these are the rows `via-acp` is answerable for, asserted
//! where the code that satisfies them lives.

use std::sync::OnceLock;

use serde_json::Value;

/// One catalogued contract.
struct Contract {
    kind: String,
    name: String,
    exact_value: String,
    file: String,
    /// The catalogue's own rationale. Several of the load-bearing clauses live
    /// here rather than in `exactValue`, and they are exactly the clauses that
    /// say *why* a value cannot be changed.
    why: String,
}

fn catalogue() -> &'static Vec<Contract> {
    static CATALOGUE: OnceLock<Vec<Contract>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("reference")
            .join("contracts.json");
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let parsed: Vec<Value> = serde_json::from_str(&raw).expect("contracts.json is JSON");
        parsed
            .into_iter()
            .map(|entry| Contract {
                kind: string(&entry, "kind"),
                name: string(&entry, "name"),
                exact_value: string(&entry, "exactValue"),
                file: string(&entry, "file"),
                why: string(&entry, "why"),
            })
            .collect()
    })
}

fn string(entry: &Value, key: &str) -> String {
    entry
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// The one contract with this `(kind, name)`.
fn contract(kind: &str, name: &str) -> &'static Contract {
    let matches: Vec<&Contract> = catalogue()
        .iter()
        .filter(|entry| entry.kind == kind && entry.name == name)
        .collect();
    assert!(
        !matches.is_empty(),
        "no catalogued contract {kind}/{name} — has it been renamed?"
    );
    matches[0]
}

/// Every contract with this `(kind, name)`; some are catalogued twice.
fn contracts(kind: &str, name: &str) -> Vec<&'static Contract> {
    let matches: Vec<&Contract> = catalogue()
        .iter()
        .filter(|entry| entry.kind == kind && entry.name == name)
        .collect();
    assert!(!matches.is_empty(), "no catalogued contract {kind}/{name}");
    matches
}

#[test]
fn the_catalogue_is_the_one_the_port_was_written_against() {
    assert_eq!(
        catalogue().len(),
        707,
        "docs/architecture.md §14: 707 contracts"
    );
}

#[test]
fn every_catalogued_json_rpc_method_is_the_one_the_client_sends() {
    // Ten `json-rpc-method` records. The catalogued `name` *is* the wire method
    // for nine of them; `initialize` is the tenth and has no prefix.
    let catalogued: Vec<&str> = catalogue()
        .iter()
        .filter(|entry| entry.kind == "json-rpc-method")
        .map(|entry| entry.name.as_str())
        .collect();
    assert_eq!(catalogued.len(), 10, "ten catalogued methods");

    for name in &catalogued {
        assert!(
            via_acp::methods::ACP_METHODS.contains(name)
                || *name == "session/list"
                || *name == "session/close",
            "{name} is catalogued but the client does not name it"
        );
    }

    // And the two catalogued as `ws-event` because they are notifications.
    for name in ["session/cancel", "session/update"] {
        let entry = contract("ws-event", name);
        assert!(
            via_acp::methods::ACP_METHODS.contains(&name),
            "{name} is catalogued but the client does not name it"
        );
        assert!(
            entry.exact_value.contains(name),
            "{name} is not spelled in its own contract"
        );
    }

    // Every name the client sends must be spelled inside the contract that
    // describes it, which is what catches a rename on either side.
    for (constant, kind, name) in [
        (
            via_acp::methods::INITIALIZE,
            "json-rpc-method",
            "initialize",
        ),
        (
            via_acp::methods::SESSION_NEW,
            "json-rpc-method",
            "session/new",
        ),
        (
            via_acp::methods::SESSION_RESUME,
            "json-rpc-method",
            "session/resume",
        ),
        (
            via_acp::methods::SESSION_LOAD,
            "json-rpc-method",
            "session/load",
        ),
        (
            via_acp::methods::SESSION_LIST,
            "json-rpc-method",
            "session/list",
        ),
        (
            via_acp::methods::SESSION_PROMPT,
            "json-rpc-method",
            "session/prompt",
        ),
        (
            via_acp::methods::SESSION_CLOSE,
            "json-rpc-method",
            "session/close",
        ),
        (
            via_acp::methods::SESSION_SET_CONFIG_OPTION,
            "json-rpc-method",
            "session/set_config_option",
        ),
        (
            via_acp::methods::SESSION_SET_MODEL,
            "json-rpc-method",
            "session/set_model",
        ),
        (
            via_acp::methods::SESSION_REQUEST_PERMISSION,
            "json-rpc-method",
            "session/request_permission",
        ),
    ] {
        let entry = contract(kind, name);
        assert_eq!(constant, name);
        assert!(
            entry.exact_value.contains(constant),
            "{constant} is not spelled in its own contract: {}",
            entry.exact_value
        );
    }
}

#[test]
fn session_set_model_is_catalogued_as_a_hard_coded_literal() {
    let entry = contract("json-rpc-method", "session/set_model");
    assert!(
        entry
            .exact_value
            .contains("hard-coded string literal (not an SDK constant)"),
        "{}",
        entry.exact_value
    );
    assert_eq!(via_acp::methods::SESSION_SET_MODEL, "session/set_model");
}

#[test]
fn the_client_declares_no_client_capability_because_the_contract_says_so() {
    let entry = contract("json-rpc-method", "initialize");
    assert!(
        entry.exact_value.contains("\"clientCapabilities\": {}"),
        "{}",
        entry.exact_value
    );
    assert!(
        entry.file.contains("acp-process-client.mjs"),
        "the contract still points at the file this module ports"
    );
    assert!(
        via_acp::declares_no_capability(&via_acp::client_capabilities()),
        "the Gateway promises an ACP agent nothing"
    );
}

#[test]
fn the_prompt_cancelled_stop_reason_is_catalogued_as_a_hard_error() {
    let entry = contract("json-rpc-method", "session/prompt");
    assert!(
        entry
            .exact_value
            .contains("If result.stopReason === \"cancelled\" the client throws"),
        "{}",
        entry.exact_value
    );
    assert!(
        entry.why.contains(
            "a Rust port that returns Ok on 'cancelled' would silently complete cancelled Work"
        ),
        "{}",
        entry.why
    );
}

#[test]
fn every_catalogued_timeout_is_the_constant_the_client_uses() {
    let entry = contract("default-value", "timeouts and limits");
    let value = &entry.exact_value;
    for (fragment, actual) in [
        ("AcpProcessClient.timeoutMs default 300_000", 300_000_u64),
        ("initialize 15_000", 15_000),
        ("session/list 15_000 per page", 15_000),
        ("session/set_config_option 15_000", 15_000),
        ("session/set_model 15_000", 15_000),
        ("session/close 5_000", 5_000),
    ] {
        assert!(value.contains(fragment), "missing `{fragment}` in {value}");
        let _ = actual;
    }
    assert_eq!(via_acp::limits::DEFAULT_TIMEOUT.as_millis(), 300_000);
    assert_eq!(via_acp::limits::INITIALIZE_TIMEOUT.as_millis(), 15_000);
    assert_eq!(via_acp::limits::SESSION_LIST_TIMEOUT.as_millis(), 15_000);
    assert_eq!(
        via_acp::limits::SET_CONFIG_OPTION_TIMEOUT.as_millis(),
        15_000
    );
    assert_eq!(via_acp::limits::SET_MODEL_TIMEOUT.as_millis(), 15_000);
    assert_eq!(via_acp::limits::CLOSE_SESSION_TIMEOUT.as_millis(), 5_000);

    assert!(value.contains("MAX_STDERR_CHARS 12_000"), "{value}");
    assert_eq!(via_acp::limits::MAX_STDERR_CHARS, 12_000);
    assert!(value.contains("PROCESS_TREE_GRACE_MS 750"), "{value}");
    assert_eq!(via_acp::limits::PROCESS_TREE_GRACE.as_millis(), 750);
    assert!(value.contains("PROCESS_TREE_POLL_MS 25"), "{value}");
    assert_eq!(via_acp::limits::PROCESS_TREE_POLL.as_millis(), 25);
    assert!(value.contains("session/list hard page cap 100"), "{value}");
    assert_eq!(via_acp::limits::SESSION_LIST_PAGE_CAP, 100);
    assert!(
        value.contains(
            "session/prompt: timeoutMs 0 on the transport (client-side pausable timer instead)"
        ),
        "session/prompt sends no transport deadline: {value}"
    );
}

#[test]
fn the_backend_session_state_file_is_the_one_on_disk() {
    // Catalogued as Partial and owed to this crate: `via-store` asserts the
    // atomic-write and quarantine halves, and this is the ACP half.
    let entry = contract("file-path", "backend session state file");
    let value = &entry.exact_value;
    assert!(
        value.contains("state/acp-sessions.json"),
        "the file name is KEEP per docs/rebrand.md: {value}"
    );
    assert_eq!(
        via_core::paths::BACKEND_SESSION_STATE_PATH,
        "state/acp-sessions.json"
    );
    assert!(value.contains("\"version\":1"), "{value}");
    assert_eq!(via_acp::registry::VERSION, 1);
    assert!(value.contains("\"coordinators\""), "{value}");
    assert!(value.contains("\"projects\""), "{value}");
    assert!(
        value.contains("QWEN_AUDIO_AGENT_BACKEND_SESSION_STATE_PATH"),
        "the override variable, renamed to VIA_* per docs/rebrand.md: {value}"
    );
    assert_eq!(
        via_core::config::names::BACKEND_SESSION_STATE_PATH,
        "VIA_BACKEND_SESSION_STATE_PATH"
    );
}

#[test]
fn the_registry_writes_the_catalogued_document() {
    let entry = contract("json-field", "ACP session registry file");
    let value = &entry.exact_value;
    assert!(value.contains("version: 1"), "{value}");
    for field in ["sessionId", "cwd", "updatedAt", "title"] {
        assert!(value.contains(field), "missing `{field}` in {value}");
    }

    // Write one of each and read the bytes back.
    let directory = tempfile::TempDir::new().expect("tempdir");
    let path = directory.path().join("state").join("acp-sessions.json");
    let registry = via_acp::AcpSessionRegistry::builder()
        .file_path(path.clone())
        .now(std::sync::Arc::new(|| 1_755_859_000_000))
        .build();
    let coordinator = via_downstream::SessionKey::coordinator("example", "owner one");
    let project = via_downstream::SessionKey::project("example", "previous-session");
    registry.set(&coordinator, "coordinator-session", "/coordinator");
    registry.set_project(&project, "previous-session", "/project", "Project");

    let raw = std::fs::read_to_string(&path).expect("the index was written");
    let document: Value = serde_json::from_str(&raw).expect("valid JSON");
    assert_eq!(document["version"], 1);
    assert_eq!(
        document
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["version", "coordinators", "projects"],
        "`version` first, then the two maps"
    );

    let stored = &document["coordinators"]["example:owner%20one:backend"];
    assert_eq!(stored["sessionId"], "coordinator-session");
    assert_eq!(stored["cwd"], "/coordinator");
    assert_eq!(stored["updatedAt"], 1_755_859_000_000_i64);
    assert_eq!(
        stored
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["sessionId", "cwd", "updatedAt"]
    );

    let stored = &document["projects"]["example:previous-session"];
    assert_eq!(stored["sessionId"], "previous-session");
    assert_eq!(stored["title"], "Project");
    assert_eq!(
        stored
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["sessionId", "cwd", "title", "updatedAt"]
    );

    assert!(raw.ends_with("}\n"), "trailing newline");
    assert!(raw.contains("\n  \"version\""), "two-space indent");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            std::fs::metadata(&path)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600,
            "mode 0600: the file holds session identity"
        );
    }
}

#[test]
fn a_node_written_index_is_readable_and_a_corrupt_one_is_quarantined() {
    // The whole reason the format is a contract: a VIA install pointed at an
    // existing Node install's configuration directory keeps its sessions.
    let directory = tempfile::TempDir::new().expect("tempdir");
    let path = directory.path().join("acp-sessions.json");
    std::fs::write(
        &path,
        // A pre-`projects` document, exactly as an older install wrote it.
        "{\n  \"version\": 1,\n  \"coordinators\": {\n    \"example:owner:backend\": \
         {\"sessionId\": \"coordinator\", \"cwd\": \"/coordinator\", \"updatedAt\": 1}\n  }\n}\n",
    )
    .expect("write");
    let registry = via_acp::AcpSessionRegistry::builder()
        .file_path(path.clone())
        .build();
    let key = via_downstream::SessionKey::coordinator("example", "owner");
    assert_eq!(
        registry.get(&key).map(|record| record.session_id),
        Some("coordinator".to_owned()),
        "a file written before `projects` existed still loads"
    );
    assert!(registry.health().ok);

    let corrupt = directory.path().join("corrupt.json");
    std::fs::write(&corrupt, "{not-json").expect("write");
    let warnings = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let registry = via_acp::AcpSessionRegistry::builder()
        .file_path(corrupt.clone())
        .on_warning({
            let warnings = std::sync::Arc::clone(&warnings);
            std::sync::Arc::new(move |warning: &via_store::StoreWarning| {
                warnings
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(warning.clone());
            })
        })
        .build();
    assert_eq!(registry.get(&key), None);
    let warnings = warnings
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(warnings.len(), 1, "one warning, once");
    let quarantine = warnings[0]
        .quarantine_path
        .as_ref()
        .expect("the original was moved aside");
    assert!(std::path::Path::new(quarantine).exists());
    assert!(!registry.health().ok);

    // And the store still accepts new state afterwards.
    registry.set(&key, "new-session", "/work");
    let document: Value =
        serde_json::from_str(&std::fs::read_to_string(&corrupt).expect("rewritten"))
            .expect("valid JSON");
    assert_eq!(
        document["coordinators"]["example:owner:backend"]["sessionId"],
        "new-session"
    );
}

#[test]
fn the_attachment_uri_scheme_is_the_renamed_one() {
    for entry in contracts("file-path", "attachment resource URI scheme")
        .into_iter()
        .chain(contracts("prompt-text", "ACP attachment URI scheme"))
    {
        assert!(
            entry.exact_value.contains("qwen-audio-agent://input/"),
            "the catalogue still records the upstream scheme: {}",
            entry.exact_value
        );
    }
    // docs/rebrand.md RENAME: `qwen-audio-agent://input/` → `via://input/`.
    assert_eq!(via_acp::ATTACHMENT_URI_PREFIX, "via://input/");

    let blocks = via_acp::input_parts_to_acp_blocks(&[via_downstream::PromptAttachment {
        url: "data:image/png;base64,aGVsbG8=".to_owned(),
        filename: "reference.png".to_owned(),
        mime: "image/png".to_owned(),
    }]);
    let serialized = serde_json::to_value(&blocks[0]).expect("serializes");
    assert_eq!(serialized["uri"], "via://input/reference.png");
    assert_eq!(serialized["type"], "image");
    assert_eq!(serialized["mimeType"], "image/png");
    assert_eq!(serialized["data"], "aGVsbG8=");
}

#[test]
fn the_concurrent_prompt_refusal_carries_the_catalogued_status() {
    let entry = contract("error-code", "concurrent prompt on one session");
    assert!(
        entry.exact_value.contains("status: 409"),
        "{}",
        entry.exact_value
    );
    assert!(
        entry.exact_value.contains("protocol: 'acp'"),
        "{}",
        entry.exact_value
    );
    assert_eq!(via_acp::STATUS_CONFLICT, 409);
    let error = via_acp::AcpError::SessionBusy {
        label: "Example".to_owned(),
        id: "s-1".to_owned(),
    };
    assert_eq!(error.status(), 409);
    assert_eq!(error.protocol(), "acp");
    assert_eq!(
        error.message(via_i18n::Locale::Zh),
        "Example Session s-1 已有正在执行的请求"
    );
}

#[test]
fn the_request_failure_wrapper_uses_the_catalogued_punctuation() {
    let entry = contract("error-code", "ACP request failure wrapper");
    assert!(
        entry
            .exact_value
            .contains("stderr is omitted when error.name === 'RequestError'"),
        "{}",
        entry.exact_value
    );
    assert!(
        entry.file.contains("acp-process-client.mjs"),
        "{}",
        entry.file
    );

    let error = via_acp::AcpError::RequestFailed {
        label: "Example".to_owned(),
        method: "session/prompt".to_owned(),
        detail: "Internal error".to_owned(),
        stderr: String::new(),
        body: "missing scope".to_owned(),
    };
    let message = error.message(via_i18n::Locale::Zh);
    assert_eq!(message, "Example ACP session/prompt 失败：Internal error");
    assert!(
        message.contains('\u{ff1a}'),
        "the catalogue calls out the FULLWIDTH COLON U+FF1A"
    );
    assert_eq!(error.body(), "missing scope");

    let with_stderr = via_acp::AcpError::RequestFailed {
        label: "Example".to_owned(),
        method: "session/prompt".to_owned(),
        detail: "Internal error".to_owned(),
        stderr: "native detail".to_owned(),
        body: "native detail".to_owned(),
    };
    assert_eq!(
        with_stderr.message(via_i18n::Locale::Zh),
        "Example ACP session/prompt 失败：Internal error：native detail"
    );
}

#[test]
fn process_lifecycle_messages_are_the_catalogued_four() {
    let entry = contract("error-code", "process lifecycle errors");
    let value = &entry.exact_value;
    for fragment in [
        "进程启动失败",
        "进程意外退出",
        "初始化失败",
        "协议版本不兼容",
    ] {
        assert!(value.contains(fragment), "missing `{fragment}` in {value}");
    }
    assert!(
        value.contains("so ENOENT is preserved"),
        "the errno survives to the UI: {value}"
    );

    let zh = via_i18n::Locale::Zh;
    assert_eq!(
        via_acp::AcpError::ProcessSpawnFailed {
            label: "Example".to_owned(),
            detail: "no such file".to_owned(),
            errno: via_acp::SpawnErrno::NoEnt,
        }
        .message(zh),
        "Example ACP 进程启动失败（no such file）"
    );
    assert_eq!(
        via_acp::AcpError::ProcessExited {
            label: "Example".to_owned(),
            code: "SIGKILL".to_owned(),
            stderr: String::new(),
        }
        .message(zh),
        "Example ACP 进程意外退出（SIGKILL）"
    );
    assert_eq!(
        via_acp::AcpError::ProcessExited {
            label: "Example".to_owned(),
            code: "1".to_owned(),
            stderr: "detail".to_owned(),
        }
        .message(zh),
        "Example ACP 进程意外退出（1）：detail"
    );
    assert_eq!(
        via_acp::AcpError::InitializeFailed {
            label: "Example".to_owned(),
            stderr: "detail".to_owned(),
        }
        .message(zh),
        "Example ACP 初始化失败：detail"
    );
    assert_eq!(
        via_acp::AcpError::ProtocolVersionMismatch {
            label: "Example".to_owned(),
            agent: "2".to_owned(),
            client: "1".to_owned(),
        }
        .message(zh),
        "Example ACP 协议版本不兼容（Agent=2，Client=1）"
    );
    assert_eq!(
        via_acp::AcpError::ResumeUnsupported {
            label: "Example".to_owned(),
        }
        .message(zh),
        "Example ACP 不支持继续已有 Session"
    );
    assert_eq!(
        via_acp::AcpError::Cancelled { reason: None }.message(zh),
        "ACP Session 已取消"
    );
}

#[test]
fn the_permission_reply_mapping_is_the_catalogued_order() {
    let entry = contract("json-field", "ACP requestPermission reply mapping");
    let value = &entry.exact_value;
    assert!(value.contains("allow_once"), "{value}");
    assert!(value.contains("allow_always"), "{value}");
    assert!(value.contains("reject_always"), "{value}");
    assert!(value.contains("reject_once"), "{value}");

    let request = contract("json-rpc-method", "session/request_permission");
    assert!(
        request
            .exact_value
            .contains("approve => first of kind ['allow_once','allow_always'] in that order"),
        "{}",
        request.exact_value
    );
    assert!(
        request
            .exact_value
            .contains("reject => first of ['reject_always','reject_once'] in that order"),
        "{}",
        request.exact_value
    );

    use agent_client_protocol::schema::v1::PermissionOptionKind as Kind;
    assert_eq!(
        via_acp::permission::APPROVE_KIND_ORDER,
        [Kind::AllowOnce, Kind::AllowAlways]
    );
    assert_eq!(
        via_acp::permission::REJECT_KIND_ORDER,
        [Kind::RejectAlways, Kind::RejectOnce]
    );
}

#[test]
fn the_payload_unwrapping_depth_is_the_catalogued_three() {
    let entry = contract("state-name", "parseCoordinatorPayload unwrapping algorithm");
    assert!(
        entry.exact_value.contains("Loop at most 3 times"),
        "{}",
        entry.exact_value
    );
    assert!(
        entry.exact_value.contains("After 3 iterations return null"),
        "{}",
        entry.exact_value
    );
    assert_eq!(via_acp::session::PAYLOAD_UNWRAP_DEPTH, 3);
}

#[test]
fn the_native_tool_output_traversal_is_the_catalogued_order() {
    let entry = contract("state-name", "nativeToolOutput extraction order");
    let value = &entry.exact_value;
    assert!(
        value.contains("childSessionKey|sessionKey|sessionId|session_id"),
        "{value}"
    );
    assert!(value.contains("recurse into value.details"), "{value}");
    assert!(
        value.contains("block?.text || block?.content || block"),
        "{value}"
    );

    // The order decides which key wins when several are present.
    let found = via_acp::native_tool_output(&serde_json::json!({
        "details": { "sessionId": "from-details" },
        "content": [{ "text": "{\"sessionId\":\"from-content\"}" }],
    }));
    assert_eq!(found["sessionId"], "from-details");
}

#[test]
fn the_legacy_inline_title_reaches_the_screen_in_the_users_locale() {
    let entry = contract(
        "json-field",
        "normalizeCoordinatorContent legacy inline upgrade",
    );
    let value = &entry.exact_value;
    assert!(value.contains("Agent 结果"), "{value}");
    assert!(value.contains("\"format\":\"markdown\""), "{value}");
    assert!(
        value.contains("<original, untrimmed string>"),
        "the stored content is untrimmed: {value}"
    );

    let normalized = via_acp::normalize_coordinator_content(
        r#"{"presentation":{"inline":" body "}}"#,
        via_i18n::Locale::Zh,
    );
    let parsed: Value = serde_json::from_str(&normalized).expect("compact JSON");
    assert_eq!(parsed["presentation"]["inline"]["title"], "Agent 结果");
    assert_eq!(parsed["presentation"]["inline"]["format"], "markdown");
    assert_eq!(parsed["presentation"]["inline"]["content"], " body ");
}

#[test]
fn the_session_key_formats_are_the_catalogued_ones() {
    // Owned by `via-downstream`; cross-checked here because the registry is
    // keyed by them and a disagreement between the two crates orphans stored
    // sessions. The catalogued examples use real backend ids, so the shape is
    // asserted against a neutral protocol instead: `via-arch-test` requires
    // that a test needing a real backend fixture live in `via-backends`.
    let coordinator = contract("state-name", "coordinator session key format");
    assert!(
        coordinator
            .exact_value
            .contains("`${protocol}:${encodeURIComponent(clean(ownerId) || 'personal')}:backend`"),
        "{}",
        coordinator.exact_value
    );
    assert_eq!(
        via_downstream::SessionKey::coordinator("example", "owner one").as_str(),
        "example:owner%20one:backend",
        "trimmed, percent-encoded, not lower-cased"
    );
    assert_eq!(
        via_downstream::SessionKey::coordinator("example", "  ").as_str(),
        "example:personal:backend",
        "an empty owner becomes the literal `personal`, never an empty segment"
    );

    let project = contract("state-name", "project session key format");
    assert!(
        project
            .exact_value
            .contains("`${protocol}:${clean(sessionId)}`"),
        "{}",
        project.exact_value
    );
    assert_eq!(
        via_downstream::SessionKey::project("example", " previous-session ").as_str(),
        "example:previous-session",
        "the backend's own id is trimmed but never re-encoded"
    );
}

#[test]
fn the_child_environment_allowlist_is_the_catalogued_one() {
    let entry = contract("env-var", "backend child environment allowlist");
    let value = &entry.exact_value;

    // Every OS name the catalogue lists is on the allow-list, and nothing else
    // is: the boundary is only a boundary if it is exact in both directions.
    let catalogued: Vec<&str> = value
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(names, _)| names.split(',').map(str::trim).collect())
        .expect("the contract lists the names in parentheses");
    assert_eq!(
        catalogued.len(),
        via_acp::SYSTEM_NAMES.len(),
        "catalogued {catalogued:?} vs {:?}",
        via_acp::SYSTEM_NAMES
    );
    for name in &catalogued {
        assert!(
            via_acp::SYSTEM_NAMES.contains(name),
            "{name} is catalogued but not forwarded"
        );
    }
    for name in via_acp::SYSTEM_NAMES {
        assert!(
            catalogued.contains(name),
            "{name} is forwarded but not catalogued — the boundary widened"
        );
    }

    for prefix in ["LC_", "npm_config_", "NPM_CONFIG_"] {
        assert!(
            value.contains(prefix),
            "missing prefix `{prefix}` in {value}"
        );
        assert!(via_acp::SYSTEM_PREFIXES.contains(&prefix));
    }
    assert_eq!(via_acp::SYSTEM_PREFIXES.len(), 3);

    // The internal names, renamed per docs/rebrand.md.
    let internals = contract("env-var", "backend env allowlist internals");
    for (upstream, ours) in [
        ("QWEN_AUDIO_AGENT_BACKEND_AGENT", "VIA_BACKEND_AGENT"),
        ("QWEN_AUDIO_AGENT_BACKEND_MODEL", "VIA_BACKEND_MODEL"),
        (
            "QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP",
            "VIA_BACKEND_OWNERSHIP",
        ),
        (
            "QWEN_AUDIO_AGENT_BACKEND_PERMISSION_MODE",
            "VIA_BACKEND_PERMISSION_MODE",
        ),
        ("QWEN_AUDIO_AGENT_DESKTOP", "VIA_DESKTOP"),
        (
            "QWEN_AUDIO_AGENT_DESKTOP_INSTALLED_ONLY",
            "VIA_DESKTOP_INSTALLED_ONLY",
        ),
        ("QWEN_AUDIO_AGENT_ENV_LOADED", "VIA_ENV_LOADED"),
        ("QWEN_AUDIO_AGENT_NODE", "VIA_NODE"),
        ("QWEN_AUDIO_AGENT_ROOT", "VIA_ROOT"),
        ("QWEN_AUDIO_AGENT_RUNTIME_ROOT", "VIA_RUNTIME_ROOT"),
        ("QWEN_AUDIO_AGENT_SOURCE_ROOT", "VIA_SOURCE_ROOT"),
    ] {
        assert!(
            internals.exact_value.contains(upstream),
            "{upstream} is not in the catalogued internals list"
        );
        assert!(
            via_acp::INTERNAL_NAMES.contains(&ours),
            "{ours} is not forwarded"
        );
    }
    assert_eq!(
        via_acp::INTERNAL_NAMES.len(),
        11,
        "eleven, matching shared/backend-environment.mjs:49-61"
    );

    // The stamp.
    let stamp = contract(
        "env-var",
        "QWEN_AUDIO_AGENT_ENV_LOADED / QWEN_AUDIO_AGENT_NODE",
    );
    assert!(
        stamp
            .exact_value
            .contains("QWEN_AUDIO_AGENT_ENV_LOADED='1'"),
        "{}",
        stamp.exact_value
    );
    let projected = via_acp::BackendEnv::project(
        &via_catalog::EnvironmentPolicy {
            names: &[],
            prefixes: &[],
            explicit_list_environment: None,
        },
        &via_core::EnvMap::new(),
        &[],
    );
    assert_eq!(projected.get("VIA_ENV_LOADED"), Some("1"));
    assert_eq!(projected.len(), 1, "the stamp, and nothing else");
}

#[test]
fn the_generic_acp_forward_list_is_the_catalogued_variable() {
    // `acp` is the generic entry point — the protocol, not a product — so it is
    // the one catalogued id this crate may read.
    let definition = via_catalog::backend_definition("acp").expect("the generic entry exists");
    assert_eq!(
        definition.environment.explicit_list_environment,
        Some("VIA_ACP_FORWARD_ENV"),
        "renamed from QWEN_AUDIO_AGENT_ACP_FORWARD_ENV per docs/rebrand.md"
    );
    let entry = contract("env-var", "Generic ACP block");
    assert!(
        entry
            .why
            .contains("QWEN_AUDIO_AGENT_ACP_FORWARD_ENV=NAME_A,NAME_B"),
        "{}",
        entry.why
    );

    let mut env = via_core::EnvMap::new();
    env.set("CUSTOM_AGENT_TOKEN", "custom-secret");
    env.set("OTHER_TOKEN", "other-secret");
    env.set("VIA_ACP_FORWARD_ENV", "CUSTOM_AGENT_TOKEN");
    let projected = via_acp::BackendEnv::project(&definition.environment, &env, &[]);
    assert_eq!(projected.get("CUSTOM_AGENT_TOKEN"), Some("custom-secret"));
    assert!(!projected.contains("OTHER_TOKEN"));
    assert!(
        !projected.contains("VIA_ACP_FORWARD_ENV"),
        "the opt-in list does not forward itself"
    );
}

#[test]
fn the_unsupported_connection_refusal_is_the_catalogued_sentence() {
    let entry = contract("error-code", "adapter session-config errors");
    assert!(
        entry.file.contains("acp-client-factory.mjs"),
        "the factory's refusal is catalogued here: {}",
        entry.file
    );
    assert!(
        entry.exact_value.contains("不支持的 ACP 连接方式"),
        "{}",
        entry.exact_value
    );
    let error = via_acp::create_acp_client_by_kind(
        "websocket",
        via_acp::SpawnSpec::new("x"),
        via_acp::AcpProcessClient::builder().label("Example"),
    )
    .expect_err("only the process kind exists");
    assert_eq!(
        error.message(via_i18n::Locale::Zh),
        "不支持的 ACP 连接方式：websocket"
    );
}
