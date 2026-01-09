use super::super::ElementHostWidget;
use crate::declarative::frame::layout_style_for_node;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;
use crate::declarative::taffy_layout::*;
use crate::layout_constraints::{AvailableSpace as RuntimeAvailableSpace, LayoutSize};

#[cfg(feature = "layout-engine-v2")]
use crate::layout_engine::{
    ParentLayoutKind, build_flow_subtree, layout_children_from_engine_if_solved,
};

#[cfg(not(feature = "layout-engine-v2"))]
use crate::declarative::frame::{ElementInstance, element_record_for_node};

#[cfg(not(feature = "layout-engine-v2"))]
use crate::layout_constraints::LayoutConstraints;

impl ElementHostWidget {
    pub(super) fn layout_flex_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
        #[cfg(feature = "layout-engine-v2")]
        {
            self.layout_flex_impl_engine(cx, window, props)
        }

        #[cfg(not(feature = "layout-engine-v2"))]
        {
            let pad_left = props.padding.left.0.max(0.0);
            let pad_right = props.padding.right.0.max(0.0);
            let pad_top = props.padding.top.0.max(0.0);
            let pad_bottom = props.padding.bottom.0.max(0.0);
            let pad_w = pad_left + pad_right;
            let pad_h = pad_top + pad_bottom;
            let inner_origin = fret_core::Point::new(
                Px(cx.bounds.origin.x.0 + pad_left),
                Px(cx.bounds.origin.y.0 + pad_top),
            );
            let outer_avail_w = match props.layout.size.width {
                Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
                Length::Fill | Length::Auto => cx.available.width,
            };
            let outer_avail_h = match props.layout.size.height {
                Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
                Length::Fill | Length::Auto => cx.available.height,
            };

            let inner_avail = Size::new(
                Px((outer_avail_w.0 - pad_w).max(0.0)),
                Px((outer_avail_h.0 - pad_h).max(0.0)),
            );

            let root_style = TaffyStyle {
                display: Display::Flex,
                flex_direction: match props.direction {
                    fret_core::Axis::Horizontal => FlexDirection::Row,
                    fret_core::Axis::Vertical => FlexDirection::Column,
                },
                flex_wrap: if props.wrap {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::NoWrap
                },
                justify_content: Some(taffy_justify(props.justify)),
                align_items: Some(taffy_align_items(props.align)),
                gap: TaffySize {
                    width: LengthPercentage::length(props.gap.0.max(0.0)),
                    height: LengthPercentage::length(props.gap.0.max(0.0)),
                },
                size: TaffySize {
                    width: match props.layout.size.width {
                        Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                        Length::Fill => Dimension::length(inner_avail.width.0.max(0.0)),
                        Length::Auto => Dimension::auto(),
                    },
                    height: match props.layout.size.height {
                        Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                        Length::Fill => Dimension::length(inner_avail.height.0.max(0.0)),
                        Length::Auto => Dimension::auto(),
                    },
                },
                max_size: TaffySize {
                    width: Dimension::length(inner_avail.width.0.max(0.0)),
                    height: Dimension::length(inner_avail.height.0.max(0.0)),
                },
                ..Default::default()
            };

            let cache = self
                .flex_cache
                .get_or_insert_with(TaffyContainerCache::default);

            cache.sync_root_style(root_style);
            cache.sync_children(cx.children, |child| {
                let layout_style = layout_style_for_node(cx.app, window, child);
                let spacer_min = element_record_for_node(cx.app, window, child).and_then(|r| {
                    if let ElementInstance::Spacer(p) = r.instance {
                        Some(p.min)
                    } else {
                        None
                    }
                });

                let mut min_w = layout_style.size.min_width.map(|p| p.0);
                let mut min_h = layout_style.size.min_height.map(|p| p.0);
                if let Some(min) = spacer_min {
                    let min = min.0.max(0.0);
                    match props.direction {
                        fret_core::Axis::Horizontal => {
                            min_w = Some(min_w.unwrap_or(0.0).max(min));
                        }
                        fret_core::Axis::Vertical => {
                            min_h = Some(min_h.unwrap_or(0.0).max(min));
                        }
                    }
                }

                TaffyStyle {
                    display: Display::Block,
                    position: taffy_position(layout_style.position),
                    inset: taffy_rect_lpa_from_inset(layout_style.position, layout_style.inset),
                    size: TaffySize {
                        width: taffy_dimension(layout_style.size.width),
                        height: taffy_dimension(layout_style.size.height),
                    },
                    aspect_ratio: layout_style.aspect_ratio,
                    min_size: TaffySize {
                        width: min_w.map(Dimension::length).unwrap_or_else(Dimension::auto),
                        height: min_h.map(Dimension::length).unwrap_or_else(Dimension::auto),
                    },
                    max_size: TaffySize {
                        width: layout_style
                            .size
                            .max_width
                            .map(|p| Dimension::length(p.0))
                            .unwrap_or_else(Dimension::auto),
                        height: layout_style
                            .size
                            .max_height
                            .map(|p| Dimension::length(p.0))
                            .unwrap_or_else(Dimension::auto),
                    },
                    margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
                    flex_grow: layout_style.flex.grow.max(0.0),
                    flex_shrink: layout_style.flex.shrink.max(0.0),
                    flex_basis: taffy_dimension(layout_style.flex.basis),
                    align_self: layout_style.flex.align_self.map(taffy_align_self),
                    ..Default::default()
                }
            });

            cache
                .taffy
                .mark_dirty(cache.root)
                .expect("taffy mark dirty");

            cache.measure_cache.clear();
            cache
                .measure_cache
                .reserve(cache.children.len().saturating_mul(4));
            let root = cache.root;

            {
                let measure_cache = &mut cache.measure_cache;
                let taffy = &mut cache.taffy;

                let available = taffy::geometry::Size {
                    width: TaffyAvailableSpace::Definite(inner_avail.width.0),
                    height: TaffyAvailableSpace::Definite(inner_avail.height.0),
                };

                taffy
                    .compute_layout_with_measure(
                        root,
                        available,
                        |known, avail, _id, ctx, _style| {
                            let Some(child) = ctx.and_then(|c| *c) else {
                                return taffy::geometry::Size::default();
                            };

                            let key = TaffyMeasureKey {
                                child,
                                known_w: known.width.map(|v| v.to_bits()),
                                known_h: known.height.map(|v| v.to_bits()),
                                avail_w: taffy_available_space_key(avail.width),
                                avail_h: taffy_available_space_key(avail.height),
                            };
                            if let Some(size) = measure_cache.get(&key) {
                                return *size;
                            }

                            let constraints = LayoutConstraints::new(
                                LayoutSize::new(known.width.map(Px), known.height.map(Px)),
                                LayoutSize::new(
                                    match avail.width {
                                        TaffyAvailableSpace::Definite(w) => {
                                            RuntimeAvailableSpace::Definite(Px(w))
                                        }
                                        TaffyAvailableSpace::MinContent => {
                                            RuntimeAvailableSpace::MinContent
                                        }
                                        TaffyAvailableSpace::MaxContent => {
                                            RuntimeAvailableSpace::MaxContent
                                        }
                                    },
                                    match avail.height {
                                        TaffyAvailableSpace::Definite(h) => {
                                            RuntimeAvailableSpace::Definite(Px(h))
                                        }
                                        TaffyAvailableSpace::MinContent => {
                                            RuntimeAvailableSpace::MinContent
                                        }
                                        TaffyAvailableSpace::MaxContent => {
                                            RuntimeAvailableSpace::MaxContent
                                        }
                                    },
                                ),
                            );

                            let s = cx.measure_in(child, constraints);
                            let out = taffy::geometry::Size {
                                width: s.width.0,
                                height: s.height.0,
                            };
                            measure_cache.insert(key, out);
                            out
                        },
                    )
                    .expect("taffy compute");
            }

            let taffy = &cache.taffy;
            let root_layout = taffy.layout(root).expect("taffy root layout");
            let container_inner_size = Size::new(
                Px(root_layout.size.width.max(0.0)),
                Px(root_layout.size.height.max(0.0)),
            );
            let auto_margin_inner_size = Size::new(
                match props.layout.size.width {
                    Length::Fill => inner_avail.width,
                    _ => container_inner_size.width,
                },
                match props.layout.size.height {
                    Length::Fill => inner_avail.height,
                    _ => container_inner_size.height,
                },
            );

            for &child_node in &cache.child_nodes {
                let layout = taffy.layout(child_node).expect("taffy layout");
                let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                    continue;
                };
                let child_style = layout_style_for_node(cx.app, window, child);
                let single_child = cx.children.len() == 1;

                let mut x = layout.location.x;
                let mut y = layout.location.y;

                let margin_left_auto =
                    matches!(child_style.margin.left, crate::element::MarginEdge::Auto);
                let margin_right_auto =
                    matches!(child_style.margin.right, crate::element::MarginEdge::Auto);
                let margin_top_auto =
                    matches!(child_style.margin.top, crate::element::MarginEdge::Auto);
                let margin_bottom_auto =
                    matches!(child_style.margin.bottom, crate::element::MarginEdge::Auto);

                let margin_px = |edge: crate::element::MarginEdge| match edge {
                    crate::element::MarginEdge::Px(px) => px.0,
                    crate::element::MarginEdge::Auto => 0.0,
                };

                match props.direction {
                    fret_core::Axis::Horizontal => {
                        if single_child && (margin_left_auto || margin_right_auto) {
                            let left = if margin_left_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.left)
                            };
                            let right = if margin_right_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.right)
                            };
                            let free =
                                auto_margin_inner_size.width.0 - layout.size.width - left - right;
                            if margin_left_auto && margin_right_auto {
                                x = (left + (free.max(0.0) / 2.0)).max(0.0);
                            } else if margin_left_auto {
                                x = (left + free.max(0.0)).max(0.0);
                            } else if margin_right_auto {
                                x = left.max(0.0);
                            }
                        }

                        if margin_top_auto || margin_bottom_auto {
                            let top = if margin_top_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.top)
                            };
                            let bottom = if margin_bottom_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.bottom)
                            };
                            let free =
                                auto_margin_inner_size.height.0 - layout.size.height - top - bottom;
                            if margin_top_auto && margin_bottom_auto {
                                y = (top + (free.max(0.0) / 2.0)).max(0.0);
                            } else if margin_top_auto {
                                y = (top + free.max(0.0)).max(0.0);
                            } else if margin_bottom_auto {
                                y = top.max(0.0);
                            }
                        }
                    }
                    fret_core::Axis::Vertical => {
                        if single_child && (margin_top_auto || margin_bottom_auto) {
                            let top = if margin_top_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.top)
                            };
                            let bottom = if margin_bottom_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.bottom)
                            };
                            let free =
                                auto_margin_inner_size.height.0 - layout.size.height - top - bottom;
                            if margin_top_auto && margin_bottom_auto {
                                y = (top + (free.max(0.0) / 2.0)).max(0.0);
                            } else if margin_top_auto {
                                y = (top + free.max(0.0)).max(0.0);
                            } else if margin_bottom_auto {
                                y = top.max(0.0);
                            }
                        }

                        if margin_left_auto || margin_right_auto {
                            let left = if margin_left_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.left)
                            };
                            let right = if margin_right_auto {
                                0.0
                            } else {
                                margin_px(child_style.margin.right)
                            };
                            let free =
                                auto_margin_inner_size.width.0 - layout.size.width - left - right;
                            if margin_left_auto && margin_right_auto {
                                x = (left + (free.max(0.0) / 2.0)).max(0.0);
                            } else if margin_left_auto {
                                x = (left + free.max(0.0)).max(0.0);
                            } else if margin_right_auto {
                                x = left.max(0.0);
                            }
                        }
                    }
                }
                let rect = Rect::new(
                    fret_core::Point::new(Px(inner_origin.x.0 + x), Px(inner_origin.y.0 + y)),
                    Size::new(Px(layout.size.width), Px(layout.size.height)),
                );
                let _ = cx.layout_in(child, rect);
            }

            let desired = Size::new(
                Px((container_inner_size.width.0 + pad_w).max(0.0)),
                Px((container_inner_size.height.0 + pad_h).max(0.0)),
            );
            clamp_to_constraints(desired, props.layout, cx.available)
        }
    }
}

#[cfg(feature = "layout-engine-v2")]
impl ElementHostWidget {
    fn layout_flex_impl_engine<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
        if let Some(size) = layout_children_from_engine_if_solved(cx) {
            return size;
        }

        let pad_left = props.padding.left.0.max(0.0);
        let pad_right = props.padding.right.0.max(0.0);
        let pad_top = props.padding.top.0.max(0.0);
        let pad_bottom = props.padding.bottom.0.max(0.0);
        let pad_w = pad_left + pad_right;
        let pad_h = pad_top + pad_bottom;
        let inner_origin = fret_core::Point::new(
            Px(cx.bounds.origin.x.0 + pad_left),
            Px(cx.bounds.origin.y.0 + pad_top),
        );
        let outer_avail_w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
            Length::Fill | Length::Auto => cx.available.width,
        };
        let outer_avail_h = match props.layout.size.height {
            Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
            Length::Fill | Length::Auto => cx.available.height,
        };

        let inner_avail = Size::new(
            Px((outer_avail_w.0 - pad_w).max(0.0)),
            Px((outer_avail_h.0 - pad_h).max(0.0)),
        );

        let sf = if cx.scale_factor.is_finite() && cx.scale_factor > 0.0 {
            cx.scale_factor
        } else {
            1.0
        };

        let root_style = TaffyStyle {
            display: Display::Flex,
            flex_direction: match props.direction {
                fret_core::Axis::Horizontal => FlexDirection::Row,
                fret_core::Axis::Vertical => FlexDirection::Column,
            },
            flex_wrap: if props.wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            },
            justify_content: Some(taffy_justify(props.justify)),
            align_items: Some(taffy_align_items(props.align)),
            gap: TaffySize {
                width: LengthPercentage::length(props.gap.0.max(0.0) * sf),
                height: LengthPercentage::length(props.gap.0.max(0.0) * sf),
            },
            size: TaffySize {
                width: match props.layout.size.width {
                    Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0) * sf),
                    Length::Fill => Dimension::length(inner_avail.width.0.max(0.0) * sf),
                    Length::Auto => Dimension::auto(),
                },
                height: match props.layout.size.height {
                    Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0) * sf),
                    Length::Fill => Dimension::length(inner_avail.height.0.max(0.0) * sf),
                    Length::Auto => Dimension::auto(),
                },
            },
            max_size: TaffySize {
                width: Dimension::length(inner_avail.width.0.max(0.0) * sf),
                height: Dimension::length(inner_avail.height.0.max(0.0) * sf),
            },
            ..Default::default()
        };

        let available = LayoutSize::new(
            RuntimeAvailableSpace::Definite(inner_avail.width),
            RuntimeAvailableSpace::Definite(inner_avail.height),
        );

        let mut engine = cx.tree.take_layout_engine();
        let root_id = engine.request_layout_node(cx.node);
        engine.set_style(cx.node, root_style);
        engine.set_children(cx.node, cx.children);
        for &child in cx.children {
            build_flow_subtree(
                &mut engine,
                cx.app,
                &*cx.tree,
                window,
                sf,
                ParentLayoutKind::Flex {
                    direction: props.direction,
                },
                child,
            );
        }

        let app = &mut *cx.app;
        let services = &mut *cx.services;
        engine.compute_root_with_measure(root_id, available, sf, |child, constraints| {
            cx.tree.measure_in(app, services, child, constraints, sf)
        });

        let container_inner_size = {
            let rect = engine.layout_rect(root_id);
            Size::new(
                Px(rect.size.width.0.max(0.0)),
                Px(rect.size.height.0.max(0.0)),
            )
        };
        let auto_margin_inner_size = Size::new(
            match props.layout.size.width {
                Length::Fill => inner_avail.width,
                _ => container_inner_size.width,
            },
            match props.layout.size.height {
                Length::Fill => inner_avail.height,
                _ => container_inner_size.height,
            },
        );

        let mut child_layouts: Vec<(NodeId, Rect)> = Vec::with_capacity(cx.children.len());
        for &child in cx.children {
            let Some(id) = engine.layout_id_for_node(child) else {
                continue;
            };
            child_layouts.push((child, engine.layout_rect(id)));
        }

        cx.tree.put_layout_engine(engine);

        for (child, layout) in child_layouts {
            let child_style = layout_style_for_node(cx.app, window, child);
            let single_child = cx.children.len() == 1;

            let mut x = layout.origin.x.0;
            let mut y = layout.origin.y.0;

            let margin_left_auto =
                matches!(child_style.margin.left, crate::element::MarginEdge::Auto);
            let margin_right_auto =
                matches!(child_style.margin.right, crate::element::MarginEdge::Auto);
            let margin_top_auto =
                matches!(child_style.margin.top, crate::element::MarginEdge::Auto);
            let margin_bottom_auto =
                matches!(child_style.margin.bottom, crate::element::MarginEdge::Auto);

            let margin_px = |edge: crate::element::MarginEdge| match edge {
                crate::element::MarginEdge::Px(px) => px.0,
                crate::element::MarginEdge::Auto => 0.0,
            };

            match props.direction {
                fret_core::Axis::Horizontal => {
                    if single_child && (margin_left_auto || margin_right_auto) {
                        let left = if margin_left_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.left)
                        };
                        let right = if margin_right_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.right)
                        };
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width.0 - left - right;
                        if margin_left_auto && margin_right_auto {
                            x = (left + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_left_auto {
                            x = (left + free.max(0.0)).max(0.0);
                        } else if margin_right_auto {
                            x = left.max(0.0);
                        }
                    }

                    if margin_top_auto || margin_bottom_auto {
                        let top = if margin_top_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.top)
                        };
                        let bottom = if margin_bottom_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.bottom)
                        };
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height.0 - top - bottom;
                        if margin_top_auto && margin_bottom_auto {
                            y = (top + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_top_auto {
                            y = (top + free.max(0.0)).max(0.0);
                        } else if margin_bottom_auto {
                            y = top.max(0.0);
                        }
                    }
                }
                fret_core::Axis::Vertical => {
                    if single_child && (margin_top_auto || margin_bottom_auto) {
                        let top = if margin_top_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.top)
                        };
                        let bottom = if margin_bottom_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.bottom)
                        };
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height.0 - top - bottom;
                        if margin_top_auto && margin_bottom_auto {
                            y = (top + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_top_auto {
                            y = (top + free.max(0.0)).max(0.0);
                        } else if margin_bottom_auto {
                            y = top.max(0.0);
                        }
                    }

                    if margin_left_auto || margin_right_auto {
                        let left = if margin_left_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.left)
                        };
                        let right = if margin_right_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.right)
                        };
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width.0 - left - right;
                        if margin_left_auto && margin_right_auto {
                            x = (left + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_left_auto {
                            x = (left + free.max(0.0)).max(0.0);
                        } else if margin_right_auto {
                            x = left.max(0.0);
                        }
                    }
                }
            }

            let rect = Rect::new(
                fret_core::Point::new(Px(inner_origin.x.0 + x), Px(inner_origin.y.0 + y)),
                layout.size,
            );

            #[cfg(feature = "layout-engine-v2")]
            if cx.pass_kind == crate::layout_pass::LayoutPassKind::Final
                && !cx.tree.children(child).is_empty()
            {
                let sf = cx.scale_factor;
                let app = &mut *cx.app;
                let services = &mut *cx.services;
                let tree = &mut *cx.tree;
                tree.precompute_flow_root_island(app, services, child, rect, sf);
            }

            let _ = cx.layout_in(child, rect);
        }

        let desired = Size::new(
            Px((container_inner_size.width.0 + pad_w).max(0.0)),
            Px((container_inner_size.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints(desired, props.layout, cx.available)
    }
}
