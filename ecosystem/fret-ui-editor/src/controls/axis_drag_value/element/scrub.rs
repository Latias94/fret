use std::sync::Arc;

use fret_core::{Color, Px};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::input_group::{
    editor_axis_segment, editor_icon_button_segment, editor_input_group_divider,
    editor_input_group_frame, editor_input_group_inset, editor_input_group_row,
    editor_input_value_text, editor_text_segment,
};
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

use super::super::model::AxisDragValueResetAction;

pub(super) struct AxisDragValueScrubFrameArgs {
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) focused: bool,
    pub(super) enabled: bool,
    pub(super) axis_label: Arc<str>,
    pub(super) axis_tint: Color,
    pub(super) value_text: Arc<str>,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) reset_action: Option<AxisDragValueResetAction>,
    pub(super) scrub_test_id: Option<Arc<str>>,
    pub(super) scrub_axis_test_id: Option<Arc<str>>,
    pub(super) scrub_value_test_id: Option<Arc<str>>,
    pub(super) scrub_prefix_test_id: Option<Arc<str>>,
    pub(super) scrub_suffix_test_id: Option<Arc<str>>,
    pub(super) scrub_reset_test_id: Option<Arc<str>>,
}

pub(super) fn axis_drag_value_scrub_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueScrubFrameArgs,
) -> AnyElement {
    let AxisDragValueScrubFrameArgs {
        density,
        frame_chrome,
        hovered,
        pressed,
        focused,
        enabled,
        axis_label,
        axis_tint,
        value_text,
        prefix,
        suffix,
        reset_action,
        scrub_test_id,
        scrub_axis_test_id,
        scrub_value_test_id,
        scrub_prefix_test_id,
        scrub_suffix_test_id,
        scrub_reset_test_id,
    } = args;

    let divider = frame_chrome.border;
    let mut scrub_frame = editor_input_group_frame(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                min_height: Some(Length::Px(density.row_height)),
                ..Default::default()
            },
            ..Default::default()
        },
        density,
        frame_chrome,
        EditorFrameState {
            enabled: true,
            hovered,
            pressed,
            focused,
            open: false,
            semantic: EditorFrameSemanticState::default(),
        },
        move |cx, visuals| {
            let affix_color = {
                let theme = Theme::global(&*cx.app);
                editor_muted_foreground(theme)
            };
            let mut axis =
                editor_axis_segment(cx, density, axis_label.clone(), axis_tint, visuals.bg);
            if let Some(test_id) = scrub_axis_test_id.as_ref() {
                axis = axis.test_id(test_id.clone()).a11y_label(axis_label.clone());
            }
            let sep = editor_input_group_divider(cx, divider);
            let value_text_el = editor_input_value_text(
                cx,
                density,
                frame_chrome.text_px,
                value_text.clone(),
                visuals.fg,
                Length::Fill,
            );
            let mut value = editor_input_group_inset(cx, frame_chrome.padding, value_text_el);
            if let Some(test_id) = scrub_value_test_id.as_ref() {
                value = value
                    .test_id(test_id.clone())
                    .a11y_label(value_text.clone());
            }

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
                if let Some(test_id) = scrub_prefix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(prefix);
                }
                segments.push(segment);
                segments.push(editor_input_group_divider(cx, divider));
            }
            segments.push(value);
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
                if let Some(test_id) = scrub_suffix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(suffix);
                }
                segments.push(segment);
            }
            if let Some(reset) = reset_action.clone() {
                segments.push(editor_input_group_divider(cx, divider));
                segments.push(editor_icon_button_segment(
                    cx,
                    density,
                    enabled,
                    reset.a11y_label.clone(),
                    reset.icon,
                    Some(Px(12.0)),
                    scrub_reset_test_id.clone(),
                    reset.on_activate.clone(),
                ));
            }

            vec![editor_input_group_row(cx, Px(0.0), segments)]
        },
    );
    if let Some(test_id) = scrub_test_id.as_ref() {
        scrub_frame = scrub_frame.test_id(test_id.clone());
    }
    scrub_frame
}
