fn resolve_window_target_from_known_windows(
    current_window: AppWindowId,
    known_windows: &[AppWindowId],
    target: UiWindowTargetV1,
) -> Option<AppWindowId> {
    let first_seen = known_windows
        .iter()
        .copied()
        .min_by_key(|w| w.data().as_ffi());
    let last_seen = known_windows
        .iter()
        .copied()
        .max_by_key(|w| w.data().as_ffi());
    match target {
        UiWindowTargetV1::Current => Some(current_window),
        UiWindowTargetV1::FirstSeen => first_seen,
        UiWindowTargetV1::FirstSeenOther => known_windows
            .iter()
            .copied()
            .filter(|w| *w != current_window)
            .min_by_key(|w| w.data().as_ffi()),
        UiWindowTargetV1::LastSeen => last_seen,
        UiWindowTargetV1::LastSeenOther => known_windows
            .iter()
            .copied()
            .filter(|w| *w != current_window)
            .max_by_key(|w| w.data().as_ffi()),
        UiWindowTargetV1::WindowFfi { window } => {
            let want = AppWindowId::from(KeyData::from_ffi(window));
            known_windows.contains(&want).then_some(want)
        }
    }
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0.max(0.0);
    let ay1 = ay0 + a.size.height.0.max(0.0);

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0.max(0.0);
    let by1 = by0 + b.size.height.0.max(0.0);

    ax1 > bx0 && bx1 > ax0 && ay1 > by0 && by1 > ay0
}

fn center_of_rect(rect: Rect) -> Point {
    let x = rect.origin.x + rect.size.width * 0.5;
    let y = rect.origin.y + rect.size.height * 0.5;
    Point::new(x, y)
}

fn center_of_rect_clamped_to_rect(rect: Rect, clamp: Rect) -> Point {
    if !rects_intersect(rect, clamp) {
        return center_of_rect(rect);
    }

    let rx0 = rect.origin.x.0;
    let ry0 = rect.origin.y.0;
    let rx1 = rx0 + rect.size.width.0.max(0.0);
    let ry1 = ry0 + rect.size.height.0.max(0.0);

    let cx0 = clamp.origin.x.0;
    let cy0 = clamp.origin.y.0;
    let cx1 = cx0 + clamp.size.width.0.max(0.0);
    let cy1 = cy0 + clamp.size.height.0.max(0.0);

    let ix0 = rx0.max(cx0);
    let iy0 = ry0.max(cy0);
    let ix1 = rx1.min(cx1);
    let iy1 = ry1.min(cy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect(rect);
    }

    Point::new(
        fret_core::Px((ix0 + ix1) * 0.5),
        fret_core::Px((iy0 + iy1) * 0.5),
    )
}

fn pointer_position_prefer_intended_hit(
    snapshot: &fret_core::SemanticsSnapshot,
    ui: &UiTree<App>,
    intended: &fret_core::SemanticsNode,
    window_bounds: Rect,
) -> Point {
    let rx0 = intended.bounds.origin.x.0;
    let ry0 = intended.bounds.origin.y.0;
    let rx1 = rx0 + intended.bounds.size.width.0.max(0.0);
    let ry1 = ry0 + intended.bounds.size.height.0.max(0.0);

    let wx0 = window_bounds.origin.x.0;
    let wy0 = window_bounds.origin.y.0;
    let wx1 = wx0 + window_bounds.size.width.0.max(0.0);
    let wy1 = wy0 + window_bounds.size.height.0.max(0.0);

    let ix0 = rx0.max(wx0);
    let iy0 = ry0.max(wy0);
    let ix1 = rx1.min(wx1);
    let iy1 = ry1.min(wy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect_clamped_to_rect(intended.bounds, window_bounds);
    }

    let w = (ix1 - ix0).max(0.0);
    let h = (iy1 - iy0).max(0.0);
    let pad_x = 8.0f32.min(w * 0.5);
    let pad_y = 8.0f32.min(h * 0.5);

    let x_mid = (ix0 + ix1) * 0.5;
    let y_mid = (iy0 + iy1) * 0.5;

    let candidates = [
        Point::new(fret_core::Px(x_mid), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy1 - pad_y)),
    ];

    let index = SemanticsIndex::new(snapshot);
    let intended_id = intended.id.data().as_ffi();
    for pos in candidates {
        let Some(hit) = pick_semantics_node_at(snapshot, ui, pos) else {
            continue;
        };
        let hit_id = hit.id.data().as_ffi();
        let controls_intended = hit
            .controls
            .iter()
            .any(|id| id.data().as_ffi() == intended_id);
        if controls_intended || index.is_descendant_of_or_self(hit_id, intended_id) {
            return pos;
        }
    }

    candidates[0]
}

fn wheel_position_prefer_intended_hit(
    snapshot: &fret_core::SemanticsSnapshot,
    ui: &UiTree<App>,
    intended: &fret_core::SemanticsNode,
    container_bounds: Rect,
    window_bounds: Rect,
) -> Point {
    let cx0 = window_bounds.origin.x.0;
    let cy0 = window_bounds.origin.y.0;
    let cx1 = cx0 + window_bounds.size.width.0.max(0.0);
    let cy1 = cy0 + window_bounds.size.height.0.max(0.0);

    let bx0 = container_bounds.origin.x.0;
    let by0 = container_bounds.origin.y.0;
    let bx1 = bx0 + container_bounds.size.width.0.max(0.0);
    let by1 = by0 + container_bounds.size.height.0.max(0.0);

    let ix0 = bx0.max(cx0);
    let iy0 = by0.max(cy0);
    let ix1 = bx1.min(cx1);
    let iy1 = by1.min(cy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect(container_bounds);
    }

    let w = (ix1 - ix0).max(0.0);
    let h = (iy1 - iy0).max(0.0);
    let pad_x = 8.0f32.min(w * 0.5);
    let pad_y = 8.0f32.min(h * 0.5);

    let x_mid = (ix0 + ix1) * 0.5;
    let y_mid = (iy0 + iy1) * 0.5;

    let candidates = [
        Point::new(fret_core::Px(x_mid), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy1 - pad_y)),
    ];

    for pos in candidates {
        if let Some(hit) = pick_semantics_node_at(snapshot, ui, pos)
            && hit.id.data().as_ffi() == intended.id.data().as_ffi()
        {
            return pos;
        }
    }

    candidates[0]
}

fn parse_semantics_numeric_value(value: &str) -> Option<f32> {
    let s = value.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(raw) = s.strip_suffix('%') {
        return raw.trim().parse::<f32>().ok();
    }
    if let Ok(v) = s.parse::<f32>() {
        return Some(v);
    }

    // Best-effort: extract the first float-ish token from the string.
    let mut token = String::new();
    let mut started = false;
    for ch in s.chars() {
        let keep = ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+');
        if keep {
            token.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    if token.is_empty() {
        return None;
    }
    token.parse::<f32>().ok()
}
