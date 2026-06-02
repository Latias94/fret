use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_inset, editor_input_group_row,
    editor_input_value_text, editor_text_segment,
};
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};

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

    let theme = Theme::global(&*cx.app);
    let visuals = EditorWidgetVisuals::new(theme).frame_visuals(
        scrub_chrome,
        EditorFrameState {
            enabled: true,
            hovered,
            pressed,
            focused,
            open: false,
            semantic: EditorFrameSemanticState::default(),
        },
    );

    let mut scrub_frame = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    min_height: Some(Length::Px(density.row_height)),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: scrub_chrome.padding.into(),
            background: Some(visuals.bg),
            border: Edges::all(scrub_chrome.border_width),
            border_color: Some(visuals.border),
            corner_radii: Corners::all(scrub_chrome.radius),
            ..Default::default()
        },
        move |cx| {
            let theme = Theme::global(&*cx.app);
            let affix_color = editor_muted_foreground(theme);
            let divider = visuals.border;
            let value_text_el = editor_input_value_text(
                cx,
                density,
                scrub_chrome.text_px,
                value_text.clone(),
                visuals.fg,
                Length::Auto,
            );
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
