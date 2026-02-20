use fret_core::Px;

pub(crate) fn compute_text_vertical_offset(bounds_height: Px, metrics_height: Px) -> Px {
    Px(((bounds_height.0 - metrics_height.0) * 0.5).max(0.0))
}

pub(crate) fn compute_text_vertical_offset_and_baseline(
    services: &mut dyn fret_core::TextService,
    blob: fret_core::TextBlobId,
    bounds_height: Px,
    metrics: fret_core::TextMetrics,
    vertical_placement: fret_core::TextVerticalPlacement,
) -> (Px, Px) {
    match vertical_placement {
        fret_core::TextVerticalPlacement::CenterMetricsBox => (
            compute_text_vertical_offset(bounds_height, metrics.size.height),
            metrics.baseline,
        ),
        fret_core::TextVerticalPlacement::BoundsAsLineBox => {
            let approx_single_line = metrics.size.height.0 <= 0.0
                || services
                    .first_line_metrics(blob)
                    .is_some_and(|m| metrics.size.height.0 <= m.line_height.0 + 0.01);

            if approx_single_line && let Some(line) = services.first_line_metrics(blob) {
                let padding_top =
                    Px(((bounds_height.0 - line.ascent.0 - line.descent.0) * 0.5).max(0.0));
                let baseline =
                    Px((padding_top.0 + line.ascent.0).clamp(0.0, bounds_height.0.max(0.0)));
                (Px(0.0), baseline)
            } else {
                (
                    compute_text_vertical_offset(bounds_height, metrics.size.height),
                    metrics.baseline,
                )
            }
        }
    }
}

pub(crate) fn compute_first_line_box_top_and_height(
    services: &mut dyn fret_core::TextService,
    blob: fret_core::TextBlobId,
    baseline: Px,
    fallback_height: Px,
) -> (Px, Px) {
    if let Some(line) = services.first_line_metrics(blob) {
        let top = Px((baseline.0 - line.ascent.0).max(0.0));
        let height = line.line_height.max(Px(1.0));
        (top, height)
    } else {
        (Px(0.0), fallback_height.max(Px(1.0)))
    }
}
