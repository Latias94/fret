use super::*;

pub(super) fn handle_pointer_move_viewport_capture<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    window: AppWindowId,
    position: fret_core::Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    pointer_id: fret_core::PointerId,
    pointer_type: fret_core::PointerType,
) -> bool {
    let viewport_capture = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| service.viewport_capture(window, pointer_id),
    );
    if let Some(mut capture) = viewport_capture {
        capture.last = position;
        if !capture.moved && capture.button == fret_core::MouseButton::Right {
            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let dx = position.x.0 - capture.start.x.0;
            let dy = position.y.0 - capture.start.y.0;
            let dist2 = dx * dx + dy * dy;
            let threshold = settings.viewport_context_menu_drag_threshold.0.max(0.0);
            capture.moved = dist2 >= threshold * threshold;
        }
        let input = viewport_input_from_hit_clamped(
            window,
            capture.hit.clone(),
            declarative_pixels_per_point(cx.app(), window),
            pointer_id,
            pointer_type,
            position,
            fret_core::ViewportInputKind::PointerMove { buttons, modifiers },
        );
        cx.push_effect(Effect::ViewportInput(input));
        cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service, _app| service.begin_viewport_capture(window, capture),
        );
        cx.request_redraw();
        cx.stop_propagation();
        return true;
    }

    cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| service.has_viewport_capture_for_window(window),
    )
}
