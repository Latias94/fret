//! Color-edit main swatch visual and tooltip-state owner.

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};

use super::super::ColorEditAlphaPreview;
use super::super::ColorEditTooltipOptions;
use super::super::popup::color_preview_stack;

pub(super) struct ColorSwatchVisualArgs {
    pub(super) open: Model<bool>,
    pub(super) tooltip_open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) current: Color,
    pub(super) alpha_preview: ColorEditAlphaPreview,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) swatch_size: Px,
    pub(super) enabled: bool,
    pub(super) popup_has_visible_content: bool,
    pub(super) tooltip_options: ColorEditTooltipOptions,
    pub(super) hovered: bool,
    pub(super) hovered_raw: bool,
    pub(super) pressed: bool,
    pub(super) focused: bool,
    pub(super) drop_over: bool,
}

pub(super) fn color_swatch_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorSwatchVisualArgs,
) -> AnyElement {
    let ColorSwatchVisualArgs {
        open,
        tooltip_open,
        copy_menu_open,
        current,
        alpha_preview,
        frame_chrome,
        swatch_size,
        enabled,
        popup_has_visible_content,
        tooltip_options,
        hovered,
        hovered_raw,
        pressed,
        focused,
        drop_over,
    } = args;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Paint)
        .unwrap_or(false);
    let copy_menu_is_open = cx
        .get_model_copied(&copy_menu_open, Invalidation::Paint)
        .unwrap_or(false);
    let tooltip_visible =
        tooltip_options.enabled && enabled && !is_open && !copy_menu_is_open && hovered_raw;
    let tooltip_open_now = cx
        .get_model_copied(&tooltip_open, Invalidation::Paint)
        .unwrap_or(false);
    if tooltip_open_now != tooltip_visible {
        let _ = cx
            .app
            .models_mut()
            .update(&tooltip_open, |value| *value = tooltip_visible);
    }

    let visuals = {
        let theme = Theme::global(&*cx.app);
        EditorWidgetVisuals::new(theme).frame_visuals(
            frame_chrome,
            EditorFrameState {
                enabled,
                hovered: hovered || hovered_raw,
                pressed: pressed || drop_over,
                focused,
                open: (is_open && popup_has_visible_content) || copy_menu_is_open,
                semantic: EditorFrameSemanticState::default(),
            },
        )
    };

    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(swatch_size),
                    height: Length::Px(swatch_size),
                    ..Default::default()
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            border: Edges::all(frame_chrome.border_width),
            border_color: Some(visuals.border),
            corner_radii: Corners::all(frame_chrome.radius),
            padding: Edges::all(frame_chrome.border_width).into(),
            ..Default::default()
        },
        move |cx| {
            vec![color_preview_stack(
                cx,
                current,
                frame_chrome.radius,
                alpha_preview,
            )]
        },
    )
}
