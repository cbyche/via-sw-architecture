//! The coded errors.
//!
//! Three codes are upstream contracts with the `QWAUDIO_` prefix renamed to
//! `VIA_` per `docs/rebrand.md`; the rename must be applied to all three
//! atomically, which is what this file pins.

use pretty_assertions::assert_eq;
use via_protocol::{
    CODE_GATEWAY_ALREADY_RUNNING, CODE_GATEWAY_SETUP_REQUIRED, CODE_ILLEGAL_TRANSITION,
    CODE_INPUT_OWNER_REQUIRED, CODE_UNKNOWN_WIRE_VALUE, CONTRACT_ERROR_CODES, MissingSetting,
    ProtocolError, WorkStatus,
};

/// `shared/gateway-setup.mjs:49`, `shared/gateway-instance-lock.mjs:157`,
/// `server/src/voice/input-arbitration.mjs:73` — upstream code, then VIA code.
const RENAMED_CODES: &[(&str, &str)] = &[
    (
        "QWAUDIO_GATEWAY_SETUP_REQUIRED",
        CODE_GATEWAY_SETUP_REQUIRED,
    ),
    (
        "QWAUDIO_GATEWAY_ALREADY_RUNNING",
        CODE_GATEWAY_ALREADY_RUNNING,
    ),
    ("QWAUDIO_INPUT_OWNER_REQUIRED", CODE_INPUT_OWNER_REQUIRED),
];

#[test]
fn the_upstream_codes_are_renamed_qwaudio_to_via() {
    for (upstream, via) in RENAMED_CODES {
        assert_eq!(
            *via,
            upstream.replace("QWAUDIO_", "VIA_"),
            "{upstream} was not renamed as docs/rebrand.md specifies",
        );
    }
    assert_eq!(CODE_GATEWAY_SETUP_REQUIRED, "VIA_GATEWAY_SETUP_REQUIRED");
    assert_eq!(CODE_GATEWAY_ALREADY_RUNNING, "VIA_GATEWAY_ALREADY_RUNNING");
    assert_eq!(CODE_INPUT_OWNER_REQUIRED, "VIA_INPUT_OWNER_REQUIRED");
}

#[test]
fn no_code_keeps_the_upstream_prefix() {
    let every = [
        CODE_GATEWAY_SETUP_REQUIRED,
        CODE_GATEWAY_ALREADY_RUNNING,
        CODE_INPUT_OWNER_REQUIRED,
        CODE_UNKNOWN_WIRE_VALUE,
        CODE_ILLEGAL_TRANSITION,
    ];
    for code in every {
        assert!(code.starts_with("VIA_"), "{code} is not VIA-namespaced");
        assert!(!code.contains("QWAUDIO"), "{code} kept the upstream prefix");
        assert!(!code.contains("QWEN"), "{code} kept the upstream brand");
        assert!(
            code.bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'),
            "{code} is not SCREAMING_SNAKE_CASE",
        );
    }
}

/// The two VIA-owned codes say so in their names, so a reader never mistakes
/// one for an upstream contract.
#[test]
fn via_owned_codes_are_namespaced_apart_from_the_contract_ones() {
    assert_eq!(CODE_UNKNOWN_WIRE_VALUE, "VIA_PROTOCOL_UNKNOWN_WIRE_VALUE");
    assert_eq!(CODE_ILLEGAL_TRANSITION, "VIA_PROTOCOL_ILLEGAL_TRANSITION");
    assert_eq!(CONTRACT_ERROR_CODES.len(), 3);
    assert!(!CONTRACT_ERROR_CODES.contains(&CODE_UNKNOWN_WIRE_VALUE));
    assert!(!CONTRACT_ERROR_CODES.contains(&CODE_ILLEGAL_TRANSITION));
}

#[test]
fn every_variant_carries_its_code() {
    let setup = ProtocolError::SetupRequired {
        missing: vec![MissingSetting {
            field: "dashscopeApiKey".to_owned(),
            key: "DASHSCOPE_API_KEY".to_owned(),
            message: "set it in config.env".to_owned(),
        }],
    };
    assert_eq!(setup.code(), "VIA_GATEWAY_SETUP_REQUIRED");
    assert!(setup.is_upstream_contract());

    let running = ProtocolError::AlreadyRunning {
        origin: Some("http://127.0.0.1:8787".to_owned()),
    };
    assert_eq!(running.code(), "VIA_GATEWAY_ALREADY_RUNNING");
    assert!(running.is_upstream_contract());

    let owner = ProtocolError::InputOwnerRequired;
    assert_eq!(owner.code(), "VIA_INPUT_OWNER_REQUIRED");
    assert!(owner.is_upstream_contract());

    let unknown = ProtocolError::UnknownWireValue {
        vocabulary: "GatewayClientEvent",
        value: "nope".to_owned(),
    };
    assert_eq!(unknown.code(), "VIA_PROTOCOL_UNKNOWN_WIRE_VALUE");
    assert!(!unknown.is_upstream_contract());

    let illegal = ProtocolError::IllegalTransition {
        from: WorkStatus::Cancelling,
        to: WorkStatus::Completed,
    };
    assert_eq!(illegal.code(), "VIA_PROTOCOL_ILLEGAL_TRANSITION");
    assert!(!illegal.is_upstream_contract());
}

/// The `missing` list keeps upstream's `{field, key, message}` shape and its
/// field order — insertion order is observable, so the order is contract.
#[test]
fn missing_settings_keep_the_upstream_field_order() {
    let missing = MissingSetting {
        field: "speechToSpeechRealtimeUrl".to_owned(),
        key: "SPEECH_TO_SPEECH_REALTIME_URL".to_owned(),
        message: "缺少服务地址".to_owned(),
    };
    assert_eq!(
        serde_json::to_string(&missing).expect("serialize"),
        r#"{"field":"speechToSpeechRealtimeUrl","key":"SPEECH_TO_SPEECH_REALTIME_URL","message":"缺少服务地址"}"#,
    );
    assert_eq!(
        serde_json::from_str::<MissingSetting>(
            r#"{"field":"dashscopeApiKey","key":"DASHSCOPE_API_KEY","message":"m"}"#
        )
        .expect("deserialize"),
        MissingSetting {
            field: "dashscopeApiKey".to_owned(),
            key: "DASHSCOPE_API_KEY".to_owned(),
            message: "m".to_owned(),
        },
    );
}

/// Display is a developer-facing rendering; the user-facing localized message
/// is `via-i18n`'s, keyed by `code()`. What matters here is that the structured
/// data a caller needs survives into the text.
#[test]
fn display_names_every_missing_key() {
    let error = ProtocolError::SetupRequired {
        missing: vec![
            MissingSetting {
                field: "dashscopeApiKey".to_owned(),
                key: "DASHSCOPE_API_KEY".to_owned(),
                message: "not set".to_owned(),
            },
            MissingSetting {
                field: "speechToSpeechRealtimeUrl".to_owned(),
                key: "SPEECH_TO_SPEECH_REALTIME_URL".to_owned(),
                message: "not set".to_owned(),
            },
        ],
    };
    assert_eq!(
        error.to_string(),
        "gateway start refused; missing required configuration: \
         DASHSCOPE_API_KEY (not set); SPEECH_TO_SPEECH_REALTIME_URL (not set)",
    );
}

#[test]
fn already_running_names_the_incumbent_origin_only_when_it_has_one() {
    assert_eq!(
        ProtocolError::AlreadyRunning {
            origin: Some("http://127.0.0.1:8787".to_owned()),
        }
        .to_string(),
        "a Gateway is already running: http://127.0.0.1:8787",
    );
    assert_eq!(
        ProtocolError::AlreadyRunning { origin: None }.to_string(),
        "a Gateway is already running",
    );
}

#[test]
fn illegal_transition_names_both_ends_by_their_wire_strings() {
    assert_eq!(
        ProtocolError::IllegalTransition {
            from: WorkStatus::Cancelling,
            to: WorkStatus::Completed,
        }
        .to_string(),
        "illegal Work transition: cancelling -> completed",
    );
}

#[test]
fn unknown_wire_value_names_the_vocabulary_that_rejected_it() {
    assert_eq!(
        ProtocolError::UnknownWireValue {
            vocabulary: "GatewayTaskEvent",
            value: "task.notification.pending".to_owned(),
        }
        .to_string(),
        "`task.notification.pending` is not a member of the GatewayTaskEvent vocabulary",
    );
}

/// `thiserror` gives us `std::error::Error`, which is what lets callers box
/// these behind `anyhow` without losing the code.
#[test]
fn protocol_error_is_a_std_error() {
    fn assert_error<E: std::error::Error + Send + Sync + 'static>(_: &E) {}
    assert_error(&ProtocolError::InputOwnerRequired);
}
