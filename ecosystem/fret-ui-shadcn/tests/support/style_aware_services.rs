#![allow(dead_code)]

use fret_core::{
    MaterialDescriptor, MaterialId, MaterialRegistrationError, MaterialService, PathCommand,
    PathConstraints, PathId, PathMetrics, PathService, PathStyle, Px, Size as CoreSize, SvgId,
    SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService, TextStyle,
    TextWrap,
};

#[derive(Debug, Clone)]
pub(super) struct RecordedTextPrepare {
    pub(super) text: String,
    pub(super) style: TextStyle,
    pub(super) constraints: TextConstraints,
}

#[derive(Default)]
pub(super) struct StyleAwareServices {
    pub(super) prepared: Vec<RecordedTextPrepare>,
}

impl TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        input: &TextInput,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let (text, style) = match input {
            TextInput::Plain { text, style } => (text.as_ref(), style),
            TextInput::Attributed { text, base, .. } => (text.as_ref(), base),
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                return (
                    TextBlobId::default(),
                    TextMetrics {
                        size: CoreSize::new(Px(0.0), Px(0.0)),
                        baseline: Px(0.0),
                    },
                );
            }
        };

        self.prepared.push(RecordedTextPrepare {
            text: text.to_string(),
            style: style.clone(),
            constraints,
        });

        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text.chars().count() as f32);

        let max_w = constraints.max_width.unwrap_or(est_w);
        let (lines, w) = match constraints.wrap {
            TextWrap::Word if max_w.0.is_finite() && max_w.0 > 0.0 => {
                let lines = (est_w.0 / max_w.0).ceil().max(1.0) as u32;
                (lines, Px(est_w.0.min(max_w.0)))
            }
            _ => (1, est_w),
        };

        let h = Px(line_height.0 * lines as f32);

        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(w, h),
                baseline: Px(h.0 * 0.8),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for StyleAwareServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for StyleAwareServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl MaterialService for StyleAwareServices {
    fn register_material(
        &mut self,
        _desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError> {
        Ok(MaterialId::default())
    }

    fn unregister_material(&mut self, _id: MaterialId) -> bool {
        true
    }
}
