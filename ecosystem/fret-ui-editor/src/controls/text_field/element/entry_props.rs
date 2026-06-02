//! TextField text-entry props owner.

use std::sync::Arc;

use fret_core::{FontId, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::Theme;
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextAreaProps, TextInputProps};
use fret_ui_kit::ChromeRefinement;
use fret_ui_kit::Size;
use fret_ui_kit::typography;

use crate::primitives::EditorDensity;
use crate::primitives::chrome::{
    joined_text_area_style, joined_text_input_style, resolve_editor_text_area_field_style,
    resolve_editor_text_field_style,
};
use crate::primitives::text_entry::EditorTextCancelBehavior;

use super::super::{TextFieldAssistiveSemantics, TextFieldMode};

pub(super) struct TextFieldAreaPropsArgs {
    pub(super) input_model: Model<String>,
    pub(super) size: Size,
    pub(super) density: EditorDensity,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) stable_line_boxes: bool,
    pub(super) min_height: Option<Px>,
}

pub(super) struct TextFieldInputPropsArgs {
    pub(super) input_model: Model<String>,
    pub(super) size: Size,
    pub(super) density: EditorDensity,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) placeholder: Option<Arc<str>>,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) mode: TextFieldMode,
    pub(super) assistive_semantics: TextFieldAssistiveSemantics,
    pub(super) buffered: bool,
    pub(super) submit_command: Option<CommandId>,
    pub(super) cancel_behavior: EditorTextCancelBehavior,
}

pub(super) fn text_field_area_props(theme: &Theme, args: TextFieldAreaPropsArgs) -> TextAreaProps {
    let TextFieldAreaPropsArgs {
        input_model,
        size,
        density,
        enabled,
        focusable,
        a11y_label,
        test_id,
        stable_line_boxes,
        min_height,
    } = args;

    let (chrome, text_style) =
        resolve_editor_text_area_field_style(theme, size, &ChromeRefinement::default());
    let text_style = if stable_line_boxes {
        typography::text_area_control_text_style_scaled(theme, FontId::ui(), text_style.size)
    } else {
        text_style
    };

    let mut props = TextAreaProps::new(input_model);
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        ..Default::default()
    };
    props.enabled = enabled;
    props.focusable = focusable;
    props.a11y_label = a11y_label;
    props.test_id = test_id;
    props.chrome = joined_text_area_style(chrome);
    props.text_style = text_style;
    props.min_height = min_height.unwrap_or_else(|| {
        let baseline = Px(80.0);
        let dense = Px(density.row_height.0 * 3.0);
        Px(baseline.0.max(dense.0))
    });
    props
}

pub(super) fn text_field_input_props(
    theme: &Theme,
    args: TextFieldInputPropsArgs,
) -> TextInputProps {
    let TextFieldInputPropsArgs {
        input_model,
        size,
        density,
        enabled,
        focusable,
        placeholder,
        a11y_label,
        test_id,
        mode,
        assistive_semantics,
        buffered,
        submit_command,
        cancel_behavior,
    } = args;

    let (chrome, text_style) =
        resolve_editor_text_field_style(theme, size, &ChromeRefinement::default());

    let mut props = TextInputProps::new(input_model);
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            min_height: Some(Length::Px(density.row_height)),
            ..Default::default()
        },
        ..Default::default()
    };
    props.enabled = enabled;
    props.focusable = focusable;
    props.placeholder = placeholder;
    props.a11y_label = a11y_label;
    props.test_id = test_id;
    props.obscure_text = matches!(mode, TextFieldMode::Password);
    props.active_descendant = assistive_semantics.active_descendant;
    props.active_descendant_element = assistive_semantics.active_descendant_element;
    props.controls_element = assistive_semantics.controls_element;
    props.expanded = assistive_semantics.expanded;
    if !buffered {
        props.submit_command = submit_command;
    }
    if !buffered && matches!(cancel_behavior, EditorTextCancelBehavior::Clear) {
        props.cancel_command = Some("text.clear".into());
    }
    props.chrome = joined_text_input_style(chrome);
    props.text_style = typography::as_control_text(TextStyle {
        line_height: Some(density.row_height),
        ..text_style
    });
    props
}
