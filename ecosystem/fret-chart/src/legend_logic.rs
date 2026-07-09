use delinea::SeriesId;
use delinea::engine::model::ChartModel;
use fret_core::Px;

const LEGEND_WHEEL_SCROLL_SPEED: f32 = 0.75;

pub(crate) fn legend_max_scroll_y(content_height: Px, view_height: Px) -> Px {
    if content_height.0 <= view_height.0 {
        return Px(0.0);
    }
    Px(content_height.0 - view_height.0)
}

pub(crate) fn legend_clamp_scroll_y(scroll_y: Px, content_height: Px, view_height: Px) -> Px {
    Px(scroll_y
        .0
        .clamp(0.0, legend_max_scroll_y(content_height, view_height).0))
}

pub(crate) fn legend_scroll_after_wheel(
    scroll_y: Px,
    content_height: Px,
    view_height: Px,
    wheel_delta_y: Px,
) -> (Px, bool) {
    let max_scroll = legend_max_scroll_y(content_height, view_height);
    if max_scroll.0 <= 0.0 {
        return (Px(0.0), scroll_y.0 != 0.0);
    }

    let next =
        Px((scroll_y.0 - wheel_delta_y.0 * LEGEND_WHEEL_SCROLL_SPEED).clamp(0.0, max_scroll.0));
    (next, next.0 != scroll_y.0)
}

pub(crate) fn legend_select_all_updates(model: &ChartModel) -> Vec<(SeriesId, bool)> {
    let mut updates = Vec::new();
    for s in model.series_in_order() {
        if !s.visible {
            updates.push((s.id, true));
        }
    }
    updates
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn legend_scroll_policy_clamps_to_content_height() {
        let content = Px(500.0);
        let view = Px(120.0);

        assert_eq!(legend_max_scroll_y(content, view), Px(380.0));
        assert_eq!(legend_clamp_scroll_y(Px(999.0), content, view), Px(380.0));
        assert_eq!(legend_clamp_scroll_y(Px(-10.0), content, view), Px(0.0));

        let (next, changed) = legend_scroll_after_wheel(Px(0.0), content, view, Px(-200.0));
        assert!(changed);
        assert_eq!(next, Px(150.0));

        let (next, changed) = legend_scroll_after_wheel(next, content, view, Px(-10_000.0));
        assert!(changed);
        assert_eq!(next, Px(380.0));

        let (next, changed) = legend_scroll_after_wheel(next, content, view, Px(10_000.0));
        assert!(changed);
        assert_eq!(next, Px(0.0));
    }

    #[test]
    fn legend_scroll_policy_resets_when_content_fits() {
        let (next, changed) = legend_scroll_after_wheel(Px(42.0), Px(80.0), Px(120.0), Px(-10.0));
        assert_eq!(next, Px(0.0));
        assert!(changed);

        let (next, changed) = legend_scroll_after_wheel(Px(0.0), Px(80.0), Px(120.0), Px(-10.0));
        assert_eq!(next, Px(0.0));
        assert!(!changed);
    }
}

pub(crate) fn legend_select_none_updates(model: &ChartModel) -> Vec<(SeriesId, bool)> {
    let mut updates = Vec::new();
    for s in model.series_in_order() {
        if s.visible {
            updates.push((s.id, false));
        }
    }
    updates
}

pub(crate) fn legend_invert_updates(model: &ChartModel) -> Vec<(SeriesId, bool)> {
    let mut updates = Vec::new();
    for s in model.series_in_order() {
        updates.push((s.id, !s.visible));
    }
    updates
}

pub(crate) fn legend_reset_updates(model: &ChartModel) -> Vec<(SeriesId, bool)> {
    legend_select_all_updates(model)
}

pub(crate) fn legend_double_click_updates(
    model: &ChartModel,
    clicked: SeriesId,
) -> Vec<(SeriesId, bool)> {
    if model.series_order.is_empty() {
        return Vec::new();
    }

    let clicked_visible = model
        .series
        .get(&clicked)
        .map(|s| s.visible)
        .unwrap_or(true);
    let only_clicked_visible = clicked_visible
        && model
            .series_in_order()
            .all(|s| s.id == clicked || !s.visible);

    let mut updates = Vec::new();
    if only_clicked_visible {
        for s in model.series_in_order() {
            if !s.visible {
                updates.push((s.id, true));
            }
        }
    } else {
        for s in model.series_in_order() {
            let target = s.id == clicked;
            if s.visible != target {
                updates.push((s.id, target));
            }
        }
    }
    updates
}

pub(crate) fn legend_shift_range_toggle_updates(
    model: &ChartModel,
    anchor: SeriesId,
    clicked: SeriesId,
) -> Vec<(SeriesId, bool)> {
    let Some(anchor_idx) = model.series_order.iter().position(|id| *id == anchor) else {
        return Vec::new();
    };
    let Some(clicked_idx) = model.series_order.iter().position(|id| *id == clicked) else {
        return Vec::new();
    };

    let clicked_visible = model
        .series
        .get(&clicked)
        .map(|s| s.visible)
        .unwrap_or(true);
    let target = !clicked_visible;

    let (lo, hi) = if anchor_idx <= clicked_idx {
        (anchor_idx, clicked_idx)
    } else {
        (clicked_idx, anchor_idx)
    };

    let mut updates = Vec::new();
    for id in &model.series_order[lo..=hi] {
        if let Some(s) = model.series.get(id)
            && s.visible != target
        {
            updates.push((*id, target));
        }
    }
    updates
}
