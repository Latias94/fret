//! Axis-drag-value typing TextInput owner.

use std::sync::Arc;

use fret_core::{SemanticsInvalid, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, GlobalElementId, TextInputStyle, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::joined_text_input_style;

pub(super) struct AxisDragValueTypingInputArgs {
    pub(super) draft: Model<String>,
    pub(super) density: EditorDensity,
    pub(super) layout: LayoutStyle,
    pub(super) input_chrome: TextInputStyle,
    pub(super) text_style: TextStyle,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) typing: bool,
    pub(super) typing_input_test_id: Option<Arc<str>>,
    pub(super) has_error: bool,
}

pub(super) struct AxisDragValueTypingInput {
    pub(super) input: AnyElement,
    pub(super) input_id: GlobalElementId,
    pub(super) is_focused: bool,
}

pub(super) fn axis_drag_value_typing_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueTypingInputArgs,
) -> AxisDragValueTypingInput {
    let AxisDragValueTypingInputArgs {
        draft,
        density,
        layout,
        input_chrome,
        text_style,
        enabled,
        focusable,
        typing,
        typing_input_test_id,
        has_error,
    } = args;

    let mut props = TextInputProps::new(draft);
    props.layout = if typing {
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                min_height: Some(Length::Px(density.row_height)),
                ..Default::default()
            },
            ..Default::default()
        }
    } else {
        layout
    };
    props.enabled = enabled && typing;
    props.focusable = focusable && typing;
    props.test_id = typing_input_test_id;
    props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
    props.chrome = joined_text_input_style(input_chrome);
    props.text_style = text_style;

    let input = cx.text_input(props);
    let input_id = input.id;
    let is_focused = cx.is_focused_element(input_id);
    AxisDragValueTypingInput {
        input,
        input_id,
        is_focused,
    }
}
