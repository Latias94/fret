use std::sync::Arc;

use fret_core::{Color, Px};
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::colors::{editor_invalid_border, editor_muted_foreground};
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, editor_axis_segment, editor_icon_button_segment,
    editor_icon_segment, editor_input_group_divider, editor_input_group_frame_with_overrides,
    editor_input_group_inset, editor_input_group_row, editor_text_segment,
};
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

use super::super::model::AxisDragValueResetAction;

pub(super) struct AxisDragValueTypingFrameArgs {
    pub(super) layout: LayoutStyle,
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) is_focused: bool,
    pub(super) has_error: bool,
    pub(super) input: AnyElement,
    pub(super) axis_label: Arc<str>,
    pub(super) axis_tint: Color,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) reset_action: Option<AxisDragValueResetAction>,
    pub(super) enabled: bool,
    pub(super) active_typing_test_id: Option<Arc<str>>,
    pub(super) typing_axis_test_id: Option<Arc<str>>,
    pub(super) typing_prefix_test_id: Option<Arc<str>>,
    pub(super) typing_suffix_test_id: Option<Arc<str>>,
    pub(super) typing_error_icon_test_id: Option<Arc<str>>,
    pub(super) typing_reset_test_id: Option<Arc<str>>,
}

pub(super) fn axis_drag_value_typing_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueTypingFrameArgs,
) -> AnyElement {
    let AxisDragValueTypingFrameArgs {
        layout,
        density,
        frame_chrome,
        is_focused,
        has_error,
        input,
        axis_label,
        axis_tint,
        prefix,
        suffix,
        reset_action,
        enabled,
        active_typing_test_id,
        typing_axis_test_id,
        typing_prefix_test_id,
        typing_suffix_test_id,
        typing_error_icon_test_id,
        typing_reset_test_id,
    } = args;

    let divider = frame_chrome.border;
    let mut typing_frame = editor_input_group_frame_with_overrides(
        cx,
        layout,
        density,
        frame_chrome,
        EditorFrameState {
            enabled: true,
            hovered: false,
            pressed: false,
            focused: is_focused,
            open: false,
            semantic: EditorFrameSemanticState {
                typing: true,
                invalid: has_error,
            },
        },
        EditorInputGroupFrameOverrides::none(),
        move |cx, visuals| {
            let affix_color = {
                let theme = Theme::global(&*cx.app);
                editor_muted_foreground(theme)
            };
            let mut axis =
                editor_axis_segment(cx, density, axis_label.clone(), axis_tint, visuals.bg);
            if let Some(test_id) = typing_axis_test_id.as_ref() {
                axis = axis.test_id(test_id.clone()).a11y_label(axis_label.clone());
            }
            let sep = editor_input_group_divider(cx, divider);

            let input_wrap = editor_input_group_inset(cx, frame_chrome.padding, input);

            let mut segments = vec![axis, sep];
            if let Some(prefix) = prefix.clone() {
                let mut segment = editor_text_segment(
                    cx,
                    density,
                    frame_chrome.text_px,
                    prefix.clone(),
                    affix_color,
                    frame_chrome.padding,
                );
                if let Some(test_id) = typing_prefix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(prefix);
                }
                segments.push(segment);
                segments.push(editor_input_group_divider(cx, divider));
            }
            segments.push(input_wrap);
            if let Some(suffix) = suffix.clone() {
                segments.push(editor_input_group_divider(cx, divider));
                let mut segment = editor_text_segment(
                    cx,
                    density,
                    frame_chrome.text_px,
                    suffix.clone(),
                    affix_color,
                    frame_chrome.padding,
                );
                if let Some(test_id) = typing_suffix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(suffix);
                }
                segments.push(segment);
            }
            if has_error {
                let error_border = {
                    let theme = Theme::global(&*cx.app);
                    editor_invalid_border(theme)
                };
                segments.push(editor_input_group_divider(cx, divider));
                let mut icon = editor_icon_segment(
                    cx,
                    density,
                    fret_icons::ids::ui::STATUS_FAILED,
                    Some(Px(12.0)),
                    Some(fret_ui_kit::ColorRef::Color(error_border)),
                );
                if let Some(test_id) = typing_error_icon_test_id.as_ref() {
                    icon = icon.test_id(test_id.clone());
                }
                segments.push(icon);
            }
            if let Some(reset) = reset_action {
                segments.push(editor_input_group_divider(cx, divider));
                segments.push(editor_icon_button_segment(
                    cx,
                    density,
                    enabled,
                    reset.a11y_label,
                    reset.icon,
                    Some(Px(12.0)),
                    typing_reset_test_id.clone(),
                    reset.on_activate,
                ));
            }

            vec![editor_input_group_row(cx, Px(0.0), segments)]
        },
    );
    if let Some(test_id) = active_typing_test_id.as_ref() {
        typing_frame = typing_frame.test_id(test_id.clone());
    }
    typing_frame
}
