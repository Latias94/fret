//! Reusable editor widgets and protocols for node-graph UI.
//!
//! These helpers are UI-only and intended to be used with `NodeGraphPortalHost` to embed regular
//! `fret-ui` controls inside node bodies while preserving undo/redo semantics.

mod chrome;
mod portal_number;
mod portal_text;

pub use chrome::{PortalSmallButtonUi, render_pressable_small_button, render_small_button};
pub use portal_number::{
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
};
pub use portal_text::{
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextEditor,
    PortalTextEditorUi,
};
