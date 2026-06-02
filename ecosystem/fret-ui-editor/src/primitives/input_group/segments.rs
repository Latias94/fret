use std::sync::Arc;

use fret_core::{Color, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::colors::editor_foreground;
use crate::primitives::readout::{
    editor_axis_marker_text_props, editor_input_segment_text_props, editor_input_value_text_props,
};

mod icon;

pub(crate) use icon::{
    editor_clear_button_segment, editor_clear_button_segment_multiline, editor_icon_button_segment,
    editor_icon_segment,
};

pub(crate) fn editor_input_group_inset<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    padding: Edges,
    child: AnyElement,
) -> AnyElement {
    editor_input_group_segment(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            flex: FlexItemStyle {
                order: 0,
                grow: 1.0,
                shrink: 1.0,
                basis: Length::Px(Px(0.0)),
                align_self: None,
            },
            ..Default::default()
        },
        padding,
        child,
    )
}

pub(crate) fn editor_input_group_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    padding: Edges,
    child: AnyElement,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout,
            padding: padding.into(),
            ..Default::default()
        },
        move |_cx| vec![child],
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
            gap: SpacingLength::Px(gap),
            padding: Edges::all(Px(0.0)).into(),
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

pub(crate) fn editor_text_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    text_px: Px,
    text: Arc<str>,
    color: Color,
    padding: Edges,
) -> AnyElement {
    let text_el = cx.text_props(editor_input_segment_text_props(
        text,
        color,
        text_px,
        density.row_height,
    ));

    editor_input_group_segment(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        },
        padding,
        text_el,
    )
}

pub(crate) fn editor_input_value_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    text_px: Px,
    text: Arc<str>,
    color: Color,
    height: Length,
) -> AnyElement {
    cx.text_props(editor_input_value_text_props(
        text,
        color,
        text_px,
        density.row_height,
        height,
    ))
}

pub(crate) fn derived_test_id(base: Option<&Arc<str>>, suffix: &str) -> Option<Arc<str>> {
    base.map(|id| Arc::<str>::from(format!("{}.{}", id.as_ref(), suffix)))
}

pub(crate) fn editor_axis_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    label: std::sync::Arc<str>,
    tint: Color,
    bg: Color,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let fg = editor_foreground(theme);

    // Keep the axis marker subtle: it should read as part of the input group, not a standalone button.
    let seg_bg = mix(bg, Color { a: 0.16, ..tint }, 0.35);
    let seg_w = density.affordance_extent();

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
            vec![cx.text_props(editor_axis_marker_text_props(
                label.clone(),
                fg,
                density.row_height,
            ))]
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
