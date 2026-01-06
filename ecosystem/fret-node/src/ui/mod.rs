//! Fret UI integration for the node graph editor.
//!
//! This module is behind the default `fret-ui` feature.

#![cfg(feature = "fret-ui")]

pub mod canvas;
pub mod presenter;
pub mod style;

pub use canvas::NodeGraphCanvas;
pub use presenter::{DefaultNodeGraphPresenter, NodeGraphPresenter};
pub use style::NodeGraphStyle;
