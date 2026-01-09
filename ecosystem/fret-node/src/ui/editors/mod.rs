//! Reusable editor widgets and protocols for node-graph UI.
//!
//! These helpers are UI-only and intended to be used with `NodeGraphPortalHost` to embed regular
//! `fret-ui` controls inside node bodies while preserving undo/redo semantics.

mod portal_text;

pub use portal_text::{
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextEditor,
    PortalTextEditorUi,
};
