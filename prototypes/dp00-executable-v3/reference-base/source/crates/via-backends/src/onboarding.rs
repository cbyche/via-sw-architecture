//! The unified onboarding contract CLI and desktop both render.
//!
//! Ported from `shared/backend-onboarding.mjs` and the compatibility wrappers in
//! `shared/backend-lifecycle.mjs`. A backend owns the details of its own
//! configuration; a caller only renders and invokes trusted actions, which is
//! why the action is a closed shape with an `id` and a `kind` rather than a free
//! command string.
//!
//! # The one rule worth restating
//!
//! `readiness.status` is **always** [`Readiness::NotConnected`]
//! (`backend-onboarding.mjs:74-76`). Runtime readiness is measured by the live
//! connection and is never inferred from an installed binary or a credential
//! file on disk — the whole point of keeping `installed`, `configured` and
//! `connected` as three separate facts.

use via_catalog::backend::{AuthProbe, ConfigurationMode, InstallationSpec};
use via_catalog::{backend_definition, normalize_backend_protocol};
use via_core::EnvMap;
use via_core::config::backend::AUTO_MODEL_SENTINEL;
use via_core::config::names;
use via_i18n::{Key, Locale, keys, t};

use crate::install::{InstallSteps, install_steps};
use crate::platform::HostPlatform;

/// The action id upstream stamps on every configuration action.
///
/// **External contract** — `shared/backend-onboarding.mjs:33`, asserted
/// verbatim by `server/test/backend-onboarding.test.mjs:52-61`.
pub const CONFIGURE_ACTION_ID: &str = "configure";

/// The action kind upstream stamps on every configuration action.
///
/// **External contract** — `shared/backend-onboarding.mjs:34`. A `terminal`
/// action is a command a human runs in their own shell; the settings UI is
/// deliberately not given a way to run it silently.
pub const CONFIGURE_ACTION_KIND: &str = "terminal";

/// Where a backend sits on the install → configure → connect path.
///
/// **External contract** — `state-name/onboarding state machine values`:
/// `'not-installed' | 'installed' | 'configuration-required'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OnboardingState {
    /// Nothing on disk yet.
    NotInstalled,
    /// Installed and configured as far as VIA can tell.
    Installed,
    /// Installed, but its own sign-in is still needed.
    ConfigurationRequired,
}

/// Whether the backend's executable is present.
///
/// **External contract** — `installation.status: 'installed' | 'not-installed'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallationStatus {
    /// Present.
    Installed,
    /// Absent.
    NotInstalled,
}

/// Runtime readiness, which is never inferred here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Readiness {
    /// The only value this type ever takes — see the module docs.
    NotConnected,
}

/// What a backend's own sign-in looks like.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationAction {
    /// Always [`CONFIGURE_ACTION_ID`].
    pub id: &'static str,
    /// Always [`CONFIGURE_ACTION_KIND`].
    pub kind: &'static str,
    /// The rendered button label (`配置`).
    pub label: String,
    /// The command a human runs, verbatim from the catalog.
    pub command: &'static str,
    /// One sentence of guidance.
    pub hint: String,
}

/// A backend's configuration half of the onboarding contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationSupport {
    /// Who owns the model configuration.
    pub mode: ConfigurationMode,
    /// Whether VIA configures this backend itself, making a sign-in
    /// unnecessary.
    pub automatic: bool,
    /// The trusted action, or `None` when there is nothing to offer.
    pub action: Option<ConfigurationAction>,
    /// How VIA checks whether configuration already happened.
    pub probe: Option<AuthProbe>,
}

/// The whole onboarding adapter for one backend.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingAdapter {
    /// The catalogued id.
    pub id: String,
    /// The installation plan, or `None` for a backend VIA never installs.
    pub installation: Option<InstallationSpec>,
    /// Configuration.
    pub configuration: ConfigurationSupport,
}

/// The resolved lifecycle a renderer consumes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedOnboarding {
    /// Where the backend sits.
    pub state: OnboardingState,
    /// Whether the executable is present.
    pub installation_status: InstallationStatus,
    /// Whether its own sign-in is still needed.
    pub configuration_required: bool,
    /// Always [`Readiness::NotConnected`].
    pub readiness: Readiness,
}

/// Whether a backend needs its own sign-in at all, and whether VIA can point at
/// one.
///
/// **External contract** — `backendAuthenticationSupport`,
/// `shared/backend-lifecycle.mjs:26-41`, asserted by
/// `server/test/backend-lifecycle.test.mjs:22-32,50-56`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationSupport {
    /// Whether a sign-in exists for this backend.
    pub required: bool,
    /// Whether VIA knows a trustworthy official entry point.
    pub supported: bool,
    /// That entry point.
    pub command: Option<&'static str>,
}

/// The onboarding hint for a backend, as an i18n key.
///
/// Upstream stores the sentence on the catalog entry
/// (`shared/backend-catalog.mjs`, `onboarding.hint`). `via-catalog` deliberately
/// does not carry it — `docs/fidelity.md` puts user-facing prose in `via-i18n` —
/// so the id → key mapping lives here, which is the crate allowed to name a
/// backend. `None` for `acp`, which has no onboarding at all.
#[must_use]
pub fn onboarding_hint_key(id: &str) -> Option<Key> {
    Some(match id {
        "opencode" => keys::BACKEND_ONBOARDING_OPENCODE,
        "openclaw" => keys::BACKEND_ONBOARDING_OPENCLAW,
        "qoder" => keys::BACKEND_ONBOARDING_QODER,
        // The third-party Qwen Code CLI, not VIA.
        "qwen" => keys::BACKEND_ONBOARDING_QWEN_CODE,
        "kimi" => keys::BACKEND_ONBOARDING_KIMI,
        "hermes" => keys::BACKEND_ONBOARDING_HERMES,
        "codebuddy" => keys::BACKEND_ONBOARDING_CODEBUDDY,
        "codex" => keys::BACKEND_ONBOARDING_CODEX,
        "claude" => keys::BACKEND_ONBOARDING_CLAUDE_CODE,
        "deepseek" => keys::BACKEND_ONBOARDING_DEEPSEEK,
        "pi" => keys::BACKEND_ONBOARDING_PI,
        _ => return None,
    })
}

/// Whether VIA configures this backend itself rather than asking for a sign-in.
///
/// **External contract** — `usesAutomaticBailianConfiguration`,
/// `shared/backend-onboarding.mjs:7-15`. Only OpenCode and OpenClaw, only with
/// a DashScope key **and** a concrete backend model. `auto` is the sentinel for
/// *"let the backend decide"*, so it does not count as a concrete model.
#[must_use]
pub fn uses_automatic_bailian_configuration(id: &str, env: &EnvMap) -> bool {
    let model = env.get_trimmed(names::BACKEND_MODEL).to_lowercase();
    matches!(id, "opencode" | "openclaw")
        && !env.get_trimmed(names::DASHSCOPE_API_KEY).is_empty()
        && !model.is_empty()
        && model != AUTO_MODEL_SENTINEL
}

/// `backendOnboardingAdapter(id, { env, platform })`.
///
/// **External contract** — `shared/backend-onboarding.mjs:21-49`. `platform` is
/// `None` for a host outside upstream's closed `darwin | linux | win32` set,
/// which suppresses the action exactly as
/// `['darwin','linux','win32'].includes(platform)` does.
#[must_use]
pub fn backend_onboarding_adapter(
    id: &str,
    env: &EnvMap,
    platform: Option<HostPlatform>,
    locale: Locale,
) -> OnboardingAdapter {
    let definition = backend_definition(id);
    let resolved_id = definition.map_or_else(
        || normalize_backend_protocol(id),
        |entry| entry.id.to_owned(),
    );
    let automatic = uses_automatic_bailian_configuration(&resolved_id, env);
    let command = definition
        .and_then(|entry| entry.onboarding)
        .map(|onboarding| onboarding.command)
        .filter(|command| !command.trim().is_empty());
    let action = match (command, automatic, platform) {
        (Some(command), false, Some(_)) => Some(ConfigurationAction {
            id: CONFIGURE_ACTION_ID,
            kind: CONFIGURE_ACTION_KIND,
            label: t(locale, keys::BACKEND_INSTALL_STEP_LABEL_CONFIGURE).to_owned(),
            command,
            hint: onboarding_hint_key(&resolved_id)
                .map(|key| t(locale, key).to_owned())
                .unwrap_or_default(),
        }),
        _ => None,
    };
    OnboardingAdapter {
        id: resolved_id,
        installation: definition.and_then(|entry| entry.lifecycle.installation),
        configuration: ConfigurationSupport {
            mode: definition.map_or(ConfigurationMode::UserManaged, |entry| {
                entry.lifecycle.configuration
            }),
            automatic,
            action,
            probe: definition
                .and_then(|entry| entry.onboarding)
                .and_then(|onboarding| onboarding.probe),
        },
    }
}

/// `backendConfigurationAction(id, options)` —
/// `shared/backend-onboarding.mjs:51-53`.
#[must_use]
pub fn backend_configuration_action(
    id: &str,
    env: &EnvMap,
    platform: Option<HostPlatform>,
    locale: Locale,
) -> Option<ConfigurationAction> {
    backend_onboarding_adapter(id, env, platform, locale)
        .configuration
        .action
}

/// `backendAuthenticationSupport(id, { env, platform })` —
/// `shared/backend-lifecycle.mjs:26-41`.
///
/// A backend with no offered action is reported as needing no authentication at
/// all — which is how automatic Bailian configuration makes OpenCode's login
/// disappear from the settings UI, and how the generic ACP backend never asks
/// for one.
#[must_use]
pub fn backend_authentication_support(
    id: &str,
    env: &EnvMap,
    platform: Option<HostPlatform>,
    locale: Locale,
) -> AuthenticationSupport {
    match backend_configuration_action(id, env, platform, locale) {
        None => AuthenticationSupport {
            required: false,
            supported: false,
            command: None,
        },
        Some(action) => AuthenticationSupport {
            required: true,
            supported: true,
            command: Some(action.command),
        },
    }
}

/// `resolveBackendOnboarding(item, { installation, configuration })` —
/// `shared/backend-onboarding.mjs:55-77`.
#[must_use]
pub fn resolve_backend_onboarding(
    installed: bool,
    configuration_required: bool,
) -> ResolvedOnboarding {
    let state = if installed && configuration_required {
        OnboardingState::ConfigurationRequired
    } else if installed {
        OnboardingState::Installed
    } else {
        OnboardingState::NotInstalled
    };
    ResolvedOnboarding {
        state,
        installation_status: if installed {
            InstallationStatus::Installed
        } else {
            InstallationStatus::NotInstalled
        },
        configuration_required,
        readiness: Readiness::NotConnected,
    }
}

/// The installation plan for a backend on one platform.
///
/// `specSteps(id, platform)` — `shared/backend-install.mjs:46-57`: the catalog's
/// spec, with the steps filtered to those that apply here. A backend with no
/// installation spec at all (`acp`) yields `None`.
#[must_use]
pub fn backend_install_steps(id: &str, platform: HostPlatform) -> Option<InstallSteps> {
    let definition = backend_definition(id)?;
    definition.lifecycle.installation?;
    Some(install_steps(definition.id, platform))
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_catalog::backend_names;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    /// `server/test/backend-onboarding.test.mjs:11-23`.
    const EXPECTED_COMMANDS: [(&str, &str); 11] = [
        ("opencode", "opencode auth login"),
        ("openclaw", "openclaw onboard"),
        ("qoder", "qodercli login"),
        ("qwen", "qwen"),
        ("kimi", "kimi login"),
        ("hermes", "hermes setup --portal"),
        ("codebuddy", "codebuddy"),
        ("codex", "codex login"),
        ("claude", "claude"),
        ("pi", "pi"),
        ("deepseek", "dsh web"),
    ];

    #[test]
    fn every_product_backend_owns_an_explicit_configuration_adapter() {
        let expected: std::collections::BTreeMap<&str, &str> =
            EXPECTED_COMMANDS.into_iter().collect();
        for id in backend_names() {
            let action = backend_configuration_action(
                id,
                &EnvMap::new(),
                Some(HostPlatform::Darwin),
                Locale::Zh,
            );
            if id == "acp" {
                assert!(action.is_none(), "acp offers no configuration action");
                continue;
            }
            let action = action.unwrap_or_else(|| panic!("{id} declares an action"));
            assert_eq!(Some(action.command), expected.get(id).copied(), "{id}");
            assert_eq!(action.kind, CONFIGURE_ACTION_KIND, "{id}");
            assert!(!action.hint.is_empty(), "{id} has a hint");
        }
    }

    #[test]
    fn the_action_shape_is_uniform() {
        let action = backend_configuration_action(
            "qoder",
            &EnvMap::new(),
            Some(HostPlatform::Darwin),
            Locale::Zh,
        )
        .expect("qoder declares one");
        assert_eq!(
            action,
            ConfigurationAction {
                id: "configure",
                kind: "terminal",
                label: "配置".to_owned(),
                command: "qodercli login",
                hint: "首次使用请完成 Qoder 官方认证。".to_owned(),
            }
        );
    }

    #[test]
    fn automatic_bailian_configuration_suppresses_the_action() {
        let configured = env(&[
            ("DASHSCOPE_API_KEY", "key"),
            ("VIA_BACKEND_MODEL", "qwen3-coder-plus"),
        ]);
        let adapter = backend_onboarding_adapter(
            "opencode",
            &configured,
            Some(HostPlatform::Darwin),
            Locale::Zh,
        );
        assert!(adapter.configuration.automatic);
        assert!(adapter.configuration.action.is_none());
        assert_eq!(
            backend_authentication_support(
                "opencode",
                &configured,
                Some(HostPlatform::Darwin),
                Locale::Zh
            ),
            AuthenticationSupport {
                required: false,
                supported: false,
                command: None,
            }
        );
    }

    #[test]
    fn the_auto_sentinel_is_not_a_concrete_model() {
        for value in ["auto", "AUTO", "Auto"] {
            let configured = env(&[("DASHSCOPE_API_KEY", "key"), ("VIA_BACKEND_MODEL", value)]);
            assert!(
                !uses_automatic_bailian_configuration("opencode", &configured),
                "{value}"
            );
        }
    }

    #[test]
    fn only_opencode_and_openclaw_configure_automatically() {
        let configured = env(&[
            ("DASHSCOPE_API_KEY", "key"),
            ("VIA_BACKEND_MODEL", "qwen3.7-max"),
        ]);
        for id in backend_names() {
            assert_eq!(
                uses_automatic_bailian_configuration(id, &configured),
                matches!(id, "opencode" | "openclaw"),
                "{id}"
            );
        }
    }

    #[test]
    fn deepseeks_product_setup_is_its_authentication_entry() {
        assert_eq!(
            backend_authentication_support(
                "deepseek",
                &EnvMap::new(),
                Some(HostPlatform::Darwin),
                Locale::Zh
            ),
            AuthenticationSupport {
                required: true,
                supported: true,
                command: Some("dsh web"),
            }
        );
    }

    #[test]
    fn an_unsupported_platform_offers_no_action() {
        assert!(
            backend_configuration_action("qoder", &EnvMap::new(), None, Locale::Zh).is_none(),
            "upstream gates the action on darwin | linux | win32"
        );
    }

    #[test]
    fn the_lifecycle_never_infers_runtime_readiness() {
        let resolved = resolve_backend_onboarding(true, true);
        assert_eq!(resolved.state, OnboardingState::ConfigurationRequired);
        assert_eq!(resolved.installation_status, InstallationStatus::Installed);
        assert_eq!(resolved.readiness, Readiness::NotConnected);

        assert_eq!(
            resolve_backend_onboarding(true, false).state,
            OnboardingState::Installed
        );
        // Configuration required while *not* installed is still `not-installed`:
        // there is nothing to configure yet.
        assert_eq!(
            resolve_backend_onboarding(false, true).state,
            OnboardingState::NotInstalled
        );
        assert_eq!(
            resolve_backend_onboarding(false, true).readiness,
            Readiness::NotConnected
        );
    }

    #[test]
    fn an_unknown_backend_falls_back_to_user_managed_with_no_action() {
        let adapter = backend_onboarding_adapter(
            "nope",
            &EnvMap::new(),
            Some(HostPlatform::Linux),
            Locale::En,
        );
        assert_eq!(adapter.id, "nope");
        assert_eq!(adapter.configuration.mode, ConfigurationMode::UserManaged);
        assert!(adapter.configuration.action.is_none());
        assert!(adapter.installation.is_none());
    }

    #[test]
    fn the_deepseek_probe_kind_is_declared() {
        let adapter = backend_onboarding_adapter(
            "deepseek",
            &EnvMap::new(),
            Some(HostPlatform::Darwin),
            Locale::Zh,
        );
        assert_eq!(
            adapter.configuration.probe.map(|probe| probe.kind),
            Some(via_catalog::backend::ProbeKind::DeepseekCredentials)
        );
    }
}
