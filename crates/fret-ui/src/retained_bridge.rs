//! Unstable retained-widget bridge for policy-heavy UI (e.g. docking migration).
//!
//! This module is intentionally feature-gated (`unstable-retained-bridge`) and is **not** part of
//! the stable `fret-ui` runtime contract surface (ADR 0066).

use crate::{UiHost, UiTree};
use fret_core::NodeId;

pub use crate::resizable_panel_group::{ResizablePanelGroupLayout, ResizablePanelGroupStyle};
pub use crate::resize_handle::ResizeHandle;
pub use crate::text_input::{BoundTextInput, TextInput};
pub use crate::widget::{
    CommandCx, EventCx, Invalidation, LayoutCx, MeasureCx, PaintCx, SemanticsCx, Widget,
};

/// Extension trait that exposes a feature-gated node creation API for retained widgets.
pub trait UiTreeRetainedExt<H: UiHost> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId;
}

impl<H: UiHost> UiTreeRetainedExt<H> for UiTree<H> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        self.create_node(widget)
    }
}

/// Unstable mechanism helpers for splitter / panel-group sizing.
pub mod resizable_panel_group {
    use fret_core::{Axis, Point, Px, Rect};

    use crate::resizable_panel_group::{
        ResizablePanelGroupLayout, apply_handle_delta, compute_resizable_panel_group_layout,
        fractions_from_sizes,
    };

    pub fn compute_layout(
        axis: Axis,
        bounds: Rect,
        children_len: usize,
        fractions: &[f32],
        gap: Px,
        hit_thickness: Px,
        min_px: &[Px],
    ) -> ResizablePanelGroupLayout {
        compute_resizable_panel_group_layout(
            axis,
            bounds,
            children_len,
            fractions.to_vec(),
            gap,
            hit_thickness,
            min_px,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn drag_update_fractions(
        axis: Axis,
        bounds: Rect,
        children_len: usize,
        fractions: &[f32],
        handle_ix: usize,
        gap: Px,
        hit_thickness: Px,
        min_px: &[Px],
        grab_offset: f32,
        position: Point,
    ) -> Option<Vec<f32>> {
        if children_len < 2 || handle_ix + 1 >= children_len {
            return None;
        }

        let layout = compute_layout(
            axis,
            bounds,
            children_len,
            fractions,
            gap,
            hit_thickness,
            min_px,
        );
        let old_center = *layout.handle_centers.get(handle_ix)?;

        let axis_pos = match axis {
            Axis::Horizontal => position.x.0,
            Axis::Vertical => position.y.0,
        };

        let desired_center = axis_pos - grab_offset;
        let desired_delta = desired_center - old_center;
        if !desired_delta.is_finite() {
            return None;
        }

        let mut sizes = layout.sizes.clone();
        let actual = apply_handle_delta(handle_ix, desired_delta, &mut sizes, &layout.mins);
        if actual.abs() <= 1.0e-6 {
            return None;
        }
        Some(fractions_from_sizes(&sizes, layout.avail))
    }
}
