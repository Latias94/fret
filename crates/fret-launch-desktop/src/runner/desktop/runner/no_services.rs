//! No-op service implementations used when the real services are not available.

pub(super) struct NoUiServices;

impl fret_core::TextService for NoUiServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        // Even in no-op mode, keep geometry outputs non-degenerate so UI code relying on caret/
        // selection rects and IME anchoring remains robust during bootstrap or headless phases.
        //
        // This intentionally does not attempt to measure widths; it only provides a stable,
        // visible baseline height.
        let line_height = fret_core::Px(16.0);
        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: fret_core::Size::new(fret_core::Px(0.0), line_height),
                baseline: fret_core::Px(12.8),
            },
        )
    }

    fn first_line_metrics(
        &mut self,
        _blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextLineMetrics> {
        Some(fret_core::TextLineMetrics {
            ascent: fret_core::Px(12.8),
            descent: fret_core::Px(3.2),
            line_height: fret_core::Px(16.0),
        })
    }

    fn caret_rect(
        &mut self,
        _blob: fret_core::TextBlobId,
        _index: usize,
        _affinity: fret_core::CaretAffinity,
    ) -> fret_core::Rect {
        fret_core::Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            fret_core::Size::new(fret_core::Px(1.0), fret_core::Px(16.0)),
        )
    }

    fn selection_rects(
        &mut self,
        _blob: fret_core::TextBlobId,
        (a, b): (usize, usize),
        out: &mut Vec<fret_core::Rect>,
    ) {
        out.clear();
        let (start, end) = (a.min(b), a.max(b));
        if start == end {
            return;
        }
        out.push(fret_core::Rect::new(
            fret_core::Point::new(fret_core::Px(start as f32), fret_core::Px(0.0)),
            fret_core::Size::new(fret_core::Px((end - start) as f32), fret_core::Px(16.0)),
        ));
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for NoUiServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for NoUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for NoUiServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}
