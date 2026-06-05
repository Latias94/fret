use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::ColorEditAlphaPreview;
use super::super::super::preview::color_preview_stack;

pub(super) struct PresetSwatchVisualArgs {
    pub(super) color: Color,
    pub(super) alpha_preview: ColorEditAlphaPreview,
    pub(super) active: bool,
    pub(super) ring: Color,
    pub(super) idle_border_color: Color,
}

pub(super) fn preset_swatch_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: PresetSwatchVisualArgs,
) -> AnyElement {
    let PresetSwatchVisualArgs {
        color,
        alpha_preview,
        active,
        ring,
        idle_border_color,
    } = args;
    let border_width = if active { Px(2.0) } else { Px(1.0) };
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            border: Edges::all(border_width),
            border_color: Some(if active { ring } else { idle_border_color }),
            corner_radii: Corners::all(Px(5.0)),
            padding: Edges::all(border_width).into(),
            ..Default::default()
        },
        move |cx| vec![color_preview_stack(cx, color, Px(5.0), alpha_preview)],
    )
}
