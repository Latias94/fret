use std::sync::Arc;

use fret_core::geometry::{Corners, Edges, Point, Size};
use fret_core::scene::{ColorSpace, GradientStop, Paint, RadialGradient, TileMode, MAX_STOPS};
use fret_core::{Color, Px};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, PointerRegionProps};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::collect_children;

#[derive(Debug, Clone)]
pub struct MagicCardProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub highlight: Color,
    pub highlight_radius: Px,
    pub border: Edges,
    pub border_base: Color,
    pub border_highlight: Color,
    pub border_highlight_radius: Px,
}

impl Default for MagicCardProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            highlight: Color {
                a: 0.35,
                ..Color::from_srgb_hex_rgb(0xe6_e6_ff)
            },
            highlight_radius: Px(180.0),
            border: Edges::all(Px(1.0)),
            border_base: Color {
                a: 0.12,
                ..Color::from_srgb_hex_rgb(0xff_ff_ff)
            },
            border_highlight: Color {
                a: 0.55,
                ..Color::from_srgb_hex_rgb(0xf2_f2_ff)
            },
            border_highlight_radius: Px(220.0),
        }
    }
}

fn radial(center: Point, radius: Px, inner: Color, outer: Color) -> Paint {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, inner);
    stops[1] = GradientStop::new(1.0, outer);
    Paint::RadialGradient(RadialGradient {
        center,
        radius: Size::new(radius, radius),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: 2,
        stops,
    })
}

#[track_caller]
pub fn magic_card<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    props: MagicCardProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: fret_ui_kit::IntoUiElement<H>,
{
    let pointer_local = cx.local_model(|| None::<Point>);

    let pointer_local_value = cx
        .get_model_copied(&pointer_local, Invalidation::Paint)
        .unwrap_or(None);

    let background_paint = if let Some(p) = pointer_local_value {
        radial(p, props.highlight_radius, props.highlight, props.base)
    } else {
        Paint::Solid(props.base)
    };

    let border_paint = if let Some(p) = pointer_local_value {
        radial(
            p,
            props.border_highlight_radius,
            props.border_highlight,
            props.border_base,
        )
    } else {
        Paint::Solid(props.border_base)
    };

    let pointer_local_for_move = pointer_local.clone();
    let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _acx, mv| {
        let local = mv.position_local;

        let _ = host.update_model(&pointer_local_for_move, |p| {
            *p = Some(local);
        });
        host.invalidate(Invalidation::Paint);
        false
    });

    let pointer_local_for_clear = pointer_local.clone();
    let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, _acx, _up| {
        let _ = host.update_model(&pointer_local_for_clear, |p| {
            *p = None;
        });
        host.invalidate(Invalidation::Paint);
        false
    });

    let pointer_local_for_cancel = pointer_local;
    let on_cancel: fret_ui::action::OnPointerCancel = Arc::new(move |host, _acx, _cancel| {
        let _ = host.update_model(&pointer_local_for_cancel, |p| {
            *p = None;
        });
        host.invalidate(Invalidation::Paint);
        false
    });

    cx.pointer_region(
        PointerRegionProps {
            layout: props.layout,
            enabled: true,
            ..Default::default()
        },
        move |cx| {
            cx.pointer_region_on_pointer_move(on_move);
            cx.pointer_region_on_pointer_up(on_up);
            cx.pointer_region_on_pointer_cancel(on_cancel);

            let mut container_layout = LayoutStyle::default();
            container_layout.size.width = fret_ui::element::Length::Fill;
            container_layout.size.height = fret_ui::element::Length::Fill;

            let body = cx.container(
                ContainerProps {
                    layout: container_layout,
                    padding: props.padding.into(),
                    background: None,
                    background_paint: Some(background_paint),
                    shadow: None,
                    border: props.border,
                    border_color: None,
                    border_paint: Some(border_paint),
                    border_dash: None,
                    focus_ring: None,
                    focus_ring_always_paint: false,
                    focus_border_color: None,
                    focus_within: false,
                    corner_radii: props.corner_radii,
                    snap_to_device_pixels: false,
                },
                move |cx| {
                    let items = children(cx);
                    collect_children(cx, items)
                },
            );

            vec![body]
        },
    )
}
