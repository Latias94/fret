use fret_core::{Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::super::picker_border_and_ring;
use super::super::preview::{alpha_bar_preview_stack, vertical_alpha_bar_preview_stack};

pub(super) fn vertical_alpha_bar_surface<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focused: bool,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    alpha_bar_surface_frame(cx, focused, move |cx| {
        vec![vertical_alpha_bar_preview_stack(cx, rgb, alpha)]
    })
}

pub(super) fn alpha_bar_surface<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focused: bool,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    alpha_bar_surface_frame(cx, focused, move |cx| {
        vec![alpha_bar_preview_stack(cx, rgb, alpha)]
    })
}

fn alpha_bar_surface_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focused: bool,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement> + 'static,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let (border, ring) = picker_border_and_ring(theme);
    let border_color = if focused { ring } else { border };

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
            border: Edges::all(Px(1.0)),
            border_color: Some(border_color),
            corner_radii: Corners::all(Px(4.0)),
            padding: Edges::all(Px(1.0)).into(),
            ..Default::default()
        },
        children,
    )
}
