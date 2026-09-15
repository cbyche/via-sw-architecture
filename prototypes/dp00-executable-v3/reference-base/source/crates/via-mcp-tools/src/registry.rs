//! The bearer-token → context table, as an **owning task**.
//!
//! Upstream keeps `this.contexts`, a plain `Map`, and mutates it from
//! `register` / `update` / `release` while `handleRequest` reads it
//! (`server/src/agent/acp-session-tools.mjs:143,180-201,209`). JavaScript's
//! single-threaded event loop makes that ordered for free; Rust's does not,
//! and the ordering is the contract. Upstream's own test states it:
//!
//! ```js
//! // server/test/acp-session-tools.test.mjs:78-95, with the tool renamed.
//! assert.equal(registration.update({ … }), true)
//! const updated = await client.callTool({ name: 'via_sessions_list', … })
//! assert.equal(JSON.parse(updated.content[0].text).sessions[0].session_id, 'two')
//! ```
//!
//! *The call issued after `update` returned must see the new context.* A
//! `tokio::sync::Mutex` would give mutual exclusion and say nothing about
//! order (`docs/architecture.md` §11), so the table is held **by value in one
//! task** and every operation is a [`Command`] on a bounded `mpsc`. A command
//! whose reply has arrived has been applied, and anything sent afterwards is
//! processed afterwards. Shutdown is by dropping senders, or explicitly with
//! [`Command::Close`].
//!
//! The task is never blocked by a running tool: [`Command::Lookup`] hands back
//! a cloned `Arc` and the caller awaits the tool outside the task.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use crate::context::SessionToolContext;

/// A context, shared between the registration that owns it and any request
/// currently running a tool against it.
pub type SharedContext = Arc<dyn SessionToolContext>;

/// How many commands may be in flight before a caller waits.
///
/// VIA's own — upstream has no queue. Bounded rather than unbounded because
/// `docs/architecture.md` §11 says so: an unbounded channel converts a stall
/// into unbounded memory rather than into backpressure. The depth only has to
/// absorb the registrations and lookups of concurrently-connecting backends,
/// of which there are single digits.
pub const REGISTRY_QUEUE_DEPTH: usize = 64;

/// One operation on the table.
#[derive(Debug)]
pub enum Command {
    /// Add a token.
    Register {
        /// The bearer token.
        token: String,
        /// The context it resolves to.
        context: SharedContext,
        /// Acknowledged once applied.
        reply: oneshot::Sender<()>,
    },
    /// Point an existing token at a different context.
    ///
    /// Replies `false` when the token is not in the table — released, or from
    /// a server that has since closed.
    Update {
        /// The bearer token.
        token: String,
        /// The context it should resolve to from now on.
        context: SharedContext,
        /// Whether the token was there to update.
        reply: oneshot::Sender<bool>,
    },
    /// Drop a token. Replies whether it was there.
    ///
    /// The reply is optional so a `Drop` impl can fire one without a runtime
    /// to await in.
    Release {
        /// The bearer token.
        token: String,
        /// Whether the token was there to remove.
        reply: Option<oneshot::Sender<bool>>,
    },
    /// Resolve a token, cloning the `Arc` out.
    Lookup {
        /// The bearer token presented on the request.
        token: String,
        /// The context, or `None` for an unknown token.
        reply: oneshot::Sender<Option<SharedContext>>,
    },
    /// Clear the table.
    ///
    /// Upstream's `close()` begins `this.contexts.clear()`
    /// (`acp-session-tools.mjs:252`): once the server is closing, a request
    /// that is still in flight must not resolve a token.
    ///
    /// The task deliberately **keeps running** afterwards, as upstream's
    /// cleared `Map` keeps existing. A request that got past the accept before
    /// the listener closed then gets a definitive "no such token" from the
    /// table rather than a dead channel, and it stops on the same 404 either
    /// way. The task ends when the last [`RegistryHandle`] drops, which is the
    /// shutdown mechanism `docs/architecture.md` §11 prescribes.
    Close {
        /// Acknowledged once the table is empty.
        reply: oneshot::Sender<()>,
    },
}

/// The task. Owns the table; nothing else may touch it.
pub async fn run(mut commands: mpsc::Receiver<Command>) {
    let mut contexts: HashMap<String, SharedContext> = HashMap::new();
    while let Some(command) = commands.recv().await {
        match command {
            Command::Register {
                token,
                context,
                reply,
            } => {
                contexts.insert(token, context);
                let _ = reply.send(());
            }
            Command::Update {
                token,
                context,
                reply,
            } => {
                let present = contexts.contains_key(&token);
                if present {
                    contexts.insert(token, context);
                }
                let _ = reply.send(present);
            }
            Command::Release { token, reply } => {
                let removed = contexts.remove(&token).is_some();
                if let Some(reply) = reply {
                    let _ = reply.send(removed);
                }
            }
            Command::Lookup { token, reply } => {
                let _ = reply.send(contexts.get(&token).cloned());
            }
            Command::Close { reply } => {
                contexts.clear();
                let _ = reply.send(());
            }
        }
    }
}

/// The caller's end of the table.
///
/// Cloning is how the axum app, every registration and the server itself all
/// address the same task. When the last handle drops the task ends, which is
/// the shutdown mechanism `docs/architecture.md` §11 prescribes.
#[derive(Debug, Clone)]
pub struct RegistryHandle {
    commands: mpsc::Sender<Command>,
}

impl RegistryHandle {
    /// Spawn the task and return a handle to it.
    ///
    /// Must be called from inside a tokio runtime; every caller reaches it
    /// through [`SessionToolServer::start`](crate::SessionToolServer::start),
    /// which is `async` and therefore always is.
    #[must_use]
    pub fn spawn() -> Self {
        let (commands, receiver) = mpsc::channel(REGISTRY_QUEUE_DEPTH);
        tokio::spawn(run(receiver));
        Self { commands }
    }

    /// Add a token. A closed task is not an error: the server is going away
    /// and the registration will resolve nothing, which is the correct
    /// behaviour for a released token too.
    pub async fn register(&self, token: String, context: SharedContext) {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Register {
                token,
                context,
                reply,
            })
            .await
            .is_ok()
        {
            let _ = answer.await;
        }
    }

    /// Point a token at a different context. `false` when it is not there.
    pub async fn update(&self, token: String, context: SharedContext) -> bool {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Update {
                token,
                context,
                reply,
            })
            .await
            .is_err()
        {
            return false;
        }
        answer.await.unwrap_or(false)
    }

    /// Remove a token. `false` when it was not there.
    pub async fn release(&self, token: String) -> bool {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Release {
                token,
                reply: Some(reply),
            })
            .await
            .is_err()
        {
            return false;
        }
        answer.await.unwrap_or(false)
    }

    /// Remove a token without waiting for the answer.
    ///
    /// For [`Drop`], which has no runtime to await in. A full queue means the
    /// token outlives its registration until the server closes — the same
    /// exposure upstream has for a registration that is simply never
    /// released.
    pub fn release_detached(&self, token: String) {
        let _ = self
            .commands
            .try_send(Command::Release { token, reply: None });
    }

    /// Resolve a token. `None` for an unknown one, and `None` once the task
    /// has stopped — a closing server authenticates nobody.
    pub async fn lookup(&self, token: String) -> Option<SharedContext> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Lookup { token, reply })
            .await
            .ok()?;
        answer.await.ok().flatten()
    }

    /// Clear the table.
    ///
    /// The task stays up until the last handle drops; see [`Command::Close`].
    pub async fn close(&self) {
        let (reply, answer) = oneshot::channel();
        if self.commands.send(Command::Close { reply }).await.is_ok() {
            let _ = answer.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::RecordingContext;

    fn context(label: &str) -> SharedContext {
        Arc::new(RecordingContext::new().labelled(label))
    }

    #[tokio::test]
    async fn a_registered_token_resolves_and_a_stranger_does_not() {
        let registry = RegistryHandle::spawn();
        registry.register("t".to_owned(), context("one")).await;
        assert!(registry.lookup("t".to_owned()).await.is_some());
        assert!(registry.lookup("other".to_owned()).await.is_none());
        assert!(registry.lookup(String::new()).await.is_none());
    }

    #[tokio::test]
    async fn an_update_is_visible_to_the_very_next_lookup() {
        let registry = RegistryHandle::spawn();
        registry.register("t".to_owned(), context("one")).await;
        assert!(registry.update("t".to_owned(), context("two")).await);
        let resolved = registry.lookup("t".to_owned()).await.expect("a context");
        assert!(format!("{resolved:?}").contains("two"));
    }

    #[tokio::test]
    async fn updating_an_unknown_token_is_false_and_does_not_create_it() {
        let registry = RegistryHandle::spawn();
        assert!(!registry.update("ghost".to_owned(), context("x")).await);
        assert!(registry.lookup("ghost".to_owned()).await.is_none());
    }

    #[tokio::test]
    async fn release_is_idempotent() {
        let registry = RegistryHandle::spawn();
        registry.register("t".to_owned(), context("one")).await;
        assert!(registry.release("t".to_owned()).await);
        assert!(!registry.release("t".to_owned()).await);
        assert!(registry.lookup("t".to_owned()).await.is_none());
    }

    #[tokio::test]
    async fn a_closed_registry_authenticates_nobody() {
        let registry = RegistryHandle::spawn();
        registry.register("a".to_owned(), context("one")).await;
        registry.register("b".to_owned(), context("two")).await;
        registry.close().await;
        // Every token, not merely the one that was asked about.
        for token in ["a", "b"] {
            assert!(registry.lookup(token.to_owned()).await.is_none(), "{token}");
            assert!(!registry.update(token.to_owned(), context("three")).await);
            assert!(!registry.release(token.to_owned()).await);
        }
        // And a token registered after the close is not resurrected either:
        // the server is going away, so nothing may be served from it.
        registry.close().await;
        assert!(registry.lookup("a".to_owned()).await.is_none());
    }

    #[tokio::test]
    async fn the_task_ends_when_the_last_handle_drops() {
        let registry = RegistryHandle::spawn();
        registry.register("t".to_owned(), context("one")).await;
        let clone = registry.clone();
        drop(registry);
        // A surviving clone still addresses a live task.
        assert!(clone.lookup("t".to_owned()).await.is_some());
        drop(clone);
        // Nothing to assert against once the last handle is gone — which is
        // the point: there is no other way to reach the table, so the task has
        // no reader left and returns.
    }

    #[tokio::test]
    async fn two_registrations_do_not_see_each_others_tokens() {
        let registry = RegistryHandle::spawn();
        registry.register("a".to_owned(), context("first")).await;
        registry.register("b".to_owned(), context("second")).await;
        assert!(registry.update("a".to_owned(), context("first-v2")).await);
        let b = registry.lookup("b".to_owned()).await.expect("still there");
        assert!(format!("{b:?}").contains("second"));
    }

    #[tokio::test]
    async fn a_detached_release_still_removes_the_token() {
        let registry = RegistryHandle::spawn();
        registry.register("t".to_owned(), context("one")).await;
        registry.release_detached("t".to_owned());
        // The detached command is queued ahead of this lookup, and the task is
        // FIFO, so the lookup cannot observe the token afterwards.
        assert!(registry.lookup("t".to_owned()).await.is_none());
    }
}
