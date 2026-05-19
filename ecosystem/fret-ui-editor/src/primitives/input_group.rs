//! Editor input-group primitives (joined frame + segments).
//!
//! This is a policy-only helper for composing "joined" controls (axis markers, value fields,
//! small action icons) into a single input-like frame without style drift.

use std::sync::{Arc, Mutex};

use fret_core::{Color, Corners, Edges, MouseButton, Px};
use fret_ui::action::{ActionCx, OnActivate, OnPointerCancel, OnPointerDown, OnPointerUp};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, HoverRegionProps,
    LayoutStyle, Length, MainAlign, PointerRegionProps, PressableA11y, PressableProps, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ColorRef;

use super::EditorDensity;
use super::chrome::ResolvedEditorFrameChrome;
use super::colors::editor_foreground;
use super::icons::{editor_icon, editor_icon_with};
use super::readout::{
    editor_axis_marker_text_props, editor_input_segment_text_props, editor_input_value_text_props,
};
use super::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};
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
    pub(crate) semantic: Option<EditorFrameSemanticState>,
}

impl EditorInputGroupFrameOverrides {
    pub(crate) fn none() -> Self {
        Self {
            bg: None,
            border: None,
            semantic: None,
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
        layout.size.min_height = Some(Length::Px(density.row_height));
    }

    let mut state = state;
    if let Some(semantic) = overrides.semantic {
        state.semantic = semantic;
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
            padding: Edges::all(Px(0.0)).into(),
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

#[derive(Debug, Default)]
struct JoinedInputPointerState {
    pressed: bool,
    last_pointer_type: Option<fret_core::PointerType>,
}

pub(crate) struct EditorJoinedInputContents {
    pub(crate) root: AnyElement,
    pub(crate) focus_id: fret_ui::GlobalElementId,
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
    editor_joined_input_frame_segments_with_overrides(
        cx,
        layout,
        density,
        chrome,
        enabled_for_paint,
        open,
        frame_test_id,
        |_cx, focused| EditorInputGroupFrameOverrides {
            semantic: Some(EditorFrameSemanticState {
                typing: focused,
                invalid: false,
            }),
            ..EditorInputGroupFrameOverrides::none()
        },
        |_cx| Vec::new(),
        build_input,
        build_trailing_segments,
    )
}

pub(crate) fn editor_joined_input_frame_segments_with_overrides<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    enabled_for_paint: bool,
    open: bool,
    frame_test_id: Option<std::sync::Arc<str>>,
    frame_overrides: impl FnOnce(&mut ElementContext<'_, H>, bool) -> EditorInputGroupFrameOverrides,
    build_leading_segments: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
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
        frame_overrides,
        move |cx| {
            let divider = chrome.border;
            let mut segments = build_leading_segments(cx);
            let input = build_input(cx);
            let focus_id = input.id;
            let input = editor_input_group_inset(cx, chrome.padding, input);

            if !segments.is_empty() {
                segments.push(editor_input_group_divider(cx, divider));
            }
            segments.push(input);

            for seg in build_trailing_segments(cx) {
                segments.push(editor_input_group_divider(cx, divider));
                segments.push(seg);
            }

            EditorJoinedInputContents {
                root: editor_input_group_row(cx, Px(0.0), segments),
                focus_id,
            }
        },
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
    build_contents: impl FnOnce(&mut ElementContext<'_, H>) -> EditorJoinedInputContents,
) -> AnyElement {
    cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
        let pointer_state: Arc<Mutex<JoinedInputPointerState>> = cx.slot_state(
            || Arc::new(Mutex::new(JoinedInputPointerState::default())),
            |s| s.clone(),
        );

        // Best-effort cleanup for mouse: if the pointer is no longer hovering the region, do not
        // keep a stale "pressed" visual (e.g. pointer-up outside the region without capture).
        //
        // Touch/stylus interactions often do not produce reliable hover signals, so we avoid
        // clearing `pressed` solely based on hover for non-mouse pointer types.
        if !hovered
            && let Ok(mut st) = pointer_state.lock()
            && matches!(st.last_pointer_type, Some(fret_core::PointerType::Mouse))
        {
            st.pressed = false;
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

                let contents = build_contents(cx);
                let focused = cx.is_focused_element(contents.focus_id);
                let overrides = frame_overrides(cx, focused);

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
                        semantic: EditorFrameSemanticState::default(),
                    },
                    overrides,
                    move |_cx, _visuals| vec![contents.root],
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size, TextOverflow, TextWrap};
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;

    use super::*;

    fn test_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        )
    }

    #[test]
    fn editor_input_value_text_is_single_line_and_shrinkable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let color = fret_core::Color::from_srgb_hex_rgb(0xDD_EE_FF);

        let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
            editor_input_value_text(
                cx,
                EditorDensity::default(),
                Px(12.0),
                Arc::from("123456789.123456789"),
                color,
                Length::Fill,
            )
        });

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected editor_input_value_text(...) to build a Text element");
        };

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }
}
