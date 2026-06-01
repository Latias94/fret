use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, DockNodeId, PanelKey, Px, Size};
use fret_ui::{ThemeSnapshot, UiHost};

use super::super::host_frame::DockSpaceLayoutSnapshot;
use super::super::manager::DockManager;
use super::super::paint::{
    ComplexDropOverlayPaintInput, FloatingChromePaintInput, SplitHandlePaintInput,
    TabChromePaintInput, TabDetailPaintInput, ViewportSurfacePaintInput,
    complex_drop_overlay_paint_inputs, split_handle_paint_inputs, tab_chrome_paint_inputs,
    tab_detail_paint_inputs, viewport_surface_paint_inputs,
};
use super::super::types::{DockDragGhostSnapshot, DockDropTarget};
use super::drag_preview::dock_drag_ghost_for_window;
use super::floating::{
    declarative_floating_hover_for_window, declarative_pressed_floating_close_for_window,
    floating_chrome_paint_inputs,
};
use super::frame::DockSpaceElementFrame;
use super::overflow::declarative_tab_overflow_menu_for_window;
use super::tab_metrics::{declarative_tab_scroll_for_frame, declarative_tab_widths_for_layout};
use super::tab_paint_state::declarative_tab_hover_for_window;

pub(super) struct DeclarativeFramePaintState {
    hover: Option<DockDropTarget>,
    tab_chrome_inputs: Vec<TabChromePaintInput>,
    tab_detail_inputs: Vec<TabDetailPaintInput>,
    tab_widths: HashMap<DockNodeId, Arc<[Px]>>,
    pub(super) tab_scroll: HashMap<DockNodeId, Px>,
    complex_drop_overlay_inputs: Vec<ComplexDropOverlayPaintInput>,
    floating_chrome_inputs: Vec<FloatingChromePaintInput>,
    dock_drag_ghost: Option<DockDragGhostSnapshot>,
    split_handle_inputs: Vec<SplitHandlePaintInput>,
    viewport_surface_inputs: Vec<ViewportSurfacePaintInput>,
}

impl DeclarativeFramePaintState {
    pub(super) fn into_frame(
        self,
        snapshot: &DockSpaceLayoutSnapshot,
        panel_last_sizes: HashMap<PanelKey, Size>,
    ) -> DockSpaceElementFrame {
        DockSpaceElementFrame::from_snapshot(
            snapshot,
            panel_last_sizes,
            self.hover,
            self.tab_chrome_inputs,
            self.tab_detail_inputs,
            self.tab_widths,
            self.tab_scroll,
            self.complex_drop_overlay_inputs,
            self.floating_chrome_inputs,
            self.dock_drag_ghost,
            self.split_handle_inputs,
            self.viewport_surface_inputs,
        )
    }
}

pub(super) fn prepare_declarative_frame_paint_state<H: UiHost>(
    app: &H,
    window: AppWindowId,
    theme: ThemeSnapshot,
    snapshot: &DockSpaceLayoutSnapshot,
    settings: fret_runtime::DockingInteractionSettings,
    sync_tab_scroll: bool,
) -> DeclarativeFramePaintState {
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        sync_tab_scroll,
    );
    let tab_overflow_menu = declarative_tab_overflow_menu_for_window(app, window);
    let tab_hover = declarative_tab_hover_for_window(app, window);
    let floating_hover = declarative_floating_hover_for_window(app, window);
    let pressed_floating_close = declarative_pressed_floating_close_for_window(app, window);
    let floating_chrome_inputs =
        floating_chrome_paint_inputs(snapshot, pressed_floating_close, floating_hover);
    let dock_drag_ghost = dock_drag_ghost_for_window(app, window);
    let (
        hover,
        tab_chrome_inputs,
        tab_detail_inputs,
        complex_drop_overlay_inputs,
        split_handle_inputs,
        viewport_surface_inputs,
    ) = app
        .global::<DockManager>()
        .map(|dock| {
            (
                dock.hover.clone(),
                tab_chrome_paint_inputs(
                    &dock.graph,
                    &snapshot.layout_all,
                    &tab_widths,
                    &tab_scroll,
                    tab_hover.tab,
                ),
                tab_detail_paint_inputs(
                    &dock.graph,
                    &snapshot.layout_all,
                    &tab_widths,
                    &tab_scroll,
                    tab_hover.tab,
                    tab_hover.tab_close,
                    tab_hover.overflow_button,
                    None,
                    tab_overflow_menu.clone(),
                ),
                complex_drop_overlay_paint_inputs(
                    theme,
                    dock.hover.clone(),
                    window,
                    &dock.graph,
                    &snapshot.layout_all,
                    settings.split_handle_gap,
                    settings.split_handle_hit_thickness,
                    &tab_scroll,
                    &tab_widths,
                ),
                split_handle_paint_inputs(&dock.graph, &snapshot.layout_all),
                viewport_surface_paint_inputs(dock, window, &snapshot.layout_all),
            )
        })
        .unwrap_or_default();

    DeclarativeFramePaintState {
        hover,
        tab_chrome_inputs,
        tab_detail_inputs,
        tab_widths,
        tab_scroll,
        complex_drop_overlay_inputs,
        floating_chrome_inputs,
        dock_drag_ghost,
        split_handle_inputs,
        viewport_surface_inputs,
    }
}
