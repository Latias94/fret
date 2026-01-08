use super::super::ElementHostWidget;
use crate::declarative::frame::layout_style_for_node;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;
use crate::declarative::taffy_layout::*;
use crate::layout_constraints::{
    AvailableSpace as RuntimeAvailableSpace, LayoutConstraints, LayoutSize,
};

impl ElementHostWidget {
    pub(super) fn layout_grid_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: crate::element::GridProps,
    ) -> Size {
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
            display: Display::Grid,
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
            grid_template_columns: taffy::style_helpers::evenly_sized_tracks(props.cols),
            grid_template_rows: props
                .rows
                .map(taffy::style_helpers::evenly_sized_tracks)
                .unwrap_or_default(),
            ..Default::default()
        };

        let cache = self
            .grid_cache
            .get_or_insert_with(TaffyContainerCache::default);

        cache.sync_root_style(root_style);
        cache.sync_children(cx.children, |child| {
            let layout_style = layout_style_for_node(cx.app, window, child);

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
                    width: layout_style
                        .size
                        .min_width
                        .map(|p| Dimension::length(p.0))
                        .unwrap_or_else(Dimension::auto),
                    height: layout_style
                        .size
                        .min_height
                        .map(|p| Dimension::length(p.0))
                        .unwrap_or_else(Dimension::auto),
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
                grid_column: taffy_grid_line(layout_style.grid.column),
                grid_row: taffy_grid_line(layout_style.grid.row),
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
                .compute_layout_with_measure(root, available, |known, avail, _id, ctx, _style| {
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
                })
                .expect("taffy compute");
        }

        let taffy = &cache.taffy;

        for &child_node in &cache.child_nodes {
            let layout = taffy.layout(child_node).expect("taffy layout");
            let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                continue;
            };
            let rect = Rect::new(
                fret_core::Point::new(
                    Px(inner_origin.x.0 + layout.location.x),
                    Px(inner_origin.y.0 + layout.location.y),
                ),
                Size::new(Px(layout.size.width), Px(layout.size.height)),
            );
            let _ = cx.layout_in(child, rect);
        }

        let layout = taffy.layout(root).expect("taffy root layout");
        let inner_size = Size::new(
            Px(layout.size.width.max(0.0)),
            Px(layout.size.height.max(0.0)),
        );

        let desired = Size::new(
            Px((inner_size.width.0 + pad_w).max(0.0)),
            Px((inner_size.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints(desired, props.layout, cx.available)
    }
}
