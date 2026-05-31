//! Editor input-group primitives (joined frame + segments).
//!
//! This is a policy-only helper for composing "joined" controls (axis markers, value fields,
//! small action icons) into a single input-like frame without style drift.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::ColorRef;

use super::EditorDensity;
use super::chrome::ResolvedEditorFrameChrome;
use super::colors::editor_foreground;
use super::icons::{editor_icon, editor_icon_with};
use super::readout::{
    editor_axis_marker_text_props, editor_input_segment_text_props, editor_input_value_text_props,
};
use super::visuals::{editor_icon_button_bg, editor_icon_button_border};

mod frame;
pub(crate) use frame::{
    EditorInputGroupFrameOverrides, editor_input_group_frame,
    editor_input_group_frame_with_overrides,
};
mod joined;
#[allow(unused_imports)]
pub(crate) use joined::{
    EditorJoinedInputContents, editor_joined_input_frame,
    editor_joined_input_frame_segments_with_overrides, editor_joined_input_frame_with_overrides,
};

#[cfg(test)]
mod tests;

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

pub(crate) fn editor_icon_button_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled_for_paint: bool,
    a11y_label: std::sync::Arc<str>,
    icon: fret_icons::IconId,
    icon_size: Option<Px>,
    test_id: Option<std::sync::Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    let affordance_extent = density.affordance_extent();

    let mut el = cx.pressable(
        PressableProps {
            enabled: enabled_for_paint,
            focusable: false,
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(affordance_extent),
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(a11y_label),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());

            let theme = Theme::global(&*cx.app);
            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;
            let bg = editor_icon_button_bg(theme, enabled_for_paint, hovered, pressed);
            let border = editor_icon_button_border(theme, enabled_for_paint, hovered, pressed);
            let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: bg,
                    border: Edges::all(border_width),
                    border_color: border,
                    corner_radii: Corners::all(Px(0.0)),
                    ..Default::default()
                },
                move |cx| {
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
                            direction: fret_core::Axis::Horizontal,
                            gap: SpacingLength::Px(Px(0.0)),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| vec![editor_icon(cx, density, icon, icon_size)],
                    )]
                },
            )]
        },
    );

    if let Some(test_id) = test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }

    el
}

pub(crate) fn editor_clear_button_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled_for_paint: bool,
    a11y_label: std::sync::Arc<str>,
    test_id: Option<std::sync::Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    editor_icon_button_segment(
        cx,
        density,
        enabled_for_paint,
        a11y_label,
        fret_icons::ids::ui::CLOSE,
        Some(Px(11.0)),
        test_id,
        on_activate,
    )
}

pub(crate) fn editor_clear_button_segment_multiline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    enabled_for_paint: bool,
    a11y_label: std::sync::Arc<str>,
    test_id: Option<std::sync::Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    let affordance_extent = density.affordance_extent();
    let button = editor_clear_button_segment(
        cx,
        density,
        enabled_for_paint,
        a11y_label,
        test_id,
        on_activate,
    );

    editor_input_group_segment(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Px(affordance_extent),
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        },
        Edges {
            top: chrome.padding.top,
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
        button,
    )
}

pub(crate) fn editor_icon_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    icon: fret_icons::IconId,
    icon_size: Option<Px>,
    color: Option<ColorRef>,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(density.hit_thickness),
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        move |cx| {
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
                    direction: fret_core::Axis::Horizontal,
                    gap: SpacingLength::Px(Px(0.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    vec![editor_icon_with(
                        cx,
                        density,
                        icon,
                        icon_size,
                        color.clone(),
                    )]
                },
            )]
        },
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
