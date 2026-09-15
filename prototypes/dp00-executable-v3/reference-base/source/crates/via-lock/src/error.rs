//! What can go wrong while taking, keeping or giving up the lease.

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::lease::{GatewayLease, VIA_GATEWAY_ALREADY_RUNNING};

/// A failure to acquire, update or release the Gateway lease.
///
/// The *codes* are contract; the shape around them is not (see
/// `docs/fidelity.md`). [`LeaseError::code`] is the accessor callers branch on.
#[derive(Debug, Error)]
pub enum LeaseError {
    /// Another Gateway holds the lease and its process is alive.
    ///
    /// **External contract.** Upstream `shared/gateway-instance-lock.mjs:154-159`
    /// throws an `Error` whose `code` is
    /// [`VIA_GATEWAY_ALREADY_RUNNING`](crate::VIA_GATEWAY_ALREADY_RUNNING) and
    /// whose `lease` property is the incumbent's document, boxed here only to
    /// keep the error small. The message is
    /// `已有 Gateway 正在运行` with `：<origin>` appended when the incumbent has
    /// published one.
    //
    // The literal below is upstream's `zh` string, and it stays a literal:
    // `docs/architecture.md` §9 gives `shared -> nothing`, so this Leaf-band
    // crate may not depend on `via-i18n` (`via-arch-test::leaf_violations`).
    // `via-i18n` carries the same sentence as `lock.gateway_already_running`
    // / `lock.gateway_already_running_at` for a caller that has a locale, and
    // `via-conformance`'s `tests/localized_leaf_messages.rs` asserts the two
    // are the same bytes so they cannot drift apart.
    #[error("已有 Gateway 正在运行{}", origin_suffix(&.lease.origin))]
    AlreadyRunning {
        /// The incumbent lease, exactly as it was read from disk.
        lease: Box<GatewayLease>,
    },

    /// Every acquisition attempt lost its race.
    ///
    /// **External contract.** Upstream `shared/gateway-instance-lock.mjs:165`
    /// throws `无法获取 Gateway 实例租约` after
    /// [`MAX_ACQUIRE_ATTEMPTS`](crate::MAX_ACQUIRE_ATTEMPTS) passes. Reaching
    /// this means the lease file was created by someone else and found dead
    /// four times running — a crash loop, not ordinary contention.
    //
    // A literal for the reason above; `lock.lease_exhausted` is its catalog
    // twin and `via-conformance` pins the two together.
    #[error("无法获取 Gateway 实例租约")]
    Exhausted,

    /// The filesystem refused an operation the lease depends on.
    #[error("gateway lease i/o failed at {path}")]
    Io {
        /// The path being created, written, renamed or removed.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
}

impl LeaseError {
    /// The stable error code a caller branches on, when this variant has one.
    ///
    /// Only the conflict carries a code upstream; the other two are plain
    /// `Error`s there, so they answer `None` rather than inventing a string no
    /// existing client knows.
    #[must_use]
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::AlreadyRunning { .. } => Some(VIA_GATEWAY_ALREADY_RUNNING),
            Self::Exhausted | Self::Io { .. } => None,
        }
    }

    /// The incumbent's lease, when this is a conflict.
    ///
    /// Mirrors upstream's `error.lease`, which the CLI reads to decide whether
    /// to attach to the running instance.
    #[must_use]
    pub fn lease(&self) -> Option<&GatewayLease> {
        match self {
            Self::AlreadyRunning { lease } => Some(lease),
            Self::Exhausted | Self::Io { .. } => None,
        }
    }

    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// `：<origin>` when the incumbent published an origin, otherwise nothing.
///
/// **External contract.** Upstream interpolates
/// `${existing.origin ? `：${existing.origin}` : ''}`
/// (`shared/gateway-instance-lock.mjs:155-156`) — note the fullwidth colon
/// `U+FF1A`, not an ASCII one.
fn origin_suffix(origin: &str) -> String {
    if origin.is_empty() {
        String::new()
    } else {
        format!("：{origin}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_suffix_is_a_fullwidth_colon() {
        assert_eq!(origin_suffix(""), "");
        assert_eq!(
            origin_suffix("http://127.0.0.1:3101"),
            "：http://127.0.0.1:3101"
        );
        assert_eq!(origin_suffix("x").chars().next(), Some('\u{ff1a}'));
    }
}
