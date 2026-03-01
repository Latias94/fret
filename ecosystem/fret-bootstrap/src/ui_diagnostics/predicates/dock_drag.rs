#[derive(Clone, Copy, Debug)]
struct DockDragRuntimeState {
    dragging: bool,
    cross_window_hover: bool,
    source_window: AppWindowId,
    current_window: AppWindowId,
    moving_window: Option<AppWindowId>,
    window_under_moving_window: Option<AppWindowId>,
    window_under_moving_window_source: fret_runtime::WindowUnderCursorSource,
    transparent_payload_applied: bool,
    transparent_payload_mouse_passthrough_applied: bool,
    window_under_cursor_source: fret_runtime::WindowUnderCursorSource,
}

fn dock_drag_pointer_id_best_effort(
    app: &fret_app::App,
    known_windows: &[AppWindowId],
) -> Option<PointerId> {
    if let Some(pointer_id) = app.find_drag_pointer_id(|d| {
        (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && d.dragging
    }) {
        return Some(pointer_id);
    }

    let store = app.global::<fret_runtime::WindowInteractionDiagnosticsStore>()?;
    for window in known_windows.iter().rev().copied() {
        let docking = store.docking_latest_for_window(window)?;
        if let Some(drag) = docking.dock_drag
            && drag.dragging
        {
            // `docking_latest_for_window` is intentionally stable across frames, which makes it
            // useful for debugging but also means it can be stale. Only treat it as authoritative
            // when the drag session is still present in the live `App` drag registry.
            if app.drag(drag.pointer_id).is_some() {
                return Some(drag.pointer_id);
            }
        }
    }

    None
}

fn dock_drag_runtime_state(
    app: &fret_app::App,
    known_windows: &[AppWindowId],
) -> Option<DockDragRuntimeState> {
    if let Some(pointer_id) = dock_drag_pointer_id_best_effort(app, known_windows)
        && let Some(drag) = app.drag(pointer_id)
    {
        return Some(DockDragRuntimeState {
            dragging: drag.dragging,
            cross_window_hover: drag.cross_window_hover,
            source_window: drag.source_window,
            current_window: drag.current_window,
            moving_window: drag.moving_window,
            window_under_moving_window: drag.window_under_moving_window,
            window_under_moving_window_source: drag.window_under_moving_window_source,
            transparent_payload_applied: drag.transparent_payload_applied,
            transparent_payload_mouse_passthrough_applied: drag
                .transparent_payload_mouse_passthrough_applied,
            window_under_cursor_source: drag.window_under_cursor_source,
        });
    }

    // If the drag session cannot be found in `App`, treat it as inactive. The per-window docking
    // diagnostics store may retain stale "latest" snapshots across frames (by design), which is
    // useful for debugging but unsuitable as a source of truth for scripted gates.
    None
}
