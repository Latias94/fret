use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::super::super::{ColorEditAlphaPreview, ColorEditPopupOptions};
use super::super::super::preview::color_side_preview;

pub(super) fn color_popup_side_preview_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    reference_color: Option<Color>,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    effective_popup_options: ColorEditPopupOptions,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    popup_test_id: Option<&Arc<str>>,
) -> Option<AnyElement> {
    effective_popup_options
        .side_preview
        .has_visible_content()
        .then(|| {
            color_side_preview(
                cx,
                current,
                reference_color,
                model,
                draft,
                error,
                effective_popup_options.side_preview,
                show_alpha,
                enabled,
                alpha_preview,
                derived_test_id(popup_test_id, "preview"),
            )
        })
}
