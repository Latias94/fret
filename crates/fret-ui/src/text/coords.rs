use fret_core::Px;
use fret_core::{Point, Rect};

pub(crate) fn compute_text_vertical_offset(bounds_height: Px, metrics_height: Px) -> Px {
    Px(((bounds_height.0 - metrics_height.0) * 0.5).max(0.0))
}

pub(crate) fn compute_text_ink_overflow_padding(
    services: &mut dyn fret_core::TextService,
    blob: fret_core::TextBlobId,
) -> (Px, Px) {
    let Some(first_line) = services.first_line_metrics(blob) else {
        return (Px(0.0), Px(0.0));
    };
    let Some(first_ink) = services.first_line_ink_metrics(blob) else {
        return (Px(0.0), Px(0.0));
    };

    let pad_top = Px((first_ink.ascent.0 - first_line.ascent.0).max(0.0));

    let last_line = services.last_line_metrics(blob).unwrap_or(first_line);
    let last_ink = services
        .last_line_ink_metrics(blob)
        .or_else(|| services.first_line_ink_metrics(blob));
    let pad_bottom = last_ink
        .map(|ink| Px((ink.descent.0 - last_line.descent.0).max(0.0)))
        .unwrap_or(Px(0.0));

    (pad_top, pad_bottom)
}

pub(crate) fn clamp_text_ink_overflow_padding_to_bounds(
    metrics_height: Px,
    bounds_height: Px,
    requested_top: Px,
    requested_bottom: Px,
) -> (Px, Px) {
    let requested_total = requested_top.0 + requested_bottom.0;
    if !requested_total.is_finite() || requested_total <= 0.0 {
        return (Px(0.0), Px(0.0));
    }

    let extra = (bounds_height.0 - metrics_height.0).max(0.0);
    if !extra.is_finite() || extra <= 0.0 {
        return (Px(0.0), Px(0.0));
    }

    let scale = (extra / requested_total).min(1.0).max(0.0);
    (Px(requested_top.0 * scale), Px(requested_bottom.0 * scale))
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TextBoxMapping {
    /// Window-space origin of the prepared text box (x=0, y=0 in text-local coordinates).
    pub box_origin: Point,
}

impl TextBoxMapping {
    pub fn new(box_origin: Point) -> Self {
        Self { box_origin }
    }

    pub fn baseline_origin(&self, baseline: Px) -> Point {
        Point::new(self.box_origin.x, self.box_origin.y + baseline)
    }

    pub fn window_to_text_local(&self, point: Point) -> Point {
        Point::new(
            Px(point.x.0 - self.box_origin.x.0),
            Px(point.y.0 - self.box_origin.y.0),
        )
    }

    pub fn text_local_to_window_point(&self, point: Point) -> Point {
        Point::new(
            Px(self.box_origin.x.0 + point.x.0),
            Px(self.box_origin.y.0 + point.y.0),
        )
    }

    pub fn text_local_to_window_rect(&self, rect: Rect) -> Rect {
        Rect::new(self.text_local_to_window_point(rect.origin), rect.size)
    }
}

pub(crate) fn compute_text_box_mapping_for_vertical_placement(
    services: &mut dyn fret_core::TextService,
    blob: fret_core::TextBlobId,
    bounds: Rect,
    metrics: fret_core::TextMetrics,
    vertical_placement: fret_core::TextVerticalPlacement,
) -> (TextBoxMapping, Px, Px) {
    let (vertical_offset, baseline) = compute_text_vertical_offset_and_baseline(
        services,
        blob,
        bounds.size.height,
        metrics,
        vertical_placement,
    );

    (
        TextBoxMapping::new(Point::new(
            bounds.origin.x,
            bounds.origin.y + vertical_offset,
        )),
        vertical_offset,
        baseline,
    )
}
