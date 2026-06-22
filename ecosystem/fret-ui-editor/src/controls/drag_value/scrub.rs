use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, editor_input_group_divider,
    editor_input_group_frame_with_overrides, editor_input_group_inset, editor_input_group_row,
    editor_input_value_text, editor_text_segment,
};
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

#[cfg(test)]
mod tests;

pub(super) struct DragValueScrubFrameArgs {
    pub(super) density: EditorDensity,
    pub(super) scrub_chrome: ResolvedEditorFrameChrome,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) focused: bool,
    pub(super) value_text: Arc<str>,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) scrub_test_id: Option<Arc<str>>,
    pub(super) prefix_test_id: Option<Arc<str>>,
    pub(super) suffix_test_id: Option<Arc<str>>,
    pub(super) value_test_id: Option<Arc<str>>,
}

pub(super) fn drag_value_scrub_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: DragValueScrubFrameArgs,
) -> AnyElement {
    let DragValueScrubFrameArgs {
        density,
        scrub_chrome,
        hovered,
        pressed,
        focused,
        value_text,
        prefix,
        suffix,
        scrub_test_id,
        prefix_test_id,
        suffix_test_id,
        value_test_id,
    } = args;

    let has_affixes = prefix.is_some() || suffix.is_some();
    let frame_overrides = if has_affixes {
        EditorInputGroupFrameOverrides::none()
    } else {
        EditorInputGroupFrameOverrides {
            padding: Some(scrub_chrome.padding),
            ..EditorInputGroupFrameOverrides::none()
        }
    };

    let mut scrub_frame = editor_input_group_frame_with_overrides(
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
        scrub_chrome,
        EditorFrameState {
            enabled: true,
            hovered,
            pressed,
            focused,
            open: false,
            semantic: EditorFrameSemanticState::default(),
        },
        frame_overrides,
        move |cx, visuals| {
            let value_text_el = editor_input_value_text(
                cx,
                density,
                scrub_chrome.text_px,
                value_text.clone(),
                visuals.fg,
                Length::Fill,
            );
            if !has_affixes {
                let mut value = value_text_el;
                if let Some(test_id) = value_test_id.as_ref() {
                    value = value
                        .test_id(test_id.clone())
                        .a11y_label(value_text.clone());
                }
                return vec![value];
            }

            let theme = Theme::global(&*cx.app);
            let affix_color = editor_muted_foreground(theme);
            let divider = scrub_chrome.border;
            let mut value = editor_input_group_inset(cx, scrub_chrome.padding, value_text_el);
            if let Some(test_id) = value_test_id.as_ref() {
                value = value
                    .test_id(test_id.clone())
                    .a11y_label(value_text.clone());
            }

            let mut segments = Vec::new();
            if let Some(prefix) = prefix.clone() {
                let mut segment = editor_text_segment(
                    cx,
                    density,
                    scrub_chrome.text_px,
                    prefix.clone(),
                    affix_color,
                    scrub_chrome.padding,
                );
                if let Some(test_id) = prefix_test_id.as_ref() {
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
                    scrub_chrome.text_px,
                    suffix.clone(),
                    affix_color,
                    scrub_chrome.padding,
                );
                if let Some(test_id) = suffix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(suffix);
                }
                segments.push(segment);
            }
            vec![editor_input_group_row(cx, Px(0.0), segments)]
        },
    );

    if let Some(test_id) = scrub_test_id.as_ref() {
        scrub_frame = scrub_frame.test_id(test_id.clone());
    }
    scrub_frame
}
