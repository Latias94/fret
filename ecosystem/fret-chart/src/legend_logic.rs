use delinea::SeriesId;
use delinea::engine::model::ChartModel;

pub(crate) fn legend_select_all_updates(model: &ChartModel) -> Vec<(SeriesId, bool)> {
    let mut updates = Vec::new();
    for s in model.series_in_order() {
        if !s.visible {
            updates.push((s.id, true));
        }
    }
    updates
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
