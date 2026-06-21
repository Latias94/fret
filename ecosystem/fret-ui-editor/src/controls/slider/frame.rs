use std::sync::Arc;

use fret_core::text::TextOverflow;
use fret_core::{Axis, Edges, Px, TextAlign};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_frame, editor_input_group_segment,
};
use crate::primitives::readout::EditorCompactReadoutStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

use super::chrome::{
    ResolvedSliderGeometry, ResolvedSliderPaint, slider_thumb_props, slider_track_flex_props,
    slider_track_segment_props,
};

pub(super) struct SliderFrameArgs {
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) geometry: ResolvedSliderGeometry,
    pub(super) paint: ResolvedSliderPaint,
    pub(super) t: f32,
    pub(super) interactive_enabled: bool,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) focused: bool,
    pub(super) show_value: bool,
    pub(super) value_width: Px,
    pub(super) value_display_text: Arc<str>,
    pub(super) value_display_test_id: Option<Arc<str>>,
}

pub(super) fn slider_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: SliderFrameArgs,
) -> AnyElement {
    let SliderFrameArgs {
        density,
        frame_chrome,
        geometry,
        paint,
        t,
        interactive_enabled,
        hovered,
        pressed,
        focused,
        show_value,
        value_width,
        value_display_text,
        value_display_test_id,
    } = args;

    let readout_style = {
        let theme = Theme::global(&*cx.app);
        EditorCompactReadoutStyle::resolve(theme, density.row_height)
    };
    let left_grow = t.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    let track_bg = paint.track_bg;
    let fill_bg = paint.fill_bg;

    editor_input_group_frame(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                min_height: Some(Length::Px(density.row_height)),
                ..Default::default()
            },
            overflow: Overflow::Clip,
            ..Default::default()
        },
        density,
        frame_chrome,
        EditorFrameState {
            enabled: interactive_enabled,
            hovered,
            pressed,
            focused,
            open: false,
            semantic: EditorFrameSemanticState::default(),
        },
        move |cx, frame_visuals| {
            let track = cx.flex(slider_track_flex_props(frame_chrome.padding), move |cx| {
                let mut seg_layout = |grow: f32, bg: fret_core::Color, left: bool| {
                    cx.container(
                        slider_track_segment_props(geometry, grow, bg, left),
                        |_cx| vec![],
                    )
                };

                let left = seg_layout(left_grow, fill_bg, true);
                let right = seg_layout(right_grow, track_bg, false);

                let thumb = cx.container(slider_thumb_props(geometry, paint), |_cx| vec![]);

                vec![left, thumb, right]
            });

            let value_el = if show_value {
                let mut value_text_el = cx.text_props(readout_style.text_props(
                    value_display_text.clone(),
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    TextAlign::End,
                    TextOverflow::Clip,
                ));
                if let Some(test_id) = value_display_test_id.as_ref() {
                    value_text_el = value_text_el
                        .test_id(test_id.clone())
                        .a11y_label(value_display_text.clone());
                }

                let value_seg = editor_input_group_segment(
                    cx,
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(value_width),
                            height: Length::Fill,
                            ..Default::default()
                        },
                        flex: FlexItemStyle {
                            order: 0,
                            grow: 0.0,
                            shrink: 0.0,
                            basis: Length::Px(value_width),
                            align_self: None,
                        },
                        ..Default::default()
                    },
                    frame_chrome.padding,
                    value_text_el,
                );
                Some(value_seg)
            } else {
                None
            };

            let mut children = vec![track];
            if let Some(value_el) = value_el {
                children.push(editor_input_group_divider(cx, frame_visuals.border));
                children.push(value_el);
            }

            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(Px(0.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| children,
            )]
        },
    )
}
