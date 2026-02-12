use std::sync::Arc;

use fret_core::geometry::{Corners, Edges, Point, Px, Size};
use fret_core::scene::{ColorSpace, GradientStop, MAX_STOPS, Mask, RadialGradient, TileMode};
use fret_core::{Color, Transform2D};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{
    AnyElement, ContainerProps, FocusTraversalGateProps, HitTestGateProps, HoverRegionProps,
    InsetStyle, LayoutStyle, Length, Overflow, PointerRegionProps, PositionStyle, SizeStyle,
    VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, UiHost};

#[derive(Debug, Clone)]
pub struct LensProps {
    pub layout: LayoutStyle,
    /// Lens diameter (logical pixels).
    pub lens_size: Px,
    /// Zoom factor; must be >= 1.0.
    pub zoom_factor: f32,
    /// Whether the lens is always visible (independent of hover).
    pub is_static: bool,
    /// Static lens center position (element-local px).
    pub position: Point,
    /// When provided, this position is used while not hovered (element-local px).
    pub default_position: Option<Point>,
    /// Rounded clipping for the container hosting the content.
    pub corner_radii: Corners,
    /// Optional visual padding on the content (applied to both base + zoomed copies).
    pub padding: Edges,
    /// Mask edge feather (logical pixels).
    pub feather: Px,
}

impl Default for LensProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            lens_size: Px(170.0),
            zoom_factor: 1.3,
            is_static: false,
            position: Point::new(Px(0.0), Px(0.0)),
            default_position: None,
            corner_radii: Corners::all(Px(12.0)),
            padding: Edges::all(Px(0.0)),
            feather: Px(8.0),
        }
    }
}

#[derive(Default)]
struct LensModels {
    pointer_local: Option<Model<Option<Point>>>,
}

fn radial_mask(center: Point, lens_size: Px, feather: Px) -> Mask {
    let radius = Px((lens_size.0 * 0.5).max(0.0));
    let opaque = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let transparent = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0,
    };

    let r = radius.0.max(1e-6);
    let feather = feather.0.clamp(0.0, r);
    let edge = ((r - feather) / r).clamp(0.0, 1.0);

    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, opaque);
    stops[1] = GradientStop::new(edge, opaque);
    stops[2] = GradientStop::new(1.0, transparent);

    Mask::radial_gradient(RadialGradient {
        center,
        radius: Size::new(radius, radius),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: 3,
        stops,
    })
}

fn zoom_about(center: Point, scale: f32) -> Transform2D {
    let to_center = Transform2D::translation(center);
    let from_center = Transform2D::translation(Point::new(Px(-center.x.0), Px(-center.y.0)));
    to_center * Transform2D::scale_uniform(scale) * from_center
}

pub fn lens<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: LensProps,
    mut children: impl FnMut(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let pointer_local = cx.with_state(LensModels::default, |st| st.pointer_local.clone());
    let pointer_local = match pointer_local {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None);
            cx.with_state(LensModels::default, |st| {
                st.pointer_local = Some(model.clone());
            });
            model
        }
    };

    let pointer_local_value = cx
        .get_model_copied(&pointer_local, Invalidation::Paint)
        .unwrap_or(None);

    let pointer_local_for_move = pointer_local.clone();
    let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _acx, mv| {
        let bounds = host.bounds();
        let local = Point::new(
            Px(mv.position.x.0 - bounds.origin.x.0),
            Px(mv.position.y.0 - bounds.origin.y.0),
        );

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

    cx.hover_region(
        HoverRegionProps {
            layout: props.layout,
        },
        move |cx, hovered| {
            let mut fill_layout = LayoutStyle::default();
            fill_layout.size.width = Length::Fill;
            fill_layout.size.height = Length::Fill;

            vec![cx.pointer_region(
                PointerRegionProps {
                    layout: fill_layout,
                    enabled: true,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    cx.pointer_region_on_pointer_cancel(on_cancel);

                    let mut container_layout = LayoutStyle::default();
                    container_layout.size.width = Length::Fill;
                    container_layout.size.height = Length::Fill;
                    container_layout.overflow = Overflow::Clip;

                    let container_props = ContainerProps {
                        layout: container_layout,
                        padding: props.padding,
                        corner_radii: props.corner_radii,
                        ..Default::default()
                    };

                    vec![cx.container(container_props, |cx| {
                        let mut out: Vec<AnyElement> = children(cx).into_iter().collect::<Vec<_>>();

                        let position = if props.is_static {
                            Some(props.position)
                        } else if let Some(default_pos) =
                            props.default_position.filter(|_| !hovered)
                        {
                            Some(default_pos)
                        } else if hovered {
                            pointer_local_value
                        } else {
                            None
                        };

                        if let Some(position) = position {
                            // NOTE: This duplicates the subtree (base + zoomed copy). This is
                            // intended for visual content; do not use with interactive children in
                            // Phase 0.
                            let mask = radial_mask(position, props.lens_size, props.feather);

                            let overlay_layout = LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    left: Some(Px(0.0)),
                                },
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let overlay = cx.focus_traversal_gate_props(
                                FocusTraversalGateProps {
                                    layout: overlay_layout,
                                    traverse: false,
                                },
                                |cx| {
                                    let mut hit_layout = LayoutStyle::default();
                                    hit_layout.size.width = Length::Fill;
                                    hit_layout.size.height = Length::Fill;

                                    [cx.hit_test_gate_props(
                                        HitTestGateProps {
                                            layout: hit_layout,
                                            hit_test: false,
                                        },
                                        |cx| {
                                            let mut mask_layout = LayoutStyle::default();
                                            mask_layout.size.width = Length::Fill;
                                            mask_layout.size.height = Length::Fill;

                                            [cx.mask_layer_props(
                                                fret_ui::element::MaskLayerProps {
                                                    layout: mask_layout,
                                                    mask,
                                                },
                                                |cx| {
                                                    let mut zoom_layout = LayoutStyle::default();
                                                    zoom_layout.size.width = Length::Fill;
                                                    zoom_layout.size.height = Length::Fill;

                                                    let transform =
                                                        zoom_about(position, props.zoom_factor);
                                                    [cx.visual_transform_props(
                                                        VisualTransformProps {
                                                            layout: zoom_layout,
                                                            transform,
                                                        },
                                                        |cx| children(cx),
                                                    )]
                                                },
                                            )]
                                        },
                                    )]
                                },
                            );

                            out.push(overlay);
                        }

                        out
                    })]
                },
            )]
        },
    )
}
