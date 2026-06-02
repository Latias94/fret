//! GradientEditor stop model read/sort owner.

use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::GradientStopBinding;
use super::preview::PreviewStop;

pub(super) struct GradientStopModelRows {
    pub(super) preview_stops: Vec<PreviewStop>,
    pub(super) stop_rows: Vec<(f64, GradientStopBinding)>,
    pub(super) stop_models: Arc<[(fret_ui::ItemKey, Model<f64>)]>,
}

pub(super) fn read_gradient_stop_model_rows<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    stops: &[GradientStopBinding],
) -> GradientStopModelRows {
    let mut preview_stops: Vec<PreviewStop> = Vec::new();
    let mut stop_rows: Vec<(f64, GradientStopBinding)> = Vec::new();

    for stop in stops {
        let pos = cx
            .get_model_copied(&stop.position, Invalidation::Paint)
            .unwrap_or(0.0);
        let color = cx
            .get_model_copied(&stop.color, Invalidation::Paint)
            .unwrap_or(Color::TRANSPARENT);
        preview_stops.push(PreviewStop {
            id: stop.id,
            position: (pos as f32).clamp(0.0, 1.0),
            color,
        });
        stop_rows.push((pos, stop.clone()));
    }

    preview_stops.sort_by(|a, b| a.position.total_cmp(&b.position));
    stop_rows.sort_by(|a, b| a.0.total_cmp(&b.0));

    let stop_models = stops
        .iter()
        .map(|s| (s.id, s.position.clone()))
        .collect::<Vec<_>>()
        .into();

    GradientStopModelRows {
        preview_stops,
        stop_rows,
        stop_models,
    }
}
