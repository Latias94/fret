use super::*;

impl CanvasPaintCache {
    pub(crate) fn text_blob(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let prepared = self
            .text
            .prepare_arc(services, text.into(), style, constraints);
        (prepared.blob, prepared.metrics)
    }

    pub(crate) fn text_metrics(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let text: Arc<str> = text.into();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.as_ref().hash(&mut hasher);
        let text_hash = hasher.finish();

        let q = |value: f32, step: f32| -> i64 {
            if !value.is_finite() {
                return 0;
            }
            (value / step).round() as i64
        };

        let max_width = constraints
            .max_width
            .map(|width| width.0.max(0.0))
            .unwrap_or(0.0);

        let key = TextMetricsKey {
            text_hash,
            text_len: text.len().min(u32::MAX as usize) as u32,
            text: text.clone(),
            font: style.font.clone(),
            size: q(style.size.0.max(0.0), 0.01),
            weight: style.weight.0,
            slant: match style.slant {
                fret_core::TextSlant::Normal => 0,
                fret_core::TextSlant::Italic => 1,
                fret_core::TextSlant::Oblique => 2,
            },
            line_height: q(
                style
                    .line_height
                    .map(|value| value.0)
                    .unwrap_or(0.0)
                    .max(0.0),
                0.01,
            ),
            letter_spacing_em: q(style.letter_spacing_em.unwrap_or(0.0), 0.0001),
            max_width: q(max_width, 0.01),
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_factor: q(constraints.scale_factor.max(0.0), 0.0001),
        };

        let now = self.frame;
        if let Some(entry) = self.text_metrics.get_mut(&key) {
            entry.last_used_frame = now;
            return entry.metrics;
        }

        let metrics = services
            .text()
            .measure_str(text.as_ref(), style, constraints);
        self.text_metrics.insert(
            key,
            TextMetricsEntry {
                metrics,
                last_used_frame: now,
            },
        );
        metrics
    }

    pub(crate) fn text_blob_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
        budget: &mut WorkBudget,
    ) -> (Option<(TextBlobId, TextMetrics)>, bool) {
        let text: Arc<str> = text.into();

        if let Some(prepared) = self.text.get_arc(text.clone(), style, constraints) {
            return (Some((prepared.blob, prepared.metrics)), false);
        }

        if !budget.try_consume(1) {
            return (None, true);
        }

        let prepared = self.text.prepare_arc(services, text, style, constraints);
        (Some((prepared.blob, prepared.metrics)), false)
    }
}
