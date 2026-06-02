use super::*;

pub(super) fn handle_internal_drag_event<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::Event,
    drag: &fret_core::InternalDragEvent,
    window: AppWindowId,
    allow_multi_window_tear_off: bool,
) {
    match drag.kind {
        fret_core::InternalDragKind::Enter | fret_core::InternalDragKind::Over => {
            handle_internal_drag_hover(cx, event, drag, window, allow_multi_window_tear_off);
        }
        fret_core::InternalDragKind::Drop => {
            handle_internal_drag_drop(cx, event, drag, window);
        }
        fret_core::InternalDragKind::Leave | fret_core::InternalDragKind::Cancel => {
            handle_internal_drag_clear_hover(cx);
        }
    }
}

fn handle_internal_drag_hover<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::Event,
    drag: &fret_core::InternalDragEvent,
    window: AppWindowId,
    allow_multi_window_tear_off: bool,
) {
    let position = cx.pointer_position_window(event).unwrap_or(drag.position);
    let bounds = cx.bounds();
    let theme = cx.theme().snapshot();
    let allow_tear_off = cx
        .app()
        .global::<fret_runtime::PlatformCapabilities>()
        .cloned()
        .unwrap_or_default()
        .ui
        .window_tear_off;
    let (effects, changed, invalidate_layout) = declarative_resolve_internal_drag_hover(
        cx.app(),
        window,
        drag.pointer_id,
        bounds,
        theme,
        position,
        allow_tear_off,
        allow_multi_window_tear_off,
    );
    for effect in effects {
        cx.push_effect(effect);
    }
    if invalidate_layout {
        cx.invalidate_self(fret_ui::Invalidation::Layout);
    }
    if changed {
        cx.invalidate_self(fret_ui::Invalidation::Paint);
        cx.request_redraw();
    }
}

fn handle_internal_drag_drop<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::Event,
    drag: &fret_core::InternalDragEvent,
    window: AppWindowId,
) {
    let position = cx.pointer_position_window(event).unwrap_or(drag.position);
    let bounds = cx.bounds();
    let theme = cx.theme().snapshot();
    let (effects, changed, invalidate_layout, end_drag) = declarative_resolve_internal_drag_drop(
        cx.app(),
        window,
        drag.pointer_id,
        bounds,
        theme,
        position,
        false,
        false,
    );
    for effect in effects {
        cx.push_effect(effect);
    }
    if invalidate_layout {
        cx.invalidate_self(fret_ui::Invalidation::Layout);
    }
    if end_drag
        && cx
            .app()
            .drag(drag.pointer_id)
            .is_some_and(|drag| is_dock_drag_kind(drag.kind))
    {
        cx.app().cancel_drag(drag.pointer_id);
    }
    if changed {
        cx.invalidate_self(fret_ui::Invalidation::Paint);
        cx.request_redraw();
    }
}

fn handle_internal_drag_clear_hover<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
) {
    let hover_cleared = cx
        .app()
        .with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
    if hover_cleared {
        cx.request_redraw();
    }
}
