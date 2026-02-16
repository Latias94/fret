//! WebSocket transport for Fret diagnostics / devtools.
//!
//! This crate provides optional client/server glue used by diagnostics tooling (e.g. `fretboard`)
//! to exchange [`fret_diag_protocol`] messages over WebSockets.
//!
//! Feature flags:
//! - `client-native`, `client-wasm`: WebSocket client implementations for native/wasm targets.
//! - `server-native`: a native WebSocket server for local devtools sessions.

#[cfg(any(feature = "client-native", feature = "client-wasm"))]
pub mod client;

#[cfg(feature = "server-native")]
pub mod server;
