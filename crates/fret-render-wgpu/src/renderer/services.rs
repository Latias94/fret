use super::*;

impl fret_core::TextService for Renderer {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        match input {
            fret_core::TextInput::Plain { text, style } => {
                self.text_system.prepare(text.as_ref(), style, constraints)
            }
            fret_core::TextInput::Attributed { text, base, spans } => {
                let rich = fret_core::AttributedText::new(text.clone(), spans.clone());
                self.text_system
                    .prepare_attributed(&rich, base, constraints)
            }
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                self.text_system.prepare(
                    input.text(),
                    &fret_core::TextStyle::default(),
                    constraints,
                )
            }
        }
    }

    fn measure(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> fret_core::TextMetrics {
        match input {
            fret_core::TextInput::Plain { text, style } => {
                self.text_system.measure(text.as_ref(), style, constraints)
            }
            fret_core::TextInput::Attributed { text, base, spans } => {
                let rich = fret_core::AttributedText::new(text.clone(), spans.clone());
                self.text_system
                    .measure_attributed(&rich, base, constraints)
            }
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                self.text_system.measure(
                    input.text(),
                    &fret_core::TextStyle::default(),
                    constraints,
                )
            }
        }
    }

    fn caret_x(&mut self, blob: fret_core::TextBlobId, index: usize) -> fret_core::Px {
        self.text_system
            .caret_x(blob, index)
            .unwrap_or(fret_core::Px(0.0))
    }

    fn hit_test_x(&mut self, blob: fret_core::TextBlobId, x: fret_core::Px) -> usize {
        self.text_system.hit_test_x(blob, x).unwrap_or(0)
    }

    fn selection_rects(
        &mut self,
        blob: fret_core::TextBlobId,
        range: (usize, usize),
        out: &mut Vec<fret_core::Rect>,
    ) {
        let _ = self.text_system.selection_rects(blob, range, out);
    }

    fn first_line_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextLineMetrics> {
        self.text_system.first_line_metrics(blob)
    }

    fn first_line_ink_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextInkMetrics> {
        self.text_system.first_line_ink_metrics(blob)
    }

    fn last_line_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextLineMetrics> {
        self.text_system.last_line_metrics(blob)
    }

    fn last_line_ink_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextInkMetrics> {
        self.text_system.last_line_ink_metrics(blob)
    }

    fn selection_rects_clipped(
        &mut self,
        blob: fret_core::TextBlobId,
        range: (usize, usize),
        clip: fret_core::Rect,
        out: &mut Vec<fret_core::Rect>,
    ) {
        let _ = self
            .text_system
            .selection_rects_clipped(blob, range, clip, out);
    }

    fn caret_stops(&mut self, blob: fret_core::TextBlobId, out: &mut Vec<(usize, fret_core::Px)>) {
        out.clear();
        if let Some(stops) = self.text_system.caret_stops(blob) {
            out.extend_from_slice(stops);
        }
    }

    fn caret_rect(
        &mut self,
        blob: fret_core::TextBlobId,
        index: usize,
        affinity: fret_core::CaretAffinity,
    ) -> fret_core::Rect {
        self.text_system
            .caret_rect(blob, index, affinity)
            .unwrap_or_default()
    }

    fn hit_test_point(
        &mut self,
        blob: fret_core::TextBlobId,
        point: fret_core::Point,
    ) -> fret_core::HitTestResult {
        self.text_system
            .hit_test_point(blob, point)
            .unwrap_or(fret_core::HitTestResult {
                index: 0,
                affinity: fret_core::CaretAffinity::Downstream,
            })
    }

    fn release(&mut self, blob: fret_core::TextBlobId) {
        self.text_system.release(blob);
    }
}

impl fret_core::PathService for Renderer {
    fn prepare(
        &mut self,
        commands: &[fret_core::PathCommand],
        style: fret_core::PathStyle,
        constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        self.path_state.prepare_path(commands, style, constraints)
    }

    fn release(&mut self, path: fret_core::PathId) {
        self.path_state.release_path(path);
    }
}
