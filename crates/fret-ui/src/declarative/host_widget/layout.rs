use super::super::frame::*;
use super::super::layout_helpers::*;
use super::super::prelude::*;
use super::ElementHostWidget;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

mod flex;
mod grid;
mod positioned_container;
mod scrolling;

impl ElementHostWidget {
    pub(super) fn layout_impl<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return Size::new(Px(0.0), Px(0.0));
        };

        if cx.pass_kind == crate::layout_pass::LayoutPassKind::Final {
            crate::elements::record_bounds_for_element(
                &mut *cx.app,
                window,
                self.element,
                cx.bounds,
            );
        }

        crate::elements::with_observed_models_for_element(cx.app, window, self.element, |items| {
            for &(model, invalidation) in items {
                (cx.observe_model)(model, invalidation);
            }
        });

        crate::elements::with_observed_globals_for_element(cx.app, window, self.element, |items| {
            for &(global, invalidation) in items {
                (cx.observe_global)(global, invalidation);
            }
        });

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return Size::new(Px(0.0), Px(0.0));
        };

        self.render_transform = None;
        self.scroll_child_transform = None;

        self.hit_testable = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::PointerRegion(p) => p.enabled,
            ElementInstance::TextInputRegion(p) => p.enabled,
            ElementInstance::InternalDragRegion(p) => p.enabled,
            ElementInstance::HoverRegion(_) => false,
            ElementInstance::Semantics(_) => false,
            ElementInstance::SemanticFlex(_) => false,
            ElementInstance::FocusScope(_) => false,
            ElementInstance::InteractivityGate(_) => false,
            ElementInstance::DismissibleLayer(_) => false,
            ElementInstance::Opacity(_) => false,
            ElementInstance::EffectLayer(_) => false,
            ElementInstance::ViewCache(_) => false,
            #[cfg(feature = "unstable-retained-bridge")]
            ElementInstance::RetainedSubtree(_) => false,
            ElementInstance::VisualTransform(_) => false,
            ElementInstance::RenderTransform(_) => false,
            ElementInstance::FractionalRenderTransform(_) => false,
            ElementInstance::Anchored(_) => false,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.hit_test_children = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::PointerRegion(_) => true,
            ElementInstance::TextInputRegion(_) => true,
            ElementInstance::InternalDragRegion(_) => true,
            ElementInstance::Semantics(_) => true,
            ElementInstance::SemanticFlex(_) => true,
            ElementInstance::FocusScope(_) => true,
            ElementInstance::InteractivityGate(p) => p.present && p.interactive,
            ElementInstance::DismissibleLayer(_) => true,
            ElementInstance::EffectLayer(_) => true,
            ElementInstance::ViewCache(_) => true,
            ElementInstance::VisualTransform(_) => true,
            ElementInstance::RenderTransform(_) => true,
            ElementInstance::FractionalRenderTransform(_) => true,
            ElementInstance::Anchored(_) => true,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.semantics_present = match &instance {
            ElementInstance::InteractivityGate(p) => p.present,
            _ => true,
        };
        self.semantics_children = match &instance {
            ElementInstance::InteractivityGate(p) => p.present,
            _ => true,
        };
        self.focus_traversal_children = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::InteractivityGate(p) => p.present && p.interactive,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.is_text_input = matches!(
            &instance,
            ElementInstance::TextInput(_)
                | ElementInstance::TextArea(_)
                | ElementInstance::TextInputRegion(_)
        );
        self.is_focusable = match &instance {
            ElementInstance::TextInput(_)
            | ElementInstance::TextArea(_)
            | ElementInstance::TextInputRegion(_) => true,
            ElementInstance::SelectableText(_) => true,
            ElementInstance::Pressable(p) => p.enabled && p.focusable,
            _ => false,
        };
        self.can_scroll_descendant = matches!(
            &instance,
            ElementInstance::Scroll(_) | ElementInstance::VirtualList(_)
        );
        self.clips_hit_test = match &instance {
            ElementInstance::Container(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Semantics(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::SemanticFlex(p) => matches!(p.flex.layout.overflow, Overflow::Clip),
            ElementInstance::FocusScope(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::InteractivityGate(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Opacity(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::EffectLayer(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::ViewCache(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::VisualTransform(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::RenderTransform(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::FractionalRenderTransform(p) => {
                matches!(p.layout.overflow, Overflow::Clip)
            }
            ElementInstance::Anchored(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Pressable(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::PointerRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextInputRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::InternalDragRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::DismissibleLayer(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Stack(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Flex(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::RovingFlex(p) => matches!(p.flex.layout.overflow, Overflow::Clip),
            ElementInstance::Grid(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextInput(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextArea(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::ResizablePanelGroup(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Scroll(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::HoverRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            #[cfg(feature = "unstable-retained-bridge")]
            ElementInstance::RetainedSubtree(p) => matches!(p.layout.overflow, Overflow::Clip),
            // These primitives are always hit-test clipped by their own bounds (they are not
            // intended as overflow-visible containers).
            ElementInstance::VirtualList(_)
            | ElementInstance::WheelRegion(_)
            | ElementInstance::Scrollbar(_)
            | ElementInstance::Image(_)
            | ElementInstance::Canvas(_)
            | ElementInstance::ViewportSurface(_)
            | ElementInstance::SvgIcon(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Text(_)
            | ElementInstance::StyledText(_)
            | ElementInstance::SelectableText(_) => true,
            ElementInstance::Spacer(_) => true,
        };
        self.clip_hit_test_corner_radii = match &instance {
            ElementInstance::Container(p) if matches!(p.layout.overflow, Overflow::Clip) => {
                if p.corner_radii.top_left.0 > 0.0
                    || p.corner_radii.top_right.0 > 0.0
                    || p.corner_radii.bottom_right.0 > 0.0
                    || p.corner_radii.bottom_left.0 > 0.0
                {
                    Some(p.corner_radii)
                } else {
                    None
                }
            }
            _ => None,
        };

        match instance {
            ElementInstance::Container(props) => {
                // Tailwind/shadcn assume `box-sizing: border-box` by default. Model borders as part
                // of the container's layout insets so child placement matches web geometry.
                let pad_left = props.padding.left.0.max(0.0) + props.border.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0) + props.border.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0) + props.border.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0) + props.border.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(
                        fret_core::Point::new(
                            Px(cx.bounds.origin.x.0 + pad_left),
                            Px(cx.bounds.origin.y.0 + pad_top),
                        ),
                        Size::new(
                            Px((cx.available.width.0 - pad_w).max(0.0)),
                            Px((cx.available.height.0 - pad_h).max(0.0)),
                        ),
                    ),
                ) {
                    return size;
                }

                let inner_avail = Size::new(
                    Px((cx.available.width.0 - pad_w).max(0.0)),
                    Px((cx.available.height.0 - pad_h).max(0.0)),
                );

                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, inner_avail);
                let probe_constraints = probe_constraints_for_size(probe_bounds.size);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                let mut non_absolute_sizes: Vec<(NodeId, Size)> = Vec::new();
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.measure_in(child, probe_constraints);
                    non_absolute_sizes.push((child, child_size));
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = Size::new(
                    Px((max_child.width.0 + pad_w).max(0.0)),
                    Px((max_child.height.0 + pad_h).max(0.0)),
                );
                let desired = clamp_to_constraints(desired, props.layout, cx.available);

                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );
                let inner_size = Size::new(
                    Px((desired.width.0 - pad_w).max(0.0)),
                    Px((desired.height.0 - pad_h).max(0.0)),
                );
                let inner_bounds = Rect::new(inner_origin, inner_size);
                let probe_inner_bounds = Rect::new(inner_origin, inner_avail);

                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    match positioned_layout_style(layout_style) {
                        PositionedLayoutStyle::Absolute(inset) => {
                            layout_absolute_child_with_probe_bounds(
                                cx,
                                child,
                                inner_bounds,
                                probe_inner_bounds,
                                inset,
                            )
                        }
                        PositionedLayoutStyle::Static => {
                            let child_size = non_absolute_sizes
                                .iter()
                                .find_map(|(id, size)| (*id == child).then_some(*size))
                                .unwrap_or(Size::new(Px(0.0), Px(0.0)));
                            let _ = cx.layout_in(child, Rect::new(inner_bounds.origin, child_size));
                        }
                        PositionedLayoutStyle::Relative(inset) => {
                            let child_size = non_absolute_sizes
                                .iter()
                                .find_map(|(id, size)| (*id == child).then_some(*size))
                                .unwrap_or(Size::new(Px(0.0), Px(0.0)));
                            let dx =
                                inset.left.unwrap_or(Px(0.0)).0 - inset.right.unwrap_or(Px(0.0)).0;
                            let dy =
                                inset.top.unwrap_or(Px(0.0)).0 - inset.bottom.unwrap_or(Px(0.0)).0;
                            let origin = fret_core::Point::new(
                                Px(inner_bounds.origin.x.0 + dx),
                                Px(inner_bounds.origin.y.0 + dy),
                            );
                            let _ = cx.layout_in(child, Rect::new(origin, child_size));
                        }
                    }
                }

                desired
            }
            ElementInstance::Pressable(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::Semantics(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::ViewCache(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::FocusScope(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::Opacity(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::InteractivityGate(props) => {
                if !props.present {
                    return Size::new(Px(0.0), Px(0.0));
                }

                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                // Pass-through wrapper (layout like Opacity/VisualTransform), but with separate
                // presence/interactivity gating handled via host widget flags.
                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::EffectLayer(props) => {
                if cx.children.len() == 1 {
                    let child = cx.children[0];
                    let child_style = layout_style_for_node(cx.app, window, child);
                    if child_style.position == crate::element::PositionStyle::Static
                        && let Some(bounds) = cx.layout_engine_child_bounds(child)
                    {
                        let _ = cx.layout_in(child, bounds);
                        return cx.available;
                    }
                }

                // Pass-through wrapper (layout like Opacity/VisualTransform), but with paint-time
                // `SceneOp::PushEffect/PopEffect` emission.
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    match positioned_layout_style(layout_style) {
                        PositionedLayoutStyle::Absolute(inset) => {
                            layout_absolute_child_with_probe_bounds(
                                cx,
                                child,
                                base,
                                probe_bounds,
                                inset,
                            )
                        }
                        style => layout_positioned_child(cx, child, base, style),
                    }
                }
                desired
            }
            ElementInstance::VisualTransform(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::RenderTransform(props) => {
                // Pass-through wrapper (layout like Opacity/VisualTransform), but with an explicit
                // render transform that affects hit-testing and pointer coordinate mapping.

                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    self.render_transform = Some(props.transform);
                    return size;
                }

                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_available =
                    clamp_to_constraints(cx.available, props.layout, cx.available);
                let probe_bounds = Rect::new(cx.bounds.origin, probe_available);
                let probe_constraints = probe_constraints_for_size(probe_bounds.size);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.measure_in(child, probe_constraints);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    match positioned_layout_style(layout_style) {
                        PositionedLayoutStyle::Absolute(inset) => {
                            layout_absolute_child_with_probe_bounds(
                                cx,
                                child,
                                base,
                                probe_bounds,
                                inset,
                            )
                        }
                        style => layout_positioned_child(cx, child, base, style),
                    }
                }

                self.render_transform = Some(props.transform);
                desired
            }
            ElementInstance::FractionalRenderTransform(props) => {
                // Pass-through wrapper like `RenderTransform`, but the translation is derived from
                // the element's own bounds during layout (matching CSS percentage translate behavior).

                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    let tx = size.width.0 * props.translate_x_fraction;
                    let ty = size.height.0 * props.translate_y_fraction;
                    if tx.is_finite() && ty.is_finite() && (tx != 0.0 || ty != 0.0) {
                        self.render_transform =
                            Some(Transform2D::translation(Point::new(Px(tx), Px(ty))));
                    }
                    return size;
                }

                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_available =
                    clamp_to_constraints(cx.available, props.layout, cx.available);
                let probe_bounds = Rect::new(cx.bounds.origin, probe_available);
                let probe_constraints = probe_constraints_for_size(probe_bounds.size);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.measure_in(child, probe_constraints);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    match positioned_layout_style(layout_style) {
                        PositionedLayoutStyle::Absolute(inset) => {
                            layout_absolute_child_with_probe_bounds(
                                cx,
                                child,
                                base,
                                probe_bounds,
                                inset,
                            )
                        }
                        style => layout_positioned_child(cx, child, base, style),
                    }
                }

                let tx = desired.width.0 * props.translate_x_fraction;
                let ty = desired.height.0 * props.translate_y_fraction;
                if tx.is_finite() && ty.is_finite() && (tx != 0.0 || ty != 0.0) {
                    self.render_transform =
                        Some(Transform2D::translation(Point::new(Px(tx), Px(ty))));
                }

                desired
            }
            ElementInstance::Anchored(props) => {
                // Layout-driven anchored placement. We measure the child subtree first, then
                // compute a placement transform relative to the wrapper bounds.

                let anchor = props
                    .anchor_element
                    .and_then(|element| {
                        crate::elements::node_for_element(
                            cx.app,
                            window,
                            crate::elements::GlobalElementId(element),
                        )
                    })
                    .and_then(|node| cx.tree.debug_node_bounds(node))
                    .filter(|bounds| *bounds != Rect::default())
                    .unwrap_or(props.anchor);

                if cx.children.len() == 1 {
                    let child = cx.children[0];
                    let child_style = layout_style_for_node(cx.app, window, child);
                    if child_style.position == crate::element::PositionStyle::Static
                        && let Some(child_bounds) = cx.layout_engine_child_bounds(child)
                    {
                        let _ = cx.layout_in(child, child_bounds);

                        let desired_child = child_bounds.size;
                        let outer =
                            crate::overlay_placement::inset_rect(cx.bounds, props.outer_margin);
                        let layout = crate::overlay_placement::anchored_panel_layout_sized_ex(
                            outer,
                            anchor,
                            desired_child,
                            props.side_offset,
                            props.side,
                            props.align,
                            props.options,
                        );

                        let delta = fret_core::Point::new(
                            Px(layout.rect.origin.x.0 - cx.bounds.origin.x.0),
                            Px(layout.rect.origin.y.0 - cx.bounds.origin.y.0),
                        );
                        self.render_transform = Some(fret_core::Transform2D::translation(delta));

                        if let Some(out) = props.layout_out {
                            let _ = cx.app.models_mut().update(&out, |v| {
                                if *v != layout {
                                    *v = layout;
                                }
                            });
                        }

                        return cx.available;
                    }
                }

                let probe_available =
                    clamp_to_constraints(cx.available, props.layout, cx.available);
                let probe_bounds = Rect::new(cx.bounds.origin, probe_available);
                let probe_constraints = probe_constraints_for_size(probe_bounds.size);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.measure_in(child, probe_constraints);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired_child = max_child;
                let desired = clamp_to_constraints(desired_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);

                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    match positioned_layout_style(layout_style) {
                        PositionedLayoutStyle::Absolute(inset) => {
                            layout_absolute_child_with_probe_bounds(
                                cx,
                                child,
                                base,
                                probe_bounds,
                                inset,
                            )
                        }
                        style => layout_positioned_child(cx, child, base, style),
                    }
                }

                let outer = crate::overlay_placement::inset_rect(cx.bounds, props.outer_margin);
                let layout = crate::overlay_placement::anchored_panel_layout_sized_ex(
                    outer,
                    anchor,
                    desired_child,
                    props.side_offset,
                    props.side,
                    props.align,
                    props.options,
                );

                let delta = fret_core::Point::new(
                    Px(layout.rect.origin.x.0 - cx.bounds.origin.x.0),
                    Px(layout.rect.origin.y.0 - cx.bounds.origin.y.0),
                );
                self.render_transform = Some(fret_core::Transform2D::translation(delta));

                if let Some(out) = props.layout_out {
                    let _ = cx.app.models_mut().update(&out, |v| {
                        if *v != layout {
                            *v = layout;
                        }
                    });
                }

                desired
            }
            ElementInstance::DismissibleLayer(props) => {
                let desired = clamp_to_constraints(cx.available, props.layout, cx.available);
                let base = cx.bounds;

                if let Some(size) =
                    try_layout_children_from_engine_or_manual_absolute(cx, window, base)
                {
                    return size;
                }

                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Stack(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }

                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::Spacer(props) => {
                clamp_to_constraints(Size::new(Px(0.0), Px(0.0)), props.layout, cx.available)
            }
            ElementInstance::Text(props) => {
                cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
                let font_stack_key = cx
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0)
                    .unwrap_or(0);
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                    line_height: Some(
                        cx.theme()
                            .metric_by_key("font.line_height")
                            .unwrap_or(cx.theme().metrics.font_line_height),
                    ),
                    ..Default::default()
                });
                let mut measure_width = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                if let Some(max_w) = props.layout.size.max_width {
                    measure_width = Px(measure_width.0.min(max_w.0.max(0.0)));
                }
                measure_width = Px(measure_width.0.max(0.0).min(cx.available.width.0.max(0.0)));
                let constraints = TextConstraints {
                    max_width: Some(measure_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };

                let scale_bits = cx.scale_factor.to_bits();
                let can_reuse_metrics = self.text_cache.metrics.is_some()
                    && self.text_cache.last_text.as_ref() == Some(&props.text)
                    && self.text_cache.last_style.as_ref() == Some(&style)
                    && self.text_cache.last_wrap == Some(props.wrap)
                    && self.text_cache.last_overflow == Some(props.overflow)
                    && self.text_cache.last_measure_width == Some(measure_width)
                    && self.text_cache.measured_scale_factor_bits == Some(scale_bits)
                    && self.text_cache.last_font_stack_key == Some(font_stack_key);

                let metrics = if can_reuse_metrics {
                    self.text_cache.metrics.expect("cached metrics")
                } else {
                    let input = fret_core::TextInput::plain(props.text.clone(), style.clone());
                    let metrics = cx.services.text().measure(&input, constraints);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.measured_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = Some(props.text.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_measure_width = Some(measure_width);
                    self.text_cache.last_font_stack_key = Some(font_stack_key);
                    metrics
                };

                clamp_to_constraints(metrics.size, props.layout, cx.available)
            }
            ElementInstance::StyledText(props) => {
                cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
                let font_stack_key = cx
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0)
                    .unwrap_or(0);
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                    line_height: Some(
                        cx.theme()
                            .metric_by_key("font.line_height")
                            .unwrap_or(cx.theme().metrics.font_line_height),
                    ),
                    ..Default::default()
                });
                let mut measure_width = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                if let Some(max_w) = props.layout.size.max_width {
                    measure_width = Px(measure_width.0.min(max_w.0.max(0.0)));
                }
                measure_width = Px(measure_width.0.max(0.0).min(cx.available.width.0.max(0.0)));
                let constraints = TextConstraints {
                    max_width: Some(measure_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };

                let scale_bits = cx.scale_factor.to_bits();
                let can_reuse_metrics = self.text_cache.metrics.is_some()
                    && self
                        .text_cache
                        .last_rich
                        .as_ref()
                        .is_some_and(|rich| rich.shaping_eq(&props.rich))
                    && self.text_cache.last_style.as_ref() == Some(&style)
                    && self.text_cache.last_wrap == Some(props.wrap)
                    && self.text_cache.last_overflow == Some(props.overflow)
                    && self.text_cache.last_measure_width == Some(measure_width)
                    && self.text_cache.measured_scale_factor_bits == Some(scale_bits)
                    && self.text_cache.last_font_stack_key == Some(font_stack_key);

                let metrics = if can_reuse_metrics {
                    self.text_cache.metrics.expect("cached metrics")
                } else {
                    let input = fret_core::TextInput::attributed(
                        props.rich.text.clone(),
                        style.clone(),
                        props.rich.spans.clone(),
                    );
                    let metrics = cx.services.text().measure(&input, constraints);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.measured_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = None;
                    self.text_cache.last_rich = Some(props.rich.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_measure_width = Some(measure_width);
                    self.text_cache.last_font_stack_key = Some(font_stack_key);
                    metrics
                };

                clamp_to_constraints(metrics.size, props.layout, cx.available)
            }
            ElementInstance::SelectableText(props) => {
                cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
                let font_stack_key = cx
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0)
                    .unwrap_or(0);
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                    line_height: Some(
                        cx.theme()
                            .metric_by_key("font.line_height")
                            .unwrap_or(cx.theme().metrics.font_line_height),
                    ),
                    ..Default::default()
                });
                let mut measure_width = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                if let Some(max_w) = props.layout.size.max_width {
                    measure_width = Px(measure_width.0.min(max_w.0.max(0.0)));
                }
                measure_width = Px(measure_width.0.max(0.0).min(cx.available.width.0.max(0.0)));
                let constraints = TextConstraints {
                    max_width: Some(measure_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };

                let scale_bits = cx.scale_factor.to_bits();
                let can_reuse_metrics = self.text_cache.metrics.is_some()
                    && self
                        .text_cache
                        .last_rich
                        .as_ref()
                        .is_some_and(|rich| rich.shaping_eq(&props.rich))
                    && self.text_cache.last_style.as_ref() == Some(&style)
                    && self.text_cache.last_wrap == Some(props.wrap)
                    && self.text_cache.last_overflow == Some(props.overflow)
                    && self.text_cache.last_measure_width == Some(measure_width)
                    && self.text_cache.measured_scale_factor_bits == Some(scale_bits)
                    && self.text_cache.last_font_stack_key == Some(font_stack_key);

                let metrics = if can_reuse_metrics {
                    self.text_cache.metrics.expect("cached metrics")
                } else {
                    let input = fret_core::TextInput::attributed(
                        props.rich.text.clone(),
                        style.clone(),
                        props.rich.spans.clone(),
                    );
                    let metrics = cx.services.text().measure(&input, constraints);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.measured_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = None;
                    self.text_cache.last_rich = Some(props.rich.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_measure_width = Some(measure_width);
                    self.text_cache.last_font_stack_key = Some(font_stack_key);
                    metrics
                };

                clamp_to_constraints(metrics.size, props.layout, cx.available)
            }
            ElementInstance::TextInput(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(model.clone()));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != model_id {
                    input.set_model(model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_placeholder(props.placeholder);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);

                let desired = input.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::TextArea(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(model.clone()));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != model_id {
                    area.set_model(model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);

                let desired = area.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::ResizablePanelGroup(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            model.clone(),
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != model_id {
                    group.set_model(model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());

                let desired = group.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::VirtualList(props) => self.layout_virtual_list_impl(cx, window, props),
            ElementInstance::Flex(props) => self.layout_flex_impl(cx, window, props),
            ElementInstance::SemanticFlex(props) => self.layout_flex_impl(cx, window, props.flex),
            ElementInstance::RovingFlex(props) => self.layout_flex_impl(cx, window, props.flex),
            ElementInstance::Grid(props) => self.layout_grid_impl(cx, window, props),
            ElementInstance::Image(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::Canvas(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            #[cfg(feature = "unstable-retained-bridge")]
            ElementInstance::RetainedSubtree(props) => {
                if let Some(&child) = cx.children.get(0) {
                    let bounds = Rect::new(cx.bounds.origin, cx.available);
                    let _ = cx.layout_in(child, bounds);
                }
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::ViewportSurface(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::SvgIcon(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::Spinner(props) => {
                clamp_to_constraints(Size::new(Px(16.0), Px(16.0)), props.layout, cx.available)
            }
            ElementInstance::PointerRegion(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }
                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::TextInputRegion(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }
                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::InternalDragRegion(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }
                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::HoverRegion(props) => {
                let has_absolute_child = cx.children.iter().copied().any(|child| {
                    layout_style_for_node(cx.app, window, child).position
                        == crate::element::PositionStyle::Absolute
                });
                if !has_absolute_child
                    && let Some(size) =
                        crate::layout_engine::layout_children_from_engine_if_solved(cx)
                {
                    return size;
                }
                self.layout_hover_region_impl(cx, window, props.layout)
            }
            ElementInstance::WheelRegion(props) => {
                if let Some(size) = try_layout_children_from_engine_or_manual_absolute(
                    cx,
                    window,
                    Rect::new(cx.bounds.origin, cx.available),
                ) {
                    return size;
                }
                self.layout_positioned_container_impl(cx, window, props.layout)
            }
            ElementInstance::Scroll(props) => self.layout_scroll_impl(cx, window, props),
            ElementInstance::Scrollbar(props) => self.layout_scrollbar_impl(cx, props),
        }
    }
}

fn probe_constraints_for_size(size: Size) -> LayoutConstraints {
    LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(size.width),
            AvailableSpace::Definite(size.height),
        ),
    )
}

fn try_layout_children_from_engine_or_manual_absolute<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    window: AppWindowId,
    base_for_absolute: Rect,
) -> Option<Size> {
    if cx.children.is_empty() {
        return None;
    }
    if let Some(size) = crate::layout_engine::layout_children_from_engine_if_solved(cx) {
        return Some(size);
    }
    try_layout_children_from_engine_with_manual_absolute(cx, window, base_for_absolute)
}

fn try_layout_children_from_engine_with_manual_absolute<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    window: AppWindowId,
    base_for_absolute: Rect,
) -> Option<Size> {
    if cx.children.is_empty() {
        return None;
    }

    let mut any_engine_child = false;
    let mut non_absolute: Vec<(NodeId, Rect)> = Vec::new();
    let mut absolute: Vec<(NodeId, crate::element::InsetStyle)> = Vec::new();

    for &child in cx.children {
        let style = layout_style_for_node(cx.app, window, child);
        if style.position == crate::element::PositionStyle::Absolute {
            if cx.layout_engine_child_bounds(child).is_some() {
                any_engine_child = true;
            }
            absolute.push((child, style.inset));
            continue;
        }

        let bounds = cx.layout_engine_child_bounds(child)?;
        any_engine_child = true;
        non_absolute.push((child, bounds));
    }

    if !any_engine_child {
        return None;
    }

    for (child, bounds) in non_absolute {
        let _ = cx.layout_in(child, bounds);
    }

    for (child, inset) in absolute {
        layout_positioned_child(
            cx,
            child,
            base_for_absolute,
            PositionedLayoutStyle::Absolute(inset),
        );
    }

    Some(cx.available)
}
