//! Interop helpers for embedding "foreign UI" as isolated surfaces.
//!
//! These helpers are intentionally ecosystem-level and designed to keep kernel contracts stable:
//! - foreign UI is embedded as a render target surface + optional input forwarding,
//! - focus/IME/a11y are not shared across runtimes (isolation boundary).

pub mod embedded_viewport;
