//! A scripted fake WebSocket peer, plus the contract catalogue reader.
//!
//! The peer is the whole point of this crate's test suite: every one of the four
//! documented gateway misbehaviours (ARGO bring-up §4) is a *server* behaviour,
//! and none of them can be reproduced by a provider double. So the tests stand
//! up a real [`TcpListener`] on port 0, speak a real WebSocket handshake, and
//! tell the peer to misbehave in one named way at a time.

#![allow(dead_code)]

#[allow(unused_imports)]
pub mod catalogue;
pub mod gateway;

#[allow(unused_imports)]
pub use catalogue::{Contract, contract_of_kind};
#[allow(unused_imports)]
pub use gateway::{Behaviour, FakeGateway, HangingPeer, ObservedRequest, Rejection};
