use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::action::{
    ActionCx, KeyDownCx, OnKeyDown, OnPinchGesture, OnPointerCancel, OnPointerDown, OnPointerMove,
    OnPointerUp, OnWheel, PinchGestureCx, PointerCancelCx, PointerDownCx, PointerMoveCx,
    PointerUpCx, WheelCx,
};

use crate::core::NodeId;
use crate::ui::NodeGraphSurfaceBinding;
use crate::ui::paint_overrides::NodeGraphPaintOverridesMap;

use super::{
    DeclarativeDiagKeyAction, DeclarativeKeyboardZoomAction, DerivedGeometryCacheState, DragState,
    GridPaintCacheState, MarqueeDragState, NodeDragState, PendingSelectionState, PortalBoundsStore,
    PortalDebugFlags, apply_pan_by_screen_delta, apply_zoom_about_screen_point,
    begin_left_pointer_down_action_host, begin_pan_pointer_down_action_host,
    handle_declarative_diag_key_action_host, handle_declarative_escape_key_action_host,
    handle_declarative_keyboard_zoom_action_host, handle_declarative_pointer_cancel_action_host,
    handle_declarative_pointer_up_action_host, handle_marquee_pointer_move_action_host,
    handle_node_drag_pointer_move_action_host, invalidate_notify_and_redraw_pointer_action_host,
    mouse_buttons_contains, notify_and_redraw_action_host,
    read_left_pointer_down_snapshot_action_host, update_hovered_node_pointer_move_action_host,
    update_view_state_action_host,
};
use fret_canvas::view::{PanZoom2D, wheel_zoom_factor};
use fret_ui::Invalidation;

type HoveredNodeModel = Model<Option<NodeId>>;
pub(super) type HoveredNodeScratch = Model<Vec<NodeId>>;

pub(super) struct KeyHandlerParams {
    pub(super) drag: Model<Option<DragState>>,
    pub(super) marquee_drag: Model<Option<MarqueeDragState>>,
    pub(super) node_drag: Model<Option<NodeDragState>>,
    pub(super) pending_selection: Model<Option<PendingSelectionState>>,
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) portal_bounds_store: Model<PortalBoundsStore>,
    pub(super) portal_debug_flags: Model<PortalDebugFlags>,
    pub(super) diagnostics: super::NodeGraphDiagnosticsConfig,
    pub(super) diag_paint_overrides_value: Arc<NodeGraphPaintOverridesMap>,
    pub(super) diag_paint_overrides_enabled: Model<bool>,
    pub(super) min_zoom: f32,
    pub(super) max_zoom: f32,
}

pub(super) struct PointerDownHandlerParams {
    pub(super) focus_target: GlobalElementId,
    pub(super) pan_button: MouseButton,
    pub(super) drag: Model<Option<DragState>>,
    pub(super) marquee_drag: Model<Option<MarqueeDragState>>,
    pub(super) node_drag: Model<Option<NodeDragState>>,
    pub(super) pending_selection: Model<Option<PendingSelectionState>>,
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) grid_cache: Model<GridPaintCacheState>,
    pub(super) derived_cache: Model<DerivedGeometryCacheState>,
    pub(super) hovered_node: HoveredNodeModel,
    pub(super) hit_scratch: HoveredNodeScratch,
}

pub(super) struct PointerMoveHandlerParams {
    pub(super) drag: Model<Option<DragState>>,
    pub(super) marquee_drag: Model<Option<MarqueeDragState>>,
    pub(super) node_drag: Model<Option<NodeDragState>>,
    pub(super) pending_selection: Model<Option<PendingSelectionState>>,
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) grid_cache: Model<GridPaintCacheState>,
    pub(super) derived_cache: Model<DerivedGeometryCacheState>,
    pub(super) hovered_node: HoveredNodeModel,
    pub(super) hit_scratch: HoveredNodeScratch,
}

pub(super) struct PointerFinishHandlerParams {
    pub(super) pan_button: MouseButton,
    pub(super) drag: Model<Option<DragState>>,
    pub(super) marquee_drag: Model<Option<MarqueeDragState>>,
    pub(super) node_drag: Model<Option<NodeDragState>>,
    pub(super) pending_selection: Model<Option<PendingSelectionState>>,
    pub(super) binding: NodeGraphSurfaceBinding,
}

pub(super) struct WheelHandlerParams {
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) grid_cache: Model<GridPaintCacheState>,
    pub(super) wheel_zoom: super::NodeGraphWheelZoomConfig,
    pub(super) min_zoom: f32,
    pub(super) max_zoom: f32,
}

pub(super) struct PinchHandlerParams {
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) grid_cache: Model<GridPaintCacheState>,
    pub(super) pinch_zoom_speed: f32,
    pub(super) min_zoom: f32,
    pub(super) max_zoom: f32,
}

pub(super) fn build_key_down_capture_handler(params: KeyHandlerParams) -> OnKeyDown {
    Arc::new(move |host, action_cx: ActionCx, key: KeyDownCx| {
        if key.repeat || key.ime_composing {
            return false;
        }

        if key.key == fret_core::KeyCode::Escape {
            let handled = handle_declarative_escape_key_action_host(
                host,
                &params.drag,
                &params.marquee_drag,
                &params.node_drag,
                &params.pending_selection,
            );
            if handled {
                host.request_redraw(action_cx.window);
            }
            return handled;
        }

        if !(key.modifiers.ctrl || key.modifiers.meta) {
            return false;
        }

        if let Some(action) =
            DeclarativeDiagKeyAction::from_key(params.diagnostics.key_actions_enabled, key.key)
        {
            let handled = handle_declarative_diag_key_action_host(
                host,
                action,
                &params.binding,
                &params.portal_bounds_store,
                &params.portal_debug_flags,
                &params.diag_paint_overrides_value,
                &params.diag_paint_overrides_enabled,
            );
            if handled {
                host.request_redraw(action_cx.window);
            }
            return handled;
        }

        let Some(action) = DeclarativeKeyboardZoomAction::from_key(key.key) else {
            return false;
        };
        let handled = handle_declarative_keyboard_zoom_action_host(
            host,
            action,
            &params.binding,
            params.min_zoom,
            params.max_zoom,
        );
        if handled {
            host.request_redraw(action_cx.window);
        }
        handled
    })
}

pub(super) fn build_pointer_down_handler(params: PointerDownHandlerParams) -> OnPointerDown {
    Arc::new(move |host, action_cx: ActionCx, down: PointerDownCx| {
        host.request_focus(params.focus_target);

        let bounds = host.bounds();
        let _ = host.models_mut().update(&params.grid_cache, |state| {
            if state.bounds != bounds {
                state.bounds = bounds;
            }
        });

        if down.button == params.pan_button {
            let handled = begin_pan_pointer_down_action_host(
                host,
                &params.drag,
                &params.marquee_drag,
                &params.node_drag,
                down,
            );
            if handled {
                host.capture_pointer();
                notify_and_redraw_action_host(host, action_cx);
            }
            return handled;
        }

        if down.button != MouseButton::Left {
            return false;
        }

        let snapshot = read_left_pointer_down_snapshot_action_host(
            host,
            &params.binding,
            &params.derived_cache,
            &params.hit_scratch,
            down,
            bounds,
        );
        let outcome = begin_left_pointer_down_action_host(
            host,
            &params.marquee_drag,
            &params.node_drag,
            &params.pending_selection,
            &params.hovered_node,
            down,
            &snapshot,
        );
        if outcome.capture_pointer() {
            host.capture_pointer();
        }
        notify_and_redraw_action_host(host, action_cx);
        true
    })
}

pub(super) fn build_pointer_move_handler(params: PointerMoveHandlerParams) -> OnPointerMove {
    Arc::new(move |host, action_cx: ActionCx, mv: PointerMoveCx| {
        let bounds = host.bounds();
        let _ = host.models_mut().update(&params.grid_cache, |state| {
            if state.bounds != bounds {
                state.bounds = bounds;
            }
        });

        let drag = host
            .models_mut()
            .read(&params.drag, |state| *state)
            .ok()
            .flatten();
        let Some(mut drag) = drag else {
            if let Some(outcome) = handle_node_drag_pointer_move_action_host(
                host,
                &params.node_drag,
                &params.pending_selection,
                &params.hovered_node,
                &params.binding,
                mv,
            ) {
                if outcome.capture_pointer {
                    host.capture_pointer();
                }
                if outcome.needs_layout_redraw {
                    invalidate_notify_and_redraw_pointer_action_host(
                        host,
                        action_cx,
                        Invalidation::Layout,
                    );
                }
                return outcome.needs_layout_redraw;
            }

            if let Some(outcome) = handle_marquee_pointer_move_action_host(
                host,
                &params.marquee_drag,
                &params.hovered_node,
                &params.binding,
                &params.derived_cache,
                mv,
                bounds,
            ) {
                match outcome {
                    super::MarqueePointerMoveOutcome::ReleaseCaptureRedrawOnly => {
                        host.release_pointer_capture();
                        host.request_redraw(action_cx.window);
                    }
                    super::MarqueePointerMoveOutcome::NotifyRedraw => {
                        notify_and_redraw_action_host(host, action_cx);
                    }
                }
                return true;
            }

            let changed = update_hovered_node_pointer_move_action_host(
                host,
                &params.hovered_node,
                &params.binding,
                &params.derived_cache,
                &params.hit_scratch,
                mv,
                bounds,
            );
            if changed {
                invalidate_notify_and_redraw_pointer_action_host(
                    host,
                    action_cx,
                    Invalidation::Paint,
                );
            }
            return changed;
        };

        if !mouse_buttons_contains(mv.buttons, drag.button) {
            return false;
        }

        let dx = mv.position.x.0 - drag.last_pos.x.0;
        let dy = mv.position.y.0 - drag.last_pos.y.0;
        if !dx.is_finite() || !dy.is_finite() {
            return false;
        }

        let updated = update_view_state_action_host(host, &params.binding, |state| {
            apply_pan_by_screen_delta(state, dx, dy);
        });
        if !updated {
            return false;
        }

        drag.last_pos = mv.position;
        let _ = host.models_mut().update(&params.drag, |state| {
            if let Some(state) = state.as_mut() {
                *state = drag;
            }
        });

        invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
        true
    })
}

pub(super) fn build_pointer_up_handler(params: PointerFinishHandlerParams) -> OnPointerUp {
    Arc::new(move |host, action_cx: ActionCx, up: PointerUpCx| {
        handle_declarative_pointer_up_action_host(
            host,
            action_cx,
            up,
            params.pan_button,
            &params.drag,
            &params.marquee_drag,
            &params.node_drag,
            &params.pending_selection,
            &params.binding,
        )
    })
}

pub(super) fn build_pointer_cancel_handler(params: PointerFinishHandlerParams) -> OnPointerCancel {
    Arc::new(move |host, action_cx: ActionCx, cancel: PointerCancelCx| {
        handle_declarative_pointer_cancel_action_host(
            host,
            action_cx,
            cancel,
            &params.drag,
            &params.marquee_drag,
            &params.node_drag,
            &params.pending_selection,
        )
    })
}

pub(super) fn build_wheel_handler(params: WheelHandlerParams) -> OnWheel {
    Arc::new(move |host, action_cx: ActionCx, wheel: WheelCx| {
        if !(wheel.modifiers.ctrl || wheel.modifiers.meta) {
            return false;
        }

        let Some(factor) = wheel_zoom_factor(
            wheel.delta.y.0,
            params.wheel_zoom.base,
            params.wheel_zoom.step,
            params.wheel_zoom.speed,
        ) else {
            return false;
        };

        let bounds = host.bounds();
        let _ = host.models_mut().update(&params.grid_cache, |state| {
            if state.bounds != bounds {
                state.bounds = bounds;
            }
        });
        let updated = update_view_state_action_host(host, &params.binding, |state| {
            let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
            let new_zoom = (zoom * factor).clamp(params.min_zoom, params.max_zoom);
            apply_zoom_about_screen_point(
                state,
                bounds,
                wheel.position,
                new_zoom,
                params.min_zoom,
                params.max_zoom,
            );
        });
        if !updated {
            return false;
        }

        invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
        true
    })
}

pub(super) fn build_pinch_handler(params: PinchHandlerParams) -> OnPinchGesture {
    Arc::new(move |host, action_cx: ActionCx, pinch: PinchGestureCx| {
        if !pinch.delta.is_finite() {
            return false;
        }
        let delta = pinch.delta * params.pinch_zoom_speed;
        if delta.abs() <= 1.0e-9 {
            return false;
        }

        let bounds = host.bounds();
        let _ = host.models_mut().update(&params.grid_cache, |state| {
            if state.bounds != bounds {
                state.bounds = bounds;
            }
        });
        let updated = update_view_state_action_host(host, &params.binding, |state| {
            let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
            let factor = (1.0 + delta).max(1.0e-6);
            let new_zoom = (zoom * factor).clamp(params.min_zoom, params.max_zoom);
            apply_zoom_about_screen_point(
                state,
                bounds,
                pinch.position,
                new_zoom,
                params.min_zoom,
                params.max_zoom,
            );
        });
        if !updated {
            return false;
        }

        invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
        true
    })
}
