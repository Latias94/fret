//! Reusable editor widgets and protocols for node-graph UI.
//!
//! These helpers are UI-only and intended to be used with `NodeGraphPortalHost` to embed regular
//! `fret-ui` controls inside node bodies while preserving undo/redo semantics.

mod chrome;
mod portal_command_policy;
mod portal_command_session;
mod portal_number;
mod portal_text;

pub use portal_command_policy::{
    PortalNumberEditSpec, PortalNumberEditSubmit, PortalTextEditSpec, PortalTextEditSubmit,
};
pub use portal_number::{PortalNumberEditHandler, PortalNumberEditor};
pub use portal_text::{PortalTextEditHandler, PortalTextEditor};
