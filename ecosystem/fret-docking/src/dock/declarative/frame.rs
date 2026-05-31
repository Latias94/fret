use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{PanelKey, Rect, Size};

use super::super::host_frame::DockSpaceLayoutSnapshot;
use super::super::paint::{
    ComplexDropOverlayPaintInput, FloatingChromePaintInput, SplitHandlePaintInput,
    TabChromePaintInput, TabDetailPaintInput, ViewportSurfacePaintInput,
};
use super::super::types::{DockDragGhostSnapshot, DockDropHints, DockDropTarget};

#[derive(Debug, Clone)]
pub(super) struct DockSpaceElementFrame {
    pub(super) paint_panel_bounds: Vec<(PanelKey, Rect)>,
    pub(super) panel_last_sizes: HashMap<PanelKey, Size>,
    pub(super) layout_all: HashMap<fret_core::DockNodeId, Rect>,
    pub(super) hover: Option<DockDropTarget>,
    pub(super) drop_hints: Option<DockDropHints>,
    pub(super) tab_chrome_inputs: Vec<TabChromePaintInput>,
    pub(super) tab_detail_inputs: Vec<TabDetailPaintInput>,
    pub(super) tab_widths: HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    pub(super) tab_scroll: HashMap<fret_core::DockNodeId, fret_core::Px>,
    pub(super) complex_drop_overlay_inputs: Vec<ComplexDropOverlayPaintInput>,
    pub(super) floating_chrome_inputs: Vec<FloatingChromePaintInput>,
    pub(super) floating_chrome_nodes: Vec<fret_core::DockNodeId>,
    pub(super) dock_drag_ghost: Option<DockDragGhostSnapshot>,
    pub(super) split_handle_inputs: Vec<SplitHandlePaintInput>,
    pub(super) viewport_surface_inputs: Vec<ViewportSurfacePaintInput>,
    pub(super) split_handle_gap: fret_core::Px,
    pub(super) split_handle_hit_thickness: fret_core::Px,
}

impl DockSpaceElementFrame {
    pub(super) fn empty(panel_last_sizes: HashMap<PanelKey, Size>) -> Self {
        Self {
            paint_panel_bounds: Vec::new(),
            panel_last_sizes,
            layout_all: HashMap::new(),
            hover: None,
            drop_hints: None,
            tab_chrome_inputs: Vec::new(),
            tab_detail_inputs: Vec::new(),
            tab_widths: HashMap::new(),
            tab_scroll: HashMap::new(),
            complex_drop_overlay_inputs: Vec::new(),
            floating_chrome_inputs: Vec::new(),
            floating_chrome_nodes: Vec::new(),
            dock_drag_ghost: None,
            split_handle_inputs: Vec::new(),
            viewport_surface_inputs: Vec::new(),
            split_handle_gap: fret_core::Px(0.0),
            split_handle_hit_thickness: fret_core::Px(0.0),
        }
    }

    pub(super) fn from_snapshot(
        snapshot: &DockSpaceLayoutSnapshot,
        panel_last_sizes: HashMap<PanelKey, Size>,
        hover: Option<DockDropTarget>,
        tab_chrome_inputs: Vec<TabChromePaintInput>,
        tab_detail_inputs: Vec<TabDetailPaintInput>,
        tab_widths: HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
        tab_scroll: HashMap<fret_core::DockNodeId, fret_core::Px>,
        complex_drop_overlay_inputs: Vec<ComplexDropOverlayPaintInput>,
        floating_chrome_inputs: Vec<FloatingChromePaintInput>,
        dock_drag_ghost: Option<DockDragGhostSnapshot>,
        split_handle_inputs: Vec<SplitHandlePaintInput>,
        viewport_surface_inputs: Vec<ViewportSurfacePaintInput>,
    ) -> Self {
        Self {
            paint_panel_bounds: snapshot.paint_panel_bounds.clone(),
            panel_last_sizes,
            layout_all: snapshot.layout_all.clone(),
            drop_hints: super::drop_hints_from_hover(hover.as_ref()),
            hover,
            tab_chrome_inputs,
            tab_detail_inputs,
            tab_widths,
            tab_scroll,
            complex_drop_overlay_inputs,
            floating_chrome_nodes: snapshot
                .floating_layouts
                .iter()
                .map(|floating| floating.floating.floating)
                .collect(),
            floating_chrome_inputs,
            dock_drag_ghost,
            split_handle_inputs,
            viewport_surface_inputs,
            split_handle_gap: snapshot.split_handle_gap,
            split_handle_hit_thickness: snapshot.split_handle_hit_thickness,
        }
    }
}
