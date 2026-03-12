//! Native WebView backend implementation for Fret's WebView contract.
//!
//! This crate owns the native `wry` integration and keeps platform-heavy dependencies out of
//! `fret-webview`, which remains a contract-only crate.
//!
//! Workstream: `docs/workstreams/webview-wry-v1/webview-wry-v1.md`.

pub mod wry_backend;

pub mod wry_host;

pub use wry_host::WryWebViewHost;
