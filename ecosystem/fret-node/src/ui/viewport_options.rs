//! Public viewport option surface for store-backed controller/binding helpers.
//!
//! These options describe only the parameters that the public store-first viewport helpers
//! actually consume. Retained queue transport may still keep richer motion controls internally, but
//! those controls are not part of the app-facing authoring contract.

/// Public fit-view options for store-backed controller/binding helpers.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NodeGraphFitViewOptions {
    /// Include hidden nodes when computing the fit target.
    pub include_hidden_nodes: bool,
    /// Optional per-call zoom clamp override.
    pub min_zoom: Option<f32>,
    /// Optional per-call zoom clamp override.
    pub max_zoom: Option<f32>,
    /// Optional per-call padding override.
    pub padding: Option<f32>,
}

/// Public set-viewport options for store-backed controller/binding helpers.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NodeGraphSetViewportOptions {
    /// Optional per-call zoom clamp override.
    pub min_zoom: Option<f32>,
    /// Optional per-call zoom clamp override.
    pub max_zoom: Option<f32>,
}
