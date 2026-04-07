use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

use crate::ui::NodeGraphSurfaceBinding;

use super::{
    KeyHandlerParams, PaintOnlySurfaceModels, PinchHandlerParams, PointerDownHandlerParams,
    PointerFinishHandlerParams, PointerMoveHandlerParams, PreparedPaintOnlySurfaceFrame,
    SurfaceRegionChildrenParams, WheelHandlerParams, build_key_down_capture_handler,
    build_pinch_handler, build_pointer_cancel_handler, build_pointer_down_handler,
    build_pointer_move_handler, build_pointer_up_handler, build_surface_region_children,
    build_wheel_handler,
};

pub(super) struct SurfaceShellParams {
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) pointer_region: PointerRegionProps,
    pub(super) canvas: crate::ui::declarative::paint_only::CanvasProps,
    pub(super) measured_geometry_present: bool,
    pub(super) portal_hosting: super::NodeGraphVisibleSubsetPortalConfig,
    pub(super) cull_margin_screen_px: f32,
    pub(super) pan_button: fret_core::MouseButton,
    pub(super) min_zoom: f32,
    pub(super) max_zoom: f32,
    pub(super) wheel_zoom: super::NodeGraphWheelZoomConfig,
    pub(super) pinch_zoom_speed: f32,
    pub(super) surface_models: PaintOnlySurfaceModels,
    pub(super) prepared_frame: PreparedPaintOnlySurfaceFrame,
}

pub(super) fn build_surface_shell<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    element: fret_ui::GlobalElementId,
    params: SurfaceShellParams,
) -> Vec<AnyElement> {
    let SurfaceShellParams {
        binding,
        pointer_region,
        canvas,
        measured_geometry_present,
        portal_hosting,
        cull_margin_screen_px,
        pan_button,
        min_zoom,
        max_zoom,
        wheel_zoom,
        pinch_zoom_speed,
        surface_models,
        prepared_frame,
    } = params;
    let PaintOnlySurfaceModels {
        drag,
        marquee_drag,
        node_drag,
        pending_selection,
        hovered_node,
        hit_scratch,
        diag_paint_overrides: _,
        diag_paint_overrides_enabled,
        grid_cache,
        derived_cache,
        edges_cache: _,
        nodes_cache: _,
        portal_bounds_store,
        portal_measured_geometry_state,
        portal_debug_flags,
        hover_anchor_store,
        authoritative_surface_boundary: _,
    } = surface_models;

    if let Some(bounds) = cx.last_bounds_for_element(element) {
        let _ = cx.app.models_mut().update(&grid_cache, |state| {
            if state.bounds != bounds {
                state.bounds = bounds;
            }
        });
    }

    let on_key_down_capture = build_key_down_capture_handler(KeyHandlerParams {
        drag: drag.clone(),
        marquee_drag: marquee_drag.clone(),
        node_drag: node_drag.clone(),
        pending_selection: pending_selection.clone(),
        binding: binding.clone(),
        portal_bounds_store: portal_bounds_store.clone(),
        portal_debug_flags: portal_debug_flags.clone(),
        diagnostics: prepared_frame.diagnostics,
        diag_paint_overrides_value: prepared_frame.diag_paint_overrides_value.clone(),
        diag_paint_overrides_enabled: diag_paint_overrides_enabled.clone(),
        min_zoom,
        max_zoom,
    });
    cx.key_on_key_down_capture_for(element, on_key_down_capture);

    let on_pointer_down = build_pointer_down_handler(PointerDownHandlerParams {
        focus_target: element,
        pan_button,
        drag: drag.clone(),
        marquee_drag: marquee_drag.clone(),
        node_drag: node_drag.clone(),
        pending_selection: pending_selection.clone(),
        binding: binding.clone(),
        grid_cache: grid_cache.clone(),
        derived_cache: derived_cache.clone(),
        hovered_node: hovered_node.clone(),
        hit_scratch: hit_scratch.clone(),
    });

    let on_pointer_move = build_pointer_move_handler(PointerMoveHandlerParams {
        drag: drag.clone(),
        marquee_drag: marquee_drag.clone(),
        node_drag: node_drag.clone(),
        pending_selection: pending_selection.clone(),
        binding: binding.clone(),
        grid_cache: grid_cache.clone(),
        derived_cache: derived_cache.clone(),
        hovered_node: hovered_node.clone(),
        hit_scratch: hit_scratch.clone(),
    });

    let on_pointer_up = build_pointer_up_handler(PointerFinishHandlerParams {
        pan_button,
        drag: drag.clone(),
        marquee_drag: marquee_drag.clone(),
        node_drag: node_drag.clone(),
        pending_selection: pending_selection.clone(),
        binding: binding.clone(),
    });

    let on_pointer_cancel = build_pointer_cancel_handler(PointerFinishHandlerParams {
        pan_button,
        drag: drag.clone(),
        marquee_drag: marquee_drag.clone(),
        node_drag: node_drag.clone(),
        pending_selection: pending_selection.clone(),
        binding: binding.clone(),
    });

    let on_wheel = build_wheel_handler(WheelHandlerParams {
        binding: binding.clone(),
        grid_cache: grid_cache.clone(),
        wheel_zoom,
        min_zoom,
        max_zoom,
    });

    let on_pinch = build_pinch_handler(PinchHandlerParams {
        binding: binding.clone(),
        grid_cache: grid_cache.clone(),
        pinch_zoom_speed,
        min_zoom,
        max_zoom,
    });

    vec![cx.pointer_region(pointer_region, move |cx| {
        cx.pointer_region_on_pointer_down(on_pointer_down);
        cx.pointer_region_on_pointer_move(on_pointer_move);
        cx.pointer_region_on_pointer_up(on_pointer_up);
        cx.pointer_region_on_pointer_cancel(on_pointer_cancel);
        cx.pointer_region_on_wheel(on_wheel);
        cx.pointer_region_on_pinch_gesture(on_pinch);

        build_surface_region_children(
            cx,
            SurfaceRegionChildrenParams {
                canvas,
                binding: binding.clone(),
                hovered_node_model: hovered_node.clone(),
                node_drag_model: node_drag.clone(),
                marquee_drag_model: marquee_drag.clone(),
                hover_anchor_store: hover_anchor_store.clone(),
                portal_bounds_store: portal_bounds_store.clone(),
                portal_measured_geometry_state: portal_measured_geometry_state.clone(),
                measured_geometry_present,
                portal_hosting,
                portals_disabled: prepared_frame.portals_disabled,
                cull_margin_screen_px,
                min_zoom,
                max_zoom,
                diagnostics: prepared_frame.diagnostics,
                panning: prepared_frame.panning,
                marquee_active: prepared_frame.marquee_active,
                node_dragging: prepared_frame.node_dragging,
                view_for_paint: prepared_frame.view_for_paint,
                grid_bounds: prepared_frame.grid_cache_value.bounds,
                grid_ops: prepared_frame.grid_cache_value.ops.clone(),
                node_draws: prepared_frame.nodes_cache_value.draws.clone(),
                edge_draws: prepared_frame.edges_cache_value.draws.clone(),
                geom_for_paint: prepared_frame.derived_cache_value.geom.clone(),
                style_tokens: prepared_frame.style_tokens.clone(),
                theme: prepared_frame.theme.clone(),
                hovered_node_value: prepared_frame.hovered_node_value,
                selected_nodes: prepared_frame.effective_selected_nodes.clone(),
                marquee_value: prepared_frame.marquee_value.clone(),
                node_drag_value: prepared_frame.node_drag_value.clone(),
                paint_overrides_ref: prepared_frame.paint_overrides_ref.clone(),
            },
        )
    })]
}
