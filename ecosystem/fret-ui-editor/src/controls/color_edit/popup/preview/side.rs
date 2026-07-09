use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::super::{ColorEditAlphaPreview, ColorEditPopupSidePreview};

mod cell;
mod original;

use cell::current_preview_cell;

#[cfg(test)]
pub(in crate::controls::color_edit) use cell::{
    SIDE_PREVIEW_SWATCH_HEIGHT, SIDE_PREVIEW_SWATCH_WIDTH,
};
#[cfg(test)]
pub(in crate::controls::color_edit) use original::restore_reference_color;

pub(in crate::controls::color_edit::popup) fn color_side_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    original: Option<Color>,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    mode: ColorEditPopupSidePreview,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let original_cell = mode
        .shows_original()
        .then_some(original)
        .flatten()
        .map(|original| {
            original::original_reference_preview_cell(
                cx,
                original,
                model,
                draft,
                error,
                show_alpha,
                enabled,
                alpha_preview,
                derived_test_id(test_id.as_ref(), "original"),
            )
        });
    let current_cell = current_preview_cell(
        cx,
        current,
        show_alpha,
        alpha_preview,
        derived_test_id(test_id.as_ref(), "current"),
    );

    let mut preview = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![current_cell];
            if let Some(original_cell) = original_cell {
                out.push(original_cell);
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        preview = preview.test_id(test_id);
    }
    preview
}
