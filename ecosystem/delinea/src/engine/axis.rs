use crate::engine::model::{AxisModel, ChartModel};
use crate::engine::window::DataWindow;
use crate::ids::AxisId;
use crate::scale::AxisScale;
use fret_core::Rect;

pub fn data_at_px(window: DataWindow, px: f32, origin_px: f32, span_px: f32) -> f64 {
    let mut window = window;
    window.clamp_non_degenerate();

    let span = window.span();
    if !span.is_finite() || span <= 0.0 {
        return window.min;
    }

    if !span_px.is_finite() || span_px <= 0.0 {
        return window.min;
    }

    let t = ((px - origin_px) / span_px).clamp(0.0, 1.0) as f64;
    window.min + t * span
}

pub fn data_at_x_in_rect(window: DataWindow, x_px: f32, rect: Rect) -> f64 {
    data_at_px(window, x_px, rect.origin.x.0, rect.size.width.0)
}

pub fn category_domain_window(axis: &AxisModel) -> Option<DataWindow> {
    let AxisScale::Category(scale) = &axis.scale else {
        return None;
    };
    category_domain_window_len(scale.len())
}

fn category_domain_window_len(n: usize) -> Option<DataWindow> {
    if n == 0 {
        return None;
    }

    Some(DataWindow {
        min: -0.5,
        max: n as f64 - 0.5,
    })
}

pub fn format_value(axis: &AxisModel, window: DataWindow, value: f64) -> String {
    match &axis.scale {
        AxisScale::Value(_) => crate::format::format_tick_value(window, value),
        AxisScale::Category(scale) => format_category_value(scale, value),
    }
}

pub fn axis_ticks(axis: &AxisModel, window: DataWindow, target_count: usize) -> Vec<f64> {
    match &axis.scale {
        AxisScale::Value(_) => crate::format::nice_ticks(window, target_count),
        AxisScale::Category(scale) => category_ticks(scale, window, target_count),
    }
}

pub fn format_axis_tick(axis: &AxisModel, window: DataWindow, value: f64) -> String {
    match &axis.scale {
        AxisScale::Value(_) => crate::format::format_tick_value(window, value),
        AxisScale::Category(scale) => format_category_value(scale, value),
    }
}

pub fn axis_ticks_for(
    model: &ChartModel,
    axis_id: AxisId,
    window: DataWindow,
    count: usize,
) -> Vec<f64> {
    model
        .axes
        .get(&axis_id)
        .map(|axis| axis_ticks(axis, window, count))
        .unwrap_or_else(|| crate::format::nice_ticks(window, count))
}

pub fn format_value_for(
    model: &ChartModel,
    axis_id: AxisId,
    window: DataWindow,
    value: f64,
) -> String {
    model
        .axes
        .get(&axis_id)
        .map(|axis| format_value(axis, window, value))
        .unwrap_or_else(|| crate::format::format_tick_value(window, value))
}

fn format_category_value(scale: &crate::scale::CategoryAxisScale, value: f64) -> String {
    if scale.is_empty() || !value.is_finite() {
        return value.to_string();
    }

    let idx = (value.round() as isize).clamp(0, scale.len().saturating_sub(1) as isize) as usize;
    scale
        .categories
        .get(idx)
        .cloned()
        .unwrap_or_else(|| idx.to_string())
}

fn category_ticks(
    scale: &crate::scale::CategoryAxisScale,
    mut window: DataWindow,
    target_count: usize,
) -> Vec<f64> {
    let n = scale.len();
    if n == 0 {
        return Vec::new();
    }

    if let Some(domain) = category_domain_window_len(n) {
        window.clamp_non_degenerate();
        let min = window.min.max(domain.min);
        let max = window.max.min(domain.max);
        window = DataWindow { min, max };
    }

    let start = window.min.ceil() as isize;
    let end = window.max.floor() as isize;
    let start = start.max(0) as usize;
    let end = end.min(n.saturating_sub(1) as isize) as usize;
    if end < start {
        return Vec::new();
    }

    let visible = end - start + 1;
    let step = if target_count == 0 {
        visible
    } else {
        (visible as f32 / target_count as f32).ceil().max(1.0) as usize
    };

    let mut ticks = Vec::new();
    let mut i = start;
    while i <= end {
        ticks.push(i as f64);
        if end - i < step {
            break;
        }
        i += step;
    }
    if ticks.last().copied() != Some(end as f64) {
        ticks.push(end as f64);
    }
    ticks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category_axis(categories: &[&str]) -> AxisModel {
        AxisModel {
            id: AxisId::new(1),
            name: None,
            kind: crate::spec::AxisKind::X,
            grid: crate::ids::GridId::new(1),
            position: crate::spec::AxisPosition::Bottom,
            scale: AxisScale::Category(crate::scale::CategoryAxisScale {
                categories: categories.iter().map(|s| (*s).to_string()).collect(),
            }),
            range: crate::spec::AxisRange::default(),
        }
    }

    #[test]
    fn category_domain_window_uses_band_edges() {
        let axis = category_axis(&["a", "b", "c"]);
        assert_eq!(
            category_domain_window(&axis),
            Some(DataWindow {
                min: -0.5,
                max: 2.5
            })
        );
    }

    #[test]
    fn format_value_category_rounds_and_clamps() {
        let axis = category_axis(&["a", "b", "c"]);
        let window = DataWindow {
            min: -0.5,
            max: 2.5,
        };
        assert_eq!(format_value(&axis, window, -10.0), "a");
        assert_eq!(format_value(&axis, window, 0.49), "a");
        assert_eq!(format_value(&axis, window, 0.51), "b");
        assert_eq!(format_value(&axis, window, 10.0), "c");
    }

    #[test]
    fn category_ticks_decimate_and_include_endpoints() {
        let axis = category_axis(&["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);
        let window = DataWindow {
            min: -0.5,
            max: 9.5,
        };
        let ticks = axis_ticks(&axis, window, 4);
        assert_eq!(ticks.first().copied(), Some(0.0));
        assert_eq!(ticks.last().copied(), Some(9.0));
        assert!(ticks.len() <= 5);
    }
}
