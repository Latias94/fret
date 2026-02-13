use std::sync::Arc;

use fret_core::geometry::{Corners, Edges, Point, Px};
use fret_core::{Color, Transform2D};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, LayoutStyle, Length, Overflow,
    PointerRegionProps, RowProps, VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, UiHost};

#[derive(Debug, Clone)]
pub struct DockProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub gap: Px,
    pub corner_radii: Corners,
    pub background: Option<Color>,
    pub border: Edges,
    pub border_color: Option<Color>,
    pub item_size: Px,
    pub magnify: f32,
    pub influence_radius: Px,
}

impl Default for DockProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Px(Px(80.0));
        Self {
            layout,
            padding: Edges::all(Px(12.0)),
            gap: Px(10.0),
            corner_radii: Corners::all(Px(16.0)),
            background: None,
            border: Edges::all(Px(1.0)),
            border_color: None,
            item_size: Px(52.0),
            magnify: 0.8,
            influence_radius: Px(120.0),
        }
    }
}

#[derive(Default)]
struct DockModels {
    pointer_local: Option<Model<Option<Point>>>,
}

fn scale_about(center: Point, s: f32) -> Transform2D {
    Transform2D::translation(center)
        * Transform2D::scale_uniform(s)
        * Transform2D::translation(Point::new(Px(-center.x.0), Px(-center.y.0)))
}

fn dock_item_scale(pointer_x: Option<Px>, item_center_x: Px, magnify: f32, radius: Px) -> f32 {
    let Some(px) = pointer_x else {
        return 1.0;
    };
    let radius = radius.0;
    if !radius.is_finite() || radius <= 0.0 {
        return 1.0;
    }
    let d = (px.0 - item_center_x.0).abs();
    if !d.is_finite() {
        return 1.0;
    }
    let magnify = magnify.clamp(0.0, 8.0);
    let sigma = radius;
    let w = (-(d * d) / (2.0 * sigma * sigma)).exp();
    (1.0 + magnify * w).clamp(1.0, 16.0)
}

pub fn dock<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: DockProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let pointer_local = cx.with_state(DockModels::default, |st| st.pointer_local.clone());
    let pointer_local = match pointer_local {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None);
            cx.with_state(DockModels::default, |st| {
                st.pointer_local = Some(model.clone());
            });
            model
        }
    };

    let pointer_local_value = cx
        .get_model_copied(&pointer_local, Invalidation::Paint)
        .unwrap_or(None);

    let pointer_local_for_move = pointer_local.clone();
    let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, acx, mv| {
        let bounds = host.bounds();
        let local = Point::new(
            Px(mv.position.x.0 - bounds.origin.x.0),
            Px(mv.position.y.0 - bounds.origin.y.0),
        );

        let _ = host.update_model(&pointer_local_for_move, |p| {
            *p = Some(local);
        });
        host.notify(acx);
        host.invalidate(Invalidation::Paint);
        false
    });

    let pointer_local_for_cancel = pointer_local.clone();
    let on_cancel: fret_ui::action::OnPointerCancel = Arc::new(move |host, acx, _cancel| {
        let _ = host.update_model(&pointer_local_for_cancel, |p| {
            *p = None;
        });
        host.notify(acx);
        host.invalidate(Invalidation::Paint);
        false
    });

    let pointer_local_for_up = pointer_local;
    let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, acx, _up| {
        let _ = host.update_model(&pointer_local_for_up, |p| {
            *p = None;
        });
        host.notify(acx);
        host.invalidate(Invalidation::Paint);
        false
    });

    let outer = ContainerProps {
        layout: LayoutStyle {
            overflow: Overflow::Visible,
            ..props.layout
        },
        padding: props.padding,
        corner_radii: props.corner_radii,
        background: props.background,
        border: props.border,
        border_color: props.border_color,
        ..Default::default()
    };

    cx.container(outer, move |cx| {
        let mut hover_layout = LayoutStyle::default();
        hover_layout.size.width = Length::Fill;
        hover_layout.size.height = Length::Fill;

        vec![cx.hover_region(
            HoverRegionProps {
                layout: hover_layout,
            },
            move |cx, hovered| {
                let mut region_layout = LayoutStyle::default();
                region_layout.size.width = Length::Fill;
                region_layout.size.height = Length::Fill;

                let pointer_x = if hovered {
                    pointer_local_value.map(|p| p.x)
                } else {
                    None
                };

                [cx.pointer_region(
                    PointerRegionProps {
                        layout: region_layout,
                        enabled: true,
                    },
                    move |cx| {
                        cx.pointer_region_on_pointer_move(on_move);
                        cx.pointer_region_on_pointer_up(on_up);
                        cx.pointer_region_on_pointer_cancel(on_cancel);

                        let items: Vec<AnyElement> = children(cx).into_iter().collect();

                        let mut row_layout = LayoutStyle::default();
                        row_layout.size.width = Length::Fill;
                        row_layout.size.height = Length::Fill;

                        let padding_left = props.padding.left;
                        let item_size = props.item_size;
                        let gap = props.gap;
                        let magnify = props.magnify;
                        let influence_radius = props.influence_radius;

                        vec![cx.row(
                            RowProps {
                                layout: row_layout,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::Center,
                                align: fret_ui::element::CrossAlign::Center,
                            },
                            move |cx| {
                                let mut out = Vec::with_capacity(items.len());
                                for (i, child) in items.into_iter().enumerate() {
                                    let item_center_x = Px((padding_left.0)
                                        + (i as f32) * (item_size.0 + gap.0)
                                        + item_size.0 * 0.5);
                                    let s = dock_item_scale(
                                        pointer_x,
                                        item_center_x,
                                        magnify,
                                        influence_radius,
                                    );

                                    let center =
                                        Point::new(Px(item_size.0 * 0.5), Px(item_size.0 * 0.5));

                                    let mut item_layout = LayoutStyle::default();
                                    item_layout.size.width = Length::Px(item_size);
                                    item_layout.size.height = Length::Px(item_size);

                                    out.push(cx.keyed(i as u64, |cx| {
                                        cx.visual_transform_props(
                                            VisualTransformProps {
                                                layout: item_layout,
                                                transform: scale_about(center, s),
                                            },
                                            |_cx| vec![child],
                                        )
                                    }));
                                }
                                out
                            },
                        )]
                    },
                )]
            },
        )]
    })
}
