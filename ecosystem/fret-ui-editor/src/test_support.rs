use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{
    MaterialDescriptor, MaterialId, MaterialRegistrationError, PathCommand, PathConstraints,
    PathId, PathMetrics, PathService, PathStyle, Size, SvgId, SvgService, TextBlobId,
    TextConstraints, TextInput, TextMetrics, TextService,
};

#[derive(Default)]
pub(crate) struct WrappingTextServices;

impl TextService for WrappingTextServices {
    fn prepare(
        &mut self,
        input: &TextInput,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let text = input.text();
        let char_width = fret_core::Px(7.0);
        let line_height = fret_core::Px(14.0);
        let unwrapped_width = fret_core::Px(text.chars().count() as f32 * char_width.0);
        let lines = match (constraints.wrap, constraints.max_width) {
            (TextWrap::None, _) | (_, None) => 1usize,
            (_, Some(max_width)) if max_width.0 <= char_width.0 => text.chars().count().max(1),
            (_, Some(max_width)) => {
                let chars_per_line = (max_width.0 / char_width.0).floor().max(1.0) as usize;
                text.chars().count().max(1).div_ceil(chars_per_line)
            }
        };
        let width = match (constraints.overflow, constraints.max_width) {
            (TextOverflow::Ellipsis, Some(max_width)) => {
                fret_core::Px(unwrapped_width.0.min(max_width.0))
            }
            (_, Some(max_width)) if constraints.wrap != TextWrap::None => {
                fret_core::Px(unwrapped_width.0.min(max_width.0))
            }
            _ => unwrapped_width,
        };

        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(width, fret_core::Px(lines as f32 * line_height.0)),
                baseline: fret_core::Px(11.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for WrappingTextServices {
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

impl SvgService for WrappingTextServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for WrappingTextServices {
    fn register_material(
        &mut self,
        _desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError> {
        Err(MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: MaterialId) -> bool {
        false
    }
}
