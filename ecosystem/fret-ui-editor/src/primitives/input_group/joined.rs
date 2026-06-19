use std::sync::{Arc, Mutex};

use fret_core::{MouseButton, Px};
use fret_ui::action::{ActionCx, OnPointerCancel, OnPointerDown, OnPointerUp};
use fret_ui::element::{
    AnyElement, HoverRegionProps, LayoutStyle, Length, PointerRegionProps, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

use super::{
    EditorInputGroupFrameOverrides, editor_input_group_divider,
    editor_input_group_frame_with_overrides, editor_input_group_row,
};

#[derive(Debug, Default)]
struct JoinedInputPointerState {
    pressed: bool,
    last_pointer_type: Option<fret_core::PointerType>,
}

#[derive(Debug)]
struct EditorJoinedInputContents {
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
            let mut segments = build_leading_segments(cx);
            let input = build_input(cx);
            let focus_id = input.id;
            let trailing_segments = build_trailing_segments(cx);

            let root = if segments.is_empty() && trailing_segments.is_empty() {
                input
            } else {
                let divider = chrome.border;

                if !segments.is_empty() {
                    segments.push(editor_input_group_divider(cx, divider));
                }
                segments.push(input);

                for seg in trailing_segments {
                    segments.push(editor_input_group_divider(cx, divider));
                    segments.push(seg);
                }

                editor_input_group_row(cx, Px(0.0), segments)
            };

            EditorJoinedInputContents { root, focus_id }
        },
    )
}

fn editor_joined_input_frame_with_overrides<H: UiHost>(
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
