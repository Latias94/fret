use super::*;

#[derive(Clone)]
pub(super) struct TouchPointerDownOutsideCandidate {
    pub(super) layer_id: UiLayerId,
    pub(super) root: NodeId,
    pub(super) consume: bool,
    pub(super) down_event: Event,
    pub(super) start_pos: Point,
    pub(super) moved: bool,
}

pub(super) fn pointer_position(pe: &PointerEvent) -> Point {
    match pe {
        PointerEvent::Move { position, .. }
        | PointerEvent::Down { position, .. }
        | PointerEvent::Up { position, .. }
        | PointerEvent::Wheel { position, .. }
        | PointerEvent::PinchGesture { position, .. } => *position,
    }
}

pub(super) fn rect_aabb_transformed(rect: Rect, t: Transform2D) -> Rect {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let p00 = t.apply_point(Point::new(Px(x0), Px(y0)));
    let p10 = t.apply_point(Point::new(Px(x1), Px(y0)));
    let p01 = t.apply_point(Point::new(Px(x0), Px(y1)));
    let p11 = t.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
    let max_x = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
    let min_y = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
    let max_y = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    )
}

pub(super) fn event_position(event: &Event) -> Option<Point> {
    match event {
        Event::Pointer(pe) => Some(pointer_position(pe)),
        Event::PointerCancel(e) => e.position,
        Event::ExternalDrag(e) => Some(e.position),
        Event::InternalDrag(e) => Some(e.position),
        _ => None,
    }
}

#[cfg(test)]
pub(super) fn event_allows_hit_test_path_cache_reuse(event: &Event) -> bool {
    matches!(
        event,
        Event::Pointer(PointerEvent::Move { .. })
            | Event::Pointer(PointerEvent::Wheel { .. })
            | Event::Pointer(PointerEvent::PinchGesture { .. })
            | Event::ExternalDrag(_)
            | Event::InternalDrag(_)
    )
}

pub(super) fn pointer_type_supports_hover(pointer_type: fret_core::PointerType) -> bool {
    // Hover is a cursor-driven affordance (Mouse/Pen). Touch pointers must not perturb hover state,
    // otherwise multi-pointer input can cause spurious hover exits while a mouse cursor remains in
    // place.
    //
    // `Unknown` is treated as hover-capable to keep desktop backends usable when pointer
    // classification is incomplete.
    matches!(
        pointer_type,
        fret_core::PointerType::Mouse
            | fret_core::PointerType::Pen
            | fret_core::PointerType::Unknown
    )
}

pub(super) fn interactive_resize_stable_frames_required() -> u8 {
    crate::runtime_config::ui_runtime_config().interactive_resize_stable_frames_required
}

pub(super) fn text_wrap_width_bucket_px() -> u8 {
    crate::runtime_config::ui_runtime_config().text_wrap_width_bucket_px
}

pub(super) fn text_wrap_width_small_step_bucket_px() -> u8 {
    crate::runtime_config::ui_runtime_config().text_wrap_width_small_step_bucket_px
}

pub(super) fn text_wrap_width_small_step_max_dw_px() -> u8 {
    crate::runtime_config::ui_runtime_config().text_wrap_width_small_step_max_dw_px
}
