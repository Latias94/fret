use super::frame::ElementInstance;
use super::frame::element_record_for_node;
use super::frame::layout_style_for_node;
use super::layout_helpers::clamp_to_constraints;
use super::prelude::*;
use super::taffy_layout::*;
use crate::widget::CommandCx;
use fret_runtime::CommandId;

mod event;
mod layout;
mod paint;
mod semantics;

#[derive(Debug, Default, Clone)]
struct TextCache {
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    prepared_scale_factor_bits: Option<u32>,
    measured_scale_factor_bits: Option<u32>,
    last_text: Option<std::sync::Arc<str>>,
    last_style: Option<TextStyle>,
    last_wrap: Option<fret_core::TextWrap>,
    last_overflow: Option<TextOverflow>,
    last_width: Option<Px>,
    last_measure_width: Option<Px>,
    last_theme_revision: Option<u64>,
    last_font_stack_key: Option<u64>,
}

pub(super) struct ElementHostWidget {
    element: GlobalElementId,
    text_cache: TextCache,
    render_transform: Option<fret_core::Transform2D>,
    hit_testable: bool,
    hit_test_children: bool,
    focus_traversal_children: bool,
    semantics_present: bool,
    semantics_children: bool,
    is_focusable: bool,
    is_text_input: bool,
    can_scroll_descendant: bool,
    clips_hit_test: bool,
    clip_hit_test_corner_radii: Option<fret_core::Corners>,
    text_input: Option<BoundTextInput>,
    text_area: Option<crate::text_area::BoundTextArea>,
    resizable_panel_group: Option<crate::resizable_panel_group::BoundResizablePanelGroup>,
    flex_cache: Option<TaffyContainerCache>,
    grid_cache: Option<TaffyContainerCache>,
}

impl ElementHostWidget {
    pub(super) fn new(element: GlobalElementId) -> Self {
        Self {
            element,
            text_cache: TextCache::default(),
            render_transform: None,
            hit_testable: true,
            hit_test_children: true,
            focus_traversal_children: true,
            semantics_present: true,
            semantics_children: true,
            is_focusable: false,
            is_text_input: false,
            can_scroll_descendant: false,
            clips_hit_test: true,
            clip_hit_test_corner_radii: None,
            text_input: None,
            text_area: None,
            resizable_panel_group: None,
            flex_cache: None,
            grid_cache: None,
        }
    }

    fn instance<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Option<ElementInstance> {
        element_record_for_node(app, window, node).map(|r| r.instance)
    }

    fn layout_flex_container<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
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

                    let max_w = match avail.width {
                        TaffyAvailableSpace::Definite(w) => Px(w),
                        _ => Px(1.0e9),
                    };
                    let max_h = match avail.height {
                        TaffyAvailableSpace::Definite(h) => Px(h),
                        _ => Px(1.0e9),
                    };

                    let known_w = known.width.map(Px);
                    let known_h = known.height.map(Px);

                    let w = known_w.unwrap_or(max_w);
                    let h = known_h.unwrap_or(max_h);

                    let probe = Rect::new(inner_origin, Size::new(w, h));
                    let s = cx.layout_in(child, probe);
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

impl<H: UiHost> Widget<H> for ElementHostWidget {
    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return false;
        };

        match instance {
            ElementInstance::FocusScope(props) if props.trap_focus => {
                let forward = match command.as_str() {
                    "focus.next" => Some(true),
                    "focus.previous" => Some(false),
                    _ => None,
                };
                let Some(forward) = forward else {
                    return false;
                };

                cx.tree
                    .focus_traverse_in_roots(cx.app, &[cx.node], forward, Some(cx.node));
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }

    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        self.clips_hit_test
    }

    fn render_transform(&self, _bounds: Rect) -> Option<fret_core::Transform2D> {
        self.render_transform
    }

    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<fret_core::Corners> {
        self.clip_hit_test_corner_radii
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        self.hit_testable
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        if !self.hit_test_children {
            return false;
        }
        true
    }

    fn focus_traversal_children(&self) -> bool {
        self.focus_traversal_children
    }

    fn semantics_present(&self) -> bool {
        self.semantics_present
    }

    fn semantics_children(&self) -> bool {
        self.semantics_children
    }

    fn is_focusable(&self) -> bool {
        self.is_focusable
    }

    fn is_text_input(&self) -> bool {
        self.is_text_input
    }

    fn can_scroll_descendant_into_view(&self) -> bool {
        self.can_scroll_descendant
    }

    fn scroll_descendant_into_view(
        &mut self,
        cx: &mut crate::widget::ScrollIntoViewCx<'_, H>,
        descendant_bounds: Rect,
    ) -> crate::widget::ScrollIntoViewResult {
        let Some(window) = cx.window else {
            return crate::widget::ScrollIntoViewResult::NotHandled;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return crate::widget::ScrollIntoViewResult::NotHandled;
        };

        match instance {
            ElementInstance::Scroll(props) => {
                let handle = if let Some(handle) = props.scroll_handle.as_ref() {
                    handle.clone()
                } else {
                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::ScrollState::default,
                        |state| state.scroll_handle.clone(),
                    )
                };

                crate::widget::ScrollIntoViewResult::Handled {
                    did_scroll: scroll_handle_into_view_y(&handle, cx.bounds, descendant_bounds),
                }
            }
            ElementInstance::VirtualList(props) => crate::widget::ScrollIntoViewResult::Handled {
                did_scroll: scroll_handle_into_view_y(
                    props.scroll_handle.base_handle(),
                    cx.bounds,
                    descendant_bounds,
                ),
            },
            _ => crate::widget::ScrollIntoViewResult::NotHandled,
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.event_impl(cx, event);
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(blob) = self.text_cache.blob.take() {
            services.text().release(blob);
        }
        self.text_cache.prepared_scale_factor_bits = None;
        self.text_cache.metrics = None;
        if let Some(input) = self.text_input.as_mut() {
            input.cleanup_resources(services);
        }
        if let Some(area) = self.text_area.as_mut() {
            area.cleanup_resources(services);
        }
        if let Some(group) = self.resizable_panel_group.as_mut() {
            group.cleanup_resources(services);
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        self.semantics_impl(cx);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.layout_impl(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paint_impl(cx);
    }
}

fn scroll_handle_into_view_y(
    handle: &crate::scroll::ScrollHandle,
    viewport: Rect,
    child: Rect,
) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let view_bottom = view_top + viewport_h;
    let child_top = child.origin.y.0;
    let child_bottom = child_top + child.size.height.0.max(0.0);

    let delta = if child_top < view_top {
        child_top - view_top
    } else if child_bottom > view_bottom {
        child_bottom - view_bottom
    } else {
        0.0
    };

    if delta.abs() <= 0.01 {
        return false;
    }

    let prev = handle.offset();
    handle.set_offset(Point::new(prev.x, Px(prev.y.0 + delta)));

    let next = handle.offset();
    (prev.y.0 - next.y.0).abs() > 0.01
}
