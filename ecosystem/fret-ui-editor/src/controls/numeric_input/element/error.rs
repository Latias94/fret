//! NumericInput error display element owner.

use std::sync::Arc;

use fret_core::{Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::colors::{editor_invalid_border, editor_invalid_foreground};
use crate::primitives::input_group::editor_icon_segment;
use crate::primitives::readout::editor_validation_message_text_props;

use super::super::model::NumericInputErrorDisplay;

pub(super) fn numeric_input_trailing_error_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    error_display: NumericInputErrorDisplay,
    error: &Model<Option<Arc<str>>>,
    error_icon_test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    if !matches!(
        error_display,
        NumericInputErrorDisplay::TrailingIcon | NumericInputErrorDisplay::InlineTextAndIcon
    ) {
        return None;
    }

    cx.get_model_cloned(error, Invalidation::Paint)
        .unwrap_or(None)?;

    let error_border = {
        let theme = Theme::global(&*cx.app);
        editor_invalid_border(theme)
    };

    let mut icon = editor_icon_segment(
        cx,
        density,
        fret_icons::ids::ui::STATUS_FAILED,
        Some(Px(12.0)),
        Some(fret_ui_kit::ColorRef::Color(error_border)),
    );
    if let Some(test_id) = error_icon_test_id.as_ref() {
        icon = icon.test_id(test_id.clone());
    }
    Some(icon)
}

pub(super) fn numeric_input_inline_error<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    error_display: NumericInputErrorDisplay,
    error_msg: Option<Arc<str>>,
    error_text_test_id: Option<Arc<str>>,
    source_text_style: TextStyle,
) -> Option<AnyElement> {
    if !matches!(
        error_display,
        NumericInputErrorDisplay::InlineText | NumericInputErrorDisplay::InlineTextAndIcon
    ) {
        return None;
    }

    let msg = error_msg?;
    let error_color = {
        let theme = Theme::global(&*cx.app);
        editor_invalid_foreground(theme)
    };
    let mut error = cx.text_props(editor_validation_message_text_props(
        msg.clone(),
        error_color,
        TextStyle {
            size: source_text_style.size,
            line_height: source_text_style.line_height,
            ..Default::default()
        },
    ));
    if let Some(test_id) = error_text_test_id.as_ref() {
        error = error.test_id(test_id.clone()).a11y_label(msg.clone());
    }
    Some(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Rect};
    use fret_ui::element::ElementKind;
    use fret_ui::elements::with_element_cx;

    #[test]
    fn inline_error_text_is_only_rendered_when_requested() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let element = with_element_cx(
            &mut app,
            window,
            Rect::default(),
            "numeric-input-inline-error",
            |cx| {
                numeric_input_inline_error(
                    cx,
                    NumericInputErrorDisplay::InlineTextAndIcon,
                    Some(Arc::from("bad input")),
                    None,
                    TextStyle::default(),
                )
            },
        );

        let Some(element) = element else {
            panic!("inline error text should be rendered when explicitly requested");
        };
        assert!(matches!(element.kind, ElementKind::Text(_)));
    }
}
