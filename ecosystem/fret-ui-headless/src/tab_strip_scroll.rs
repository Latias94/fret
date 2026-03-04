use fret_core::Px;

/// Ensure a horizontal child range is visible inside a scroll viewport.
///
/// All coordinates are in the same **content** coordinate space:
/// - `current_scroll_x` and `max_scroll_x` are scroll offsets in content space
/// - `view_width` is the viewport width
/// - `child_start_x`/`child_end_x` are the child's bounds in content space
///
/// `margin` reserves extra space on both edges (useful for keyboard-driven activation).
pub fn ensure_range_visible_x(
    current_scroll_x: Px,
    max_scroll_x: Px,
    view_width: Px,
    child_start_x: Px,
    child_end_x: Px,
    margin: Px,
) -> Px {
    if view_width.0 <= 0.0 || max_scroll_x.0 <= 0.0 {
        return Px(0.0);
    }

    let mut next = Px(current_scroll_x.0.clamp(0.0, max_scroll_x.0));
    let child_start = child_start_x.0;
    let child_end = child_end_x.0.max(child_start);

    let margin = margin.0.max(0.0);
    let view_start = next.0 + margin;
    let view_end = next.0 + view_width.0 - margin;

    if child_start < view_start {
        next = Px((child_start - margin).max(0.0));
    } else if child_end > view_end {
        next = Px((child_end - (view_width.0 - margin)).max(0.0));
    }

    Px(next.0.clamp(0.0, max_scroll_x.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_range_visible_x_returns_zero_when_view_is_non_positive() {
        assert_eq!(
            ensure_range_visible_x(Px(10.0), Px(100.0), Px(0.0), Px(5.0), Px(15.0), Px(0.0)),
            Px(0.0)
        );
    }

    #[test]
    fn ensure_range_visible_x_scrolls_left_when_child_is_left_clipped() {
        let next =
            ensure_range_visible_x(Px(50.0), Px(300.0), Px(100.0), Px(20.0), Px(40.0), Px(0.0));
        assert_eq!(next, Px(20.0));
    }

    #[test]
    fn ensure_range_visible_x_scrolls_right_when_child_is_right_clipped() {
        let next =
            ensure_range_visible_x(Px(0.0), Px(300.0), Px(100.0), Px(120.0), Px(160.0), Px(0.0));
        assert_eq!(next, Px(60.0));
    }

    #[test]
    fn ensure_range_visible_x_respects_margin() {
        // Child ends at 100; with margin=10 and view=100, right edge is 90 => must scroll.
        let next =
            ensure_range_visible_x(Px(0.0), Px(300.0), Px(100.0), Px(70.0), Px(100.0), Px(10.0));
        assert_eq!(next, Px(10.0));
    }
}
