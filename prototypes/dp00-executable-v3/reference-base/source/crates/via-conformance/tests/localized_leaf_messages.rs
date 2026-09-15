//! The leaf crates' Chinese literals, pinned to the `via-i18n` catalog.
//!
//! Phase 0 left `TODO(via-i18n)` on six literals in `via-log` and `via-lock`,
//! each promising that the string would "become a lookup" once the catalog
//! landed. `via-i18n` has landed, and **the move those markers describe cannot
//! happen**: `docs/architecture.md` §9's adjacency table gives `shared → ∅`,
//! and `via-arch-test`'s `leaf_violations` enforces it — a Leaf-band crate may
//! not depend on a Core-band one, of any kind, so `via-log` and `via-lock`
//! cannot call `via_i18n::t`. (`via-protocol` took the other exit: its
//! `Display` is developer-facing English and the user-facing sentence lives
//! only in the catalog. `via-log`'s and `via-lock`'s messages *are* the
//! contract other code matches on, so that exit is not open to them.)
//!
//! What was owed is therefore not a refactor but a **guard**: the literal and
//! the catalog entry now say the same thing in two places, and nothing stopped
//! them drifting. This file is that guard. It is here rather than in either
//! crate because a test crate is the only place in the graph allowed to see
//! both bands at once (`via-arch-test`: "a test crate may reach every band").
//!
//! `via-i18n`'s own `the_phase_zero_todo_keys_exist_with_the_recorded_text`
//! asserts the catalog side against retyped text. This asserts the catalog
//! against the *shipped constants*, which is the half that was missing.

use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use pretty_assertions::assert_eq;
use via_i18n::{Locale, format, keys, t};
use via_lock::{
    AcquireOptions, CliAcquireOptions, CliLockError, Clock, LeaseError, LeaseUpdate, ProcessProbe,
    SignalOutcome, acquire_cli_instance, acquire_gateway_lease,
};
use via_log::sink::{LOG_SINK_FAILURE_PREFIX_ZH, log_sink_failure_message};

/// A fixed instant, so a lease document is a test input rather than a reading.
#[derive(Debug)]
struct FrozenClock(DateTime<Utc>);

impl Clock for FrozenClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

fn frozen() -> Box<dyn Clock> {
    Box::new(FrozenClock(
        Utc.with_ymd_and_hms(2026, 8, 22, 10, 36, 0)
            .single()
            .expect("2026-08-22T10:36:00Z is a real, unambiguous instant"),
    ))
}

#[derive(Debug)]
struct Fixed(SignalOutcome);

impl ProcessProbe for Fixed {
    fn signal_zero(&self, _pid: i64) -> SignalOutcome {
        self.0
    }
}

fn dead() -> Box<dyn ProcessProbe> {
    Box::new(Fixed(SignalOutcome::NoSuchProcess))
}

fn alive() -> Box<dyn ProcessProbe> {
    Box::new(Fixed(SignalOutcome::Delivered))
}

const INSTANCE_ID: &str = "11111111-2222-3333-4444-555555555555";
const CHALLENGER_ID: &str = "22222222-3333-4444-5555-666666666666";

/// Provoke a real lease conflict against a live incumbent with `origin`.
fn refused_lease(directory: &Path, origin: &str) -> LeaseError {
    let mut incumbent = acquire_gateway_lease(
        directory,
        AcquireOptions::new()
            .pid(4242)
            .owner("desktop")
            .instance_id(INSTANCE_ID)
            .clock(frozen())
            .probe(dead()),
    )
    .expect("an empty directory has no incumbent");
    if !origin.is_empty() {
        incumbent
            .update(LeaseUpdate::new().origin(origin))
            .expect("the incumbent publishes its origin");
    }

    acquire_gateway_lease(
        directory,
        AcquireOptions::new()
            .pid(99)
            .instance_id(CHALLENGER_ID)
            .clock(frozen())
            .probe(alive()),
    )
    .expect_err("a live incumbent refuses")
}

/// `via-lock`'s two lease messages are the catalog's `zh` values.
///
/// Both spellings of the conflict — with and without the incumbent's origin —
/// because they are two catalog keys and upstream builds them from one sentence
/// plus a conditional suffix.
#[test]
fn the_gateway_lease_messages_are_the_catalog_zh_values() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let with_origin = refused_lease(dir.path(), "http://127.0.0.1:8765");
    assert_eq!(
        with_origin.to_string(),
        format(
            Locale::Zh,
            keys::LOCK_GATEWAY_ALREADY_RUNNING_AT,
            &[("origin", "http://127.0.0.1:8765")],
        ),
    );

    let bare = tempfile::TempDir::new().expect("tempdir");
    let without_origin = refused_lease(bare.path(), "");
    assert_eq!(
        without_origin.to_string(),
        t(Locale::Zh, keys::LOCK_GATEWAY_ALREADY_RUNNING),
        "with no origin the suffix is absent, which is the bare key"
    );

    assert_eq!(
        LeaseError::Exhausted.to_string(),
        t(Locale::Zh, keys::LOCK_LEASE_EXHAUSTED),
    );

    // The three keys really are localized: `en` is not the same bytes, so this
    // is pinning a Chinese value against a Chinese value and not against a
    // catalog that fell back to one locale for everything.
    for key in [
        keys::LOCK_GATEWAY_ALREADY_RUNNING,
        keys::LOCK_LEASE_EXHAUSTED,
    ] {
        assert_ne!(t(Locale::Zh, key), t(Locale::En, key));
        assert_ne!(t(Locale::Zh, key), t(Locale::Ko, key));
    }
}

/// `via-lock`'s two CLI-lock messages are the catalog's `zh` values.
///
/// These carry `docs/rebrand.md`'s binary rename inside the sentence, so the
/// catalog and the constant have to agree on the renamed text as well as on
/// the rest of it.
#[test]
fn the_cli_lock_messages_are_the_catalog_zh_values() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let _held = acquire_cli_instance(
        dir.path(),
        CliAcquireOptions::new().pid(1).token("a").probe(dead()),
    )
    .expect("first acquire");

    let refused = acquire_cli_instance(
        dir.path(),
        CliAcquireOptions::new().pid(2).token("b").probe(alive()),
    )
    .expect_err("a live holder refuses");

    assert_eq!(
        refused.to_string(),
        t(Locale::Zh, keys::LOCK_CLI_ALREADY_RUNNING),
    );
    assert_eq!(
        CliLockError::Exhausted.to_string(),
        t(Locale::Zh, keys::LOCK_CLI_LEASE_EXHAUSTED),
    );
}

/// `via-log`'s sink-failure prefix is the catalog's `zh` value with the detail
/// removed.
///
/// The constant is a *prefix* — the crate appends the scrubbed detail and a
/// newline — and the catalog entry is the whole sentence with a `{detail}`
/// placeholder, so the comparison is: render the key with an empty detail, and
/// that is the prefix.
#[test]
fn the_log_sink_failure_prefix_is_the_catalog_zh_value() {
    assert_eq!(
        LOG_SINK_FAILURE_PREFIX_ZH,
        format(Locale::Zh, keys::LOG_SINK_WRITE_FAILED, &[("detail", "")]),
    );
    assert_eq!(
        log_sink_failure_message("disk full"),
        std::format!(
            "{}\n",
            format(
                Locale::Zh,
                keys::LOG_SINK_WRITE_FAILED,
                &[("detail", "disk full")],
            )
        ),
    );
}

/// The band rule these three tests exist because of.
///
/// If `via-log` or `via-lock` ever grows the `via-i18n` edge the phase-0
/// markers promised, `via-arch-test`'s `leaf_violations` fails first — but that
/// crate is not in this one's graph, so this states the rule where the
/// duplication is, and names the rule that keeps it.
#[test]
fn the_leaf_crates_carry_no_i18n_edge() {
    let manifests = [
        ("via-log", include_str!("../../via-log/Cargo.toml")),
        ("via-lock", include_str!("../../via-lock/Cargo.toml")),
    ];
    for (name, manifest) in manifests {
        assert!(
            !manifest.contains("via-i18n"),
            "{name} has grown a via-i18n dependency; `docs/architecture.md` §9 gives \
             `shared -> nothing`, and via-arch-test::leaf_violations enforces it"
        );
    }
}
