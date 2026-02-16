//! Editor input-group primitives (joined frame + segments).
//!
//! This is a policy-only helper for composing "joined" controls (axis markers, value fields,
//! small action icons) into a single input-like frame without style drift.

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Corners, Edges, Px, TextAlign, TextStyle};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use super::EditorDensity;
use super::chrome::ResolvedEditorFrameChrome;
use super::visuals::{EditorFrameState, EditorWidgetVisuals};

pub(crate) fn editor_input_group_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    state: EditorFrameState,
    contents: impl FnOnce(
        &mut ElementContext<'_, H>,
        super::visuals::EditorFrameVisuals,
    ) -> Vec<AnyElement>,
) -> AnyElement {
    editor_input_group_frame_with_overrides(
        cx,
        layout,
        density,
        chrome,
        state,
        EditorInputGroupFrameOverrides::none(),
        contents,
    )
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorInputGroupFrameOverrides {
    pub(crate) bg: Option<Color>,
    pub(crate) border: Option<Color>,
}

impl EditorInputGroupFrameOverrides {
    pub(crate) fn none() -> Self {
        Self {
            bg: None,
            border: None,
        }
    }
}

pub(crate) fn editor_input_group_frame_with_overrides<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    state: EditorFrameState,
    overrides: EditorInputGroupFrameOverrides,
    contents: impl FnOnce(
        &mut ElementContext<'_, H>,
        super::visuals::EditorFrameVisuals,
    ) -> Vec<AnyElement>,
) -> AnyElement {
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(density.row_height);
    }

    let theme = Theme::global(&*cx.app);
    let mut visuals = EditorWidgetVisuals::new(theme).frame_visuals(chrome, state);
    if let Some(bg) = overrides.bg {
        visuals.bg = bg;
    }
    if let Some(border) = overrides.border {
        visuals.border = border;
    }

    cx.container(
        ContainerProps {
            layout,
            padding: chrome.padding,
            background: Some(visuals.bg),
            border: Edges::all(chrome.border_width),
            border_color: Some(visuals.border),
            corner_radii: Corners::all(chrome.radius),
            ..Default::default()
        },
        move |cx| contents(cx, visuals),
    )
}

pub(crate) fn editor_input_group_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    gap: Px,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| children,
    )
}

pub(crate) fn editor_input_group_divider<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(1.0)),
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            background: Some(color),
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}

pub(crate) fn editor_axis_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    label: std::sync::Arc<str>,
    tint: Color,
    bg: Color,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let fg = theme.color_token("foreground");

    // Keep the axis marker subtle: it should read as part of the input group, not a standalone button.
    let seg_bg = mix(bg, Color { a: 0.16, ..tint }, 0.35);
    let seg_w = Px(density.row_height.0.max(density.hit_thickness.0));

    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(seg_w),
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            background: Some(seg_bg),
            ..Default::default()
        },
        move |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: label.clone(),
                style: Some(TextStyle {
                    size: Px(11.0),
                    weight: fret_core::FontWeight::SEMIBOLD,
                    line_height: Some(density.row_height),
                    ..Default::default()
                }),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Center,
            })]
        },
    )
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}
