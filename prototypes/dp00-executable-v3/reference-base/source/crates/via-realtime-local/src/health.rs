//! What `/api/health` says about the on-device path.
//!
//! `via-realtime`'s `ActiveRealtime` is a closed struct with no free-form slot,
//! and this crate does not edit it. So the note is a typed value this crate
//! publishes and the Gateway embeds beside the realtime block — the same shape
//! every other `/api/health` payload has, with the field order that is
//! observable because `serde_json` preserves insertion order.
//!
//! # Why there is a note at all
//!
//! Two facts about the on-device path are not derivable from the provider
//! surface, and both change what an operator should do:
//!
//! **Which mode is mounted.** `local-omni` is one provider key with two very
//! different implementations behind it (`docs/architecture.md` §7), and a
//! session's sample rates, its voice and whether it needs a server at all
//! follow from the mode. A health payload that said only `local-omni` would be
//! ambiguous about the thing most likely to be wrong.
//!
//! **Whether it has ever run.** `docs/architecture.md` §7 is explicit that
//! `local-omni:endpoint` is *"CUDA-only; cannot be verified on this machine"*,
//! and the delivery decision in §16 is that it *"ships unverified-on-hardware,
//! **saying so**"*. [`LocalHealth::verified`] and [`LocalHealth::note`] are that
//! sentence, in the operator's own language — not a comment in a source file
//! nobody reading `/api/health` will open.

use serde::{Deserialize, Serialize};
use via_i18n::{Locale, t};

use crate::mode::LocalMode;
use crate::settings::LocalSettings;

/// The on-device note `/api/health` carries.
///
/// Field order is the order it serializes in, and it is the order an operator
/// reads it in: *what is mounted*, *can it run*, *has it ever run*, *why not*.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalHealth {
    /// The qualified mode name — `local-omni:pipeline` or
    /// `local-omni:endpoint`.
    pub mode: String,
    /// Whether the mode has everything it needs: the weights, for the pipeline;
    /// an endpoint, for the local server.
    pub configured: bool,
    /// Whether this mode has ever been exercised against real hardware in this
    /// repository.
    ///
    /// `false` for [`LocalMode::Endpoint`], always. See the module docs.
    pub verified: bool,
    /// The sentence explaining an unverified mode, or `None` for a verified
    /// one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The model root the pipeline loads from, or `None` in endpoint mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_root: Option<String>,
    /// Every required model file that is absent, in pipeline order.
    ///
    /// Empty in endpoint mode and on a complete install. It is a list rather
    /// than a count because the fix is per-file, and an operator reading
    /// `/api/health` is exactly the person who has to create them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_files: Vec<String>,
    /// The address the endpoint mode dials, or `None` in pipeline mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

impl LocalHealth {
    /// Describe a configuration.
    #[must_use]
    pub fn describe(settings: &LocalSettings, locale: Locale) -> Self {
        let verified = settings.mode.is_verified_on_this_machine();
        let note = (!verified)
            .then(|| t(locale, via_i18n::keys::REALTIME_LOCAL_ENDPOINT_UNVERIFIED).to_owned());
        match settings.mode {
            LocalMode::Pipeline => {
                let missing: Vec<String> = settings
                    .weights
                    .missing()
                    .into_iter()
                    .map(|required| required.path.display().to_string())
                    .collect();
                Self {
                    mode: settings.mode.qualified(),
                    configured: missing.is_empty(),
                    verified,
                    note,
                    model_root: Some(settings.weights.root.display().to_string()),
                    missing_files: missing,
                    endpoint: None,
                }
            }
            LocalMode::Endpoint => Self {
                mode: settings.mode.qualified(),
                configured: settings.endpoint_configured(),
                verified,
                note,
                model_root: None,
                missing_files: Vec::new(),
                endpoint: Some(settings.endpoint.clone()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::weights::WeightsSet;

    #[test]
    fn the_pipeline_note_lists_what_is_missing_and_claims_no_unverified_status() {
        let health = LocalHealth::describe(
            &LocalSettings::default().with_model_root("/models/local-omni"),
            Locale::En,
        );
        assert_eq!(health.mode, "local-omni:pipeline");
        assert!(health.verified);
        assert_eq!(health.note, None);
        assert!(!health.configured);
        assert_eq!(health.model_root.as_deref(), Some("/models/local-omni"));
        assert_eq!(
            health.missing_files.len(),
            WeightsSet::resolve("/models/local-omni").required().len(),
            "nothing is installed, so every required path is listed"
        );
        assert_eq!(health.endpoint, None);
    }

    #[test]
    fn a_complete_install_is_configured_and_lists_nothing() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = WeightsSet::resolve(root.path());
        for required in set.required() {
            if required.is_directory {
                std::fs::create_dir_all(&required.path).expect("directory");
            } else {
                if let Some(parent) = required.path.parent() {
                    std::fs::create_dir_all(parent).expect("parent");
                }
                std::fs::write(&required.path, b"x").expect("file");
            }
        }
        let health = LocalHealth::describe(
            &LocalSettings::default().with_model_root(root.path()),
            Locale::En,
        );
        assert!(health.configured);
        assert!(health.missing_files.is_empty());
    }

    #[test]
    fn the_endpoint_note_says_it_is_unverified_in_every_locale() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let health = LocalHealth::describe(
                &LocalSettings::default().with_endpoint("ws://127.0.0.1:8000"),
                locale,
            );
            assert_eq!(health.mode, "local-omni:endpoint");
            assert!(health.configured);
            assert!(
                !health.verified,
                "docs/architecture.md §7: CUDA-only, cannot be verified on this machine"
            );
            let note = health
                .note
                .clone()
                .expect("an unverified mode carries a note");
            assert!(!note.is_empty(), "{locale}");
            assert!(!note.contains('{'), "{locale}: {note}");
        }
    }

    #[test]
    fn an_endpointless_endpoint_mode_is_reported_unconfigured() {
        let health = LocalHealth::describe(
            &LocalSettings::default().with_mode(LocalMode::Endpoint),
            Locale::En,
        );
        assert!(!health.configured);
        assert_eq!(health.endpoint.as_deref(), Some(""));
        assert_eq!(health.model_root, None);
    }

    #[test]
    fn the_note_serializes_in_the_order_an_operator_reads_it() {
        let health = LocalHealth::describe(
            &LocalSettings::default().with_endpoint("ws://127.0.0.1:8000"),
            Locale::En,
        );
        let value = serde_json::to_value(&health).expect("serialize");
        let keys: Vec<&str> = value
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_default();
        assert_eq!(keys, ["mode", "configured", "verified", "note", "endpoint"]);

        // A verified mode drops the note entirely rather than carrying `null`.
        let pipeline = LocalHealth::describe(&LocalSettings::default(), Locale::En);
        let value = serde_json::to_value(&pipeline).expect("serialize");
        assert_eq!(value.get("note"), None);
        assert_eq!(value["verified"], json!(true));
    }

    #[test]
    fn the_note_round_trips() {
        let health = LocalHealth::describe(
            &LocalSettings::default().with_endpoint("ws://127.0.0.1:8000"),
            Locale::Ko,
        );
        let text = serde_json::to_string(&health).expect("serialize");
        let back: LocalHealth = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(back, health);
    }
}
