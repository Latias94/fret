//! Editor input-group primitives (joined frame + segments).
//!
//! This is a policy-only helper for composing "joined" controls (axis markers, value fields,
//! small action icons) into a single input-like frame without style drift.

use std::sync::{Arc, Mutex};

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Corners, Edges, MouseButton, Px, TextAlign, TextStyle};
use fret_ui::action::{ActionCx, OnActivate, OnPointerCancel, OnPointerDown, OnPointerUp};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, HoverRegionProps, LayoutStyle, Length,
    MainAlign, PointerRegionProps, PressableA11y, PressableProps, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ColorRef;

use super::EditorDensity;
use super::chrome::ResolvedEditorFrameChrome;
use super::icons::{editor_icon, editor_icon_with};
use super::visuals::{EditorFrameState, EditorWidgetVisuals};
use super::visuals::{editor_icon_button_bg, editor_icon_button_border};

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
            padding: Edges::all(Px(0.0)),
            background: Some(visuals.bg),
            border: Edges::all(chrome.border_width),
            border_color: Some(visuals.border),
            corner_radii: Corners::all(chrome.radius),
            ..Default::default()
        },
        move |cx| contents(cx, visuals),
    )
}

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
            padding,
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
    let mut el = cx.pressable(
        PressableProps {
            enabled: enabled_for_paint,
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(density.hit_thickness),
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
                move |cx| vec![editor_icon(cx, density, icon, icon_size)],
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
        Some(Px(12.0)),
        test_id,
        on_activate,
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
            padding: Edges::all(Px(0.0)),
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
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
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

#[derive(Debug, Default)]
struct JoinedInputPointerState {
    pressed: bool,
    last_pointer_type: Option<fret_core::PointerType>,
}

pub(crate) fn editor_joined_input_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    enabled_for_paint: bool,
    open: bool,
    frame_test_id: Option<std::sync::Arc<str>>,
    build_input: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    build_trailing_segments: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    editor_joined_input_frame_with_overrides(
        cx,
        layout,
        density,
        chrome,
        enabled_for_paint,
        open,
        frame_test_id,
        |_cx, _focused| EditorInputGroupFrameOverrides::none(),
        build_input,
        build_trailing_segments,
    )
}

pub(crate) fn editor_joined_input_frame_with_overrides<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    enabled_for_paint: bool,
    open: bool,
    frame_test_id: Option<std::sync::Arc<str>>,
    frame_overrides: impl FnOnce(&mut ElementContext<'_, H>, bool) -> EditorInputGroupFrameOverrides,
    build_input: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    build_trailing_segments: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
        let pointer_state: Arc<Mutex<JoinedInputPointerState>> = cx.with_state(
            || Arc::new(Mutex::new(JoinedInputPointerState::default())),
            |s| s.clone(),
        );

        // Best-effort cleanup for mouse: if the pointer is no longer hovering the region, do not
        // keep a stale "pressed" visual (e.g. pointer-up outside the region without capture).
        //
        // Touch/stylus interactions often do not produce reliable hover signals, so we avoid
        // clearing `pressed` solely based on hover for non-mouse pointer types.
        if !hovered {
            if let Ok(mut st) = pointer_state.lock() {
                if matches!(st.last_pointer_type, Some(fret_core::PointerType::Mouse)) {
                    st.pressed = false;
                }
            }
        }

        let pointer_state_down = pointer_state.clone();
        let on_down: OnPointerDown = Arc::new(move |host, action_cx: ActionCx, down| {
            // Only show the frame "pressed" state when the pointer-down hits the text input
            // surface, not when interacting with trailing segments (e.g. clear button).
            if !down.hit_is_text_input {
                return false;
            }
            if down.pointer_type == fret_core::PointerType::Mouse
                && down.button != MouseButton::Left
            {
                return false;
            }

            if let Ok(mut st) = pointer_state_down.lock() {
                st.pressed = true;
                st.last_pointer_type = Some(down.pointer_type);
            }
            host.invalidate(Invalidation::Paint);
            host.request_redraw(action_cx.window);
            false
        });

        let pointer_state_up = pointer_state.clone();
        let on_up: OnPointerUp = Arc::new(move |host, action_cx: ActionCx, _up| {
            if let Ok(mut st) = pointer_state_up.lock() {
                st.pressed = false;
                st.last_pointer_type = Some(_up.pointer_type);
            }
            host.invalidate(Invalidation::Paint);
            host.request_redraw(action_cx.window);
            false
        });

        let pointer_state_cancel = pointer_state.clone();
        let on_cancel: OnPointerCancel = Arc::new(move |host, action_cx: ActionCx, _cancel| {
            if let Ok(mut st) = pointer_state_cancel.lock() {
                st.pressed = false;
                st.last_pointer_type = Some(_cancel.pointer_type);
            }
            host.invalidate(Invalidation::Paint);
            host.request_redraw(action_cx.window);
            false
        });

        let pressed = pointer_state.lock().map(|s| s.pressed).unwrap_or(false);

        let root = cx.pointer_region(
            PointerRegionProps {
                layout: LayoutStyle::default(),
                enabled: enabled_for_paint,
                capture_phase_pointer_moves: false,
            },
            move |cx| {
                cx.pointer_region_add_on_pointer_down(on_down);
                cx.pointer_region_add_on_pointer_up(on_up);
                cx.pointer_region_on_pointer_cancel(on_cancel);

                let input = build_input(cx);
                let focused = cx.is_focused_element(input.id);
                let overrides = frame_overrides(cx, focused);

                let divider = chrome.border;
                let input = editor_input_group_inset(cx, chrome.padding, input);

                let mut segments = vec![input];
                for seg in build_trailing_segments(cx) {
                    segments.push(editor_input_group_divider(cx, divider));
                    segments.push(seg);
                }

                let mut frame = editor_input_group_frame_with_overrides(
                    cx,
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    density,
                    chrome,
                    EditorFrameState {
                        enabled: enabled_for_paint,
                        hovered,
                        pressed: enabled_for_paint && pressed,
                        focused,
                        open,
                    },
                    overrides,
                    move |cx, _visuals| vec![editor_input_group_row(cx, Px(0.0), segments)],
                );

                if let Some(test_id) = frame_test_id.as_ref() {
                    frame = frame.test_id(test_id.clone());
                }

                vec![frame]
            },
        );

        vec![root]
    })
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
