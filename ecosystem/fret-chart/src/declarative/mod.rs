//! Declarative chart canvas panel.
//!
//! This is an incremental migration surface: it renders `delinea::ChartEngine` marks via the
//! declarative `Canvas` element while wiring input through the `fret-ui-kit` canvas tool router.

mod legend_overlay;
mod panel;
mod tooltip_overlay;

pub use panel::*;
