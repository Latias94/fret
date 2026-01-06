//! Fret UI integration for the node graph editor.
//!
//! This module is behind the default `fret-ui` feature.
//!
//! The initial implementation will provide:
//! - a retained canvas widget (pan/zoom via render transforms),
//! - action-hook driven interactions (drag, connect, selection),
//! - presenter/viewer surfaces for domain-owned node UI.

#![cfg(feature = "fret-ui")]

/// Placeholder type to reserve the public module path.
///
/// The actual widget/presenter surfaces will be introduced incrementally.
#[derive(Debug, Default, Clone)]
pub struct NodeGraphUi;
