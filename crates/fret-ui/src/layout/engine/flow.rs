use crate::UiHost;
use crate::declarative::frame::{ElementInstance, element_record_for_node, layout_style_for_node};
use crate::layout_engine::TaffyLayoutEngine;
use crate::tree::UiTree;
use crate::widget::LayoutCx;
use fret_core::{AppWindowId, NodeId, Px, Rect, Size};
use taffy::geometry::{Line as TaffyLine, Rect as TaffyRect, Size as TaffySize};
use taffy::style::{
    AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, GridPlacement,
    GridTemplateComponent, JustifyContent, LengthPercentage, LengthPercentageAuto,
    Position as TaffyPosition, Style,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ParentLayoutKind {
    Root,
    Flex { direction: fret_core::Axis },
    Grid,
    PassthroughOverlayStretch,
    PassthroughOverlayNoStretch,
    Overlay,
}

pub(crate) fn layout_children_from_engine_if_solved<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
) -> Option<Size> {
    let mut child_bounds: Vec<(NodeId, Rect)> = Vec::with_capacity(cx.children.len());
    for &child in cx.children {
        let bounds = cx.layout_engine_child_bounds(child)?;
        child_bounds.push((child, bounds));
    }
    for (child, bounds) in child_bounds {
        let _ = cx.layout_in(child, bounds);
    }
    Some(cx.available)
}

fn apply_container_insets(style: &mut Style, props: &crate::element::ContainerProps, sf: f32) {
    fn adjust_border_box_dimension(dim: &mut Dimension, inset: f32) {
        let length_tag = Dimension::length(0.0).tag();
        if dim.tag() != length_tag {
            return;
        }
        let value = dim.value();
        *dim = Dimension::length((value - inset).max(0.0));
    }

    style.padding = TaffyRect {
        left: LengthPercentage::length(scale_nonneg_px(props.padding.left, sf)),
        right: LengthPercentage::length(scale_nonneg_px(props.padding.right, sf)),
        top: LengthPercentage::length(scale_nonneg_px(props.padding.top, sf)),
        bottom: LengthPercentage::length(scale_nonneg_px(props.padding.bottom, sf)),
    };
    style.border = TaffyRect {
        left: LengthPercentage::length(scale_nonneg_px(props.border.left, sf)),
        right: LengthPercentage::length(scale_nonneg_px(props.border.right, sf)),
        top: LengthPercentage::length(scale_nonneg_px(props.border.top, sf)),
        bottom: LengthPercentage::length(scale_nonneg_px(props.border.bottom, sf)),
    };

    // Tailwind preflight uses `box-sizing: border-box`, meaning `width`/`min-width`/etc are
    // specified on the border box. Taffy applies padding/border outside of the content box, so we
    // subtract insets here to keep border-box sizing consistent with shadcn-web geometry.
    // Taffy models padding as part of the resolved box size, but border is applied outside of it.
    // Subtract border thickness from size constraints so `h-*`/`w-*` and `min-*` tokens line up
    // with Tailwind's `border-box` sizing (padding included; border included).
    let inset_w = scale_nonneg_px(props.border.left, sf) + scale_nonneg_px(props.border.right, sf);
    let inset_h = scale_nonneg_px(props.border.top, sf) + scale_nonneg_px(props.border.bottom, sf);

    adjust_border_box_dimension(&mut style.min_size.width, inset_w);
    adjust_border_box_dimension(&mut style.min_size.height, inset_h);
    // NOTE: Taffy applies `max_size` constraints after border/padding are resolved. Treating
    // `max_width`/`max_height` as border-box constraints (Tailwind preflight) without additional
    // subtraction keeps shadcn-style `w-full max-w-*` overlays aligned with web snapshots.
}

pub(crate) fn build_flow_subtree<H: UiHost>(
    engine: &mut TaffyLayoutEngine,
    app: &mut H,
    tree: &UiTree<H>,
    window: AppWindowId,
    scale_factor: f32,
    parent_kind: ParentLayoutKind,
    node: NodeId,
) {
    build_flow_subtree_impl(
        engine,
        app,
        tree,
        window,
        scale_factor,
        parent_kind,
        node,
        None,
    );
}

pub(crate) fn build_viewport_flow_subtree<H: UiHost>(
    engine: &mut TaffyLayoutEngine,
    app: &mut H,
    tree: &UiTree<H>,
    window: AppWindowId,
    scale_factor: f32,
    viewport_root: NodeId,
    viewport_size: Size,
) {
    build_flow_subtree_impl(
        engine,
        app,
        tree,
        window,
        scale_factor,
        ParentLayoutKind::Root,
        viewport_root,
        Some(viewport_size),
    );
}

#[allow(clippy::too_many_arguments)]
#[stacksafe::stacksafe]
fn build_flow_subtree_impl<H: UiHost>(
    engine: &mut TaffyLayoutEngine,
    app: &mut H,
    tree: &UiTree<H>,
    window: AppWindowId,
    scale_factor: f32,
    parent_kind: ParentLayoutKind,
    node: NodeId,
    root_override_size: Option<Size>,
) {
    let sf = sanitize_scale_factor(scale_factor);
    let _ = engine.request_layout_node(node);
    if let Some((child, child_parent_kind)) = passthrough_wrapper_child(app, tree, window, node) {
        let mut style = style_for_item_in_parent(
            app,
            window,
            sf,
            parent_kind,
            node,
            Display::Grid,
            root_override_size,
        );
        // Wrapper nodes (semantics/hover/pressable/opacity/...) must be "pass-through": they
        // provide a containing block for percent sizing, but should not impose sizing on their
        // single child.
        //
        // In particular, shrink-wrapped wrappers (e.g. a `Pressable` button trigger inside a
        // larger container) must not stretch to the parent's full width/height just because the
        // wrapper is modeled as a grid container.
        //
        // We therefore default to `auto` tracks (shrink-wrap), and only opt into a `fr(1)` track
        // when the wrapper is expected to act as a definite containing block (e.g. when the child
        // is `Fill` in that axis).
        let wrapper_style = layout_style_for_node(app, window, node);

        // Percent sizing needs a definite containing block. For nested wrapper chains like:
        // `Semantics -> FocusScope -> ... -> Fill`, we must promote the entire chain so the leaf
        // can resolve its `Fill` size (and so the wrappers don't collapse to 0).
        let mut descendant_requests_fill_width = false;
        let mut descendant_requests_fill_height = false;
        let mut scan_width = matches!(wrapper_style.size.width, crate::element::Length::Auto);
        let mut scan_height = matches!(wrapper_style.size.height, crate::element::Length::Auto);
        let mut probe = child;
        for _ in 0..32 {
            let probe_style = layout_style_for_node(app, window, probe);

            if scan_width {
                match probe_style.size.width {
                    crate::element::Length::Fill => {
                        descendant_requests_fill_width = true;
                        scan_width = false;
                    }
                    crate::element::Length::Px(_) => {
                        scan_width = false;
                    }
                    crate::element::Length::Auto => {}
                }
            }
            if scan_height {
                match probe_style.size.height {
                    crate::element::Length::Fill => {
                        descendant_requests_fill_height = true;
                        scan_height = false;
                    }
                    crate::element::Length::Px(_) => {
                        scan_height = false;
                    }
                    crate::element::Length::Auto => {}
                }
            }

            if !scan_width && !scan_height {
                break;
            }

            let Some((next, _)) = passthrough_wrapper_child(app, tree, window, probe) else {
                break;
            };
            probe = next;
        }

        let needs_definite_width = descendant_requests_fill_width
            || matches!(wrapper_style.size.width, crate::element::Length::Fill)
            || matches!(wrapper_style.size.width, crate::element::Length::Px(_));
        style.grid_template_columns =
            vec![GridTemplateComponent::Single(if needs_definite_width {
                taffy::style_helpers::fr(1.0)
            } else {
                taffy::style_helpers::auto()
            })];

        let needs_definite_height = descendant_requests_fill_height
            || matches!(wrapper_style.size.height, crate::element::Length::Fill)
            || matches!(wrapper_style.size.height, crate::element::Length::Px(_));
        style.grid_template_rows = vec![GridTemplateComponent::Single(if needs_definite_height {
            taffy::style_helpers::fr(1.0)
        } else {
            taffy::style_helpers::auto()
        })];
        let child_instance = element_record_for_node(app, window, child).map(|r| r.instance);
        let child_is_layout_container = matches!(
            child_instance,
            Some(ElementInstance::Flex(_))
                | Some(ElementInstance::SemanticFlex(_))
                | Some(ElementInstance::RovingFlex(_))
                | Some(ElementInstance::Stack(_))
                | Some(ElementInstance::Grid(_))
        );

        // Prevent wrappers from stretching intrinsic/auto-sized children (e.g. spacers), but still
        // stretch layout containers (Flex/Grid/Stack/...) when the wrapper provides a definite box.
        style.align_items = Some(if needs_definite_height && child_is_layout_container {
            AlignItems::Stretch
        } else {
            AlignItems::FlexStart
        });
        style.justify_items = Some(if needs_definite_width && child_is_layout_container {
            AlignItems::Stretch
        } else {
            AlignItems::FlexStart
        });
        style.justify_content = Some(JustifyContent::FlexStart);

        if matches!(wrapper_style.size.width, crate::element::Length::Auto)
            && descendant_requests_fill_width
        {
            style.size.width = Dimension::percent(1.0);
        }
        if matches!(wrapper_style.size.height, crate::element::Length::Auto)
            && descendant_requests_fill_height
        {
            style.size.height = Dimension::percent(1.0);
        }

        if let Some(props) = element_record_for_node(app, window, node).and_then(|r| {
            if let ElementInstance::Container(p) = r.instance {
                Some(p)
            } else {
                None
            }
        }) {
            apply_container_insets(&mut style, &props, sf);
        }

        engine.set_style(node, style);
        engine.set_children(node, &[child]);
        engine.set_measured(node, false);
        build_flow_subtree(engine, app, tree, window, sf, child_parent_kind, child);
        return;
    }

    let instance = element_record_for_node(app, window, node).map(|r| r.instance);
    match instance {
        Some(ElementInstance::InteractivityGate(props)) if !props.present => {
            let style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Block,
                root_override_size,
            );
            engine.set_style(node, style);
            engine.set_children(node, &[]);
            engine.set_measured(node, true);
        }
        Some(
            instance @ (ElementInstance::Container(_)
            | ElementInstance::Pressable(_)
            | ElementInstance::Opacity(_)
            | ElementInstance::VisualTransform(_)
            | ElementInstance::RenderTransform(_)
            | ElementInstance::FractionalRenderTransform(_)
            | ElementInstance::Semantics(_)
            | ElementInstance::ViewCache(_)
            | ElementInstance::FocusScope(_)
            | ElementInstance::InteractivityGate(_)
            | ElementInstance::PointerRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::WheelRegion(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::Anchored(_)
            | ElementInstance::Stack(_)),
        ) if tree.children(node).is_empty() => {
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Block,
                root_override_size,
            );

            if let ElementInstance::Container(p) = &instance {
                apply_container_insets(&mut style, p, sf);
            }

            engine.set_style(node, style);
            engine.set_children(node, &[]);
            engine.set_measured(node, false);
        }
        Some(ElementInstance::Flex(props)) => {
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Flex,
                root_override_size,
            );
            style.flex_direction = match props.direction {
                fret_core::Axis::Horizontal => FlexDirection::Row,
                fret_core::Axis::Vertical => FlexDirection::Column,
            };
            style.flex_wrap = if props.wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            };
            style.justify_content = Some(taffy_justify(props.justify));
            style.align_items = Some(taffy_align_items(props.align));
            style.gap = TaffySize {
                width: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
                height: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
            };
            style.padding = TaffyRect {
                left: LengthPercentage::length(scale_nonneg_px(props.padding.left, sf)),
                right: LengthPercentage::length(scale_nonneg_px(props.padding.right, sf)),
                top: LengthPercentage::length(scale_nonneg_px(props.padding.top, sf)),
                bottom: LengthPercentage::length(scale_nonneg_px(props.padding.bottom, sf)),
            };

            let children = tree.children(node).to_vec();
            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                build_flow_subtree(
                    engine,
                    app,
                    tree,
                    window,
                    sf,
                    ParentLayoutKind::Flex {
                        direction: props.direction,
                    },
                    child,
                );
            }
        }
        Some(ElementInstance::SemanticFlex(props)) => {
            let props = props.flex;
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Flex,
                root_override_size,
            );
            style.flex_direction = match props.direction {
                fret_core::Axis::Horizontal => FlexDirection::Row,
                fret_core::Axis::Vertical => FlexDirection::Column,
            };
            style.flex_wrap = if props.wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            };
            style.justify_content = Some(taffy_justify(props.justify));
            style.align_items = Some(taffy_align_items(props.align));
            style.gap = TaffySize {
                width: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
                height: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
            };
            style.padding = TaffyRect {
                left: LengthPercentage::length(scale_nonneg_px(props.padding.left, sf)),
                right: LengthPercentage::length(scale_nonneg_px(props.padding.right, sf)),
                top: LengthPercentage::length(scale_nonneg_px(props.padding.top, sf)),
                bottom: LengthPercentage::length(scale_nonneg_px(props.padding.bottom, sf)),
            };

            let children = tree.children(node).to_vec();
            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                build_flow_subtree(
                    engine,
                    app,
                    tree,
                    window,
                    sf,
                    ParentLayoutKind::Flex {
                        direction: props.direction,
                    },
                    child,
                );
            }
        }
        Some(ElementInstance::RovingFlex(props)) => {
            let props = props.flex;
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Flex,
                root_override_size,
            );
            style.flex_direction = match props.direction {
                fret_core::Axis::Horizontal => FlexDirection::Row,
                fret_core::Axis::Vertical => FlexDirection::Column,
            };
            style.flex_wrap = if props.wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            };
            style.justify_content = Some(taffy_justify(props.justify));
            style.align_items = Some(taffy_align_items(props.align));
            style.gap = TaffySize {
                width: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
                height: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
            };
            style.padding = TaffyRect {
                left: LengthPercentage::length(scale_nonneg_px(props.padding.left, sf)),
                right: LengthPercentage::length(scale_nonneg_px(props.padding.right, sf)),
                top: LengthPercentage::length(scale_nonneg_px(props.padding.top, sf)),
                bottom: LengthPercentage::length(scale_nonneg_px(props.padding.bottom, sf)),
            };

            let children = tree.children(node).to_vec();
            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                build_flow_subtree(
                    engine,
                    app,
                    tree,
                    window,
                    sf,
                    ParentLayoutKind::Flex {
                        direction: props.direction,
                    },
                    child,
                );
            }
        }
        Some(ElementInstance::Grid(props)) => {
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Grid,
                root_override_size,
            );
            style.justify_content = Some(taffy_justify(props.justify));
            style.align_items = Some(taffy_align_items(props.align));
            style.gap = TaffySize {
                width: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
                height: LengthPercentage::length(scale_nonneg_px(props.gap, sf)),
            };
            style.padding = TaffyRect {
                left: LengthPercentage::length(scale_nonneg_px(props.padding.left, sf)),
                right: LengthPercentage::length(scale_nonneg_px(props.padding.right, sf)),
                top: LengthPercentage::length(scale_nonneg_px(props.padding.top, sf)),
                bottom: LengthPercentage::length(scale_nonneg_px(props.padding.bottom, sf)),
            };
            style.grid_template_columns = taffy::style_helpers::evenly_sized_tracks(props.cols);
            style.grid_template_rows = props
                .rows
                .map(taffy::style_helpers::evenly_sized_tracks)
                .unwrap_or_default();

            let children = tree.children(node).to_vec();
            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                build_flow_subtree(engine, app, tree, window, sf, ParentLayoutKind::Grid, child);
            }
        }
        Some(
            instance @ (ElementInstance::Container(_)
            | ElementInstance::Pressable(_)
            | ElementInstance::Opacity(_)
            | ElementInstance::VisualTransform(_)
            | ElementInstance::RenderTransform(_)
            | ElementInstance::FractionalRenderTransform(_)
            | ElementInstance::Semantics(_)
            | ElementInstance::ViewCache(_)
            | ElementInstance::FocusScope(_)
            | ElementInstance::InteractivityGate(_)
            | ElementInstance::PointerRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::WheelRegion(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::Anchored(_)
            | ElementInstance::Stack(_)),
        ) if !tree.children(node).is_empty()
            && (!matches!(&instance, ElementInstance::HoverRegion(_))
                || !hover_region_has_absolute_child(app, tree, window, node)) =>
        {
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Grid,
                root_override_size,
            );
            // Use `1fr` tracks so percent/fill sizing resolves against the wrapper's definite box
            // when available, without forcing auto-sized children to stretch.
            style.grid_template_columns = vec![GridTemplateComponent::Single(
                taffy::style_helpers::flex(1.0),
            )];
            style.grid_template_rows = vec![GridTemplateComponent::Single(
                taffy::style_helpers::flex(1.0),
            )];
            style.align_items = Some(AlignItems::FlexStart);
            style.justify_items = Some(AlignItems::FlexStart);
            style.justify_content = Some(JustifyContent::FlexStart);

            if let Some(props) = element_record_for_node(app, window, node).and_then(|r| {
                if let ElementInstance::Container(p) = r.instance {
                    Some(p)
                } else {
                    None
                }
            }) {
                apply_container_insets(&mut style, &props, sf);
            }

            let children = tree.children(node).to_vec();

            // If this wrapper sizes itself to content (`Auto`) but its *flow child* requests `Fill`,
            // promote the wrapper to `Fill` as well.
            //
            // This avoids Taffy collapsing `%` sizing to zero when the containing block size is
            // otherwise undetermined, and matches the Fret contract that `Fill` only resolves when
            // an ancestor provides definite space.
            //
            // Practical impact: shadcn-like compositions commonly put `w_full()/h_full()` on inner
            // stacks, expecting the outer wrapper (CardHeader/CardContent, etc) to fill the card.
            let wrapper_style = layout_style_for_node(app, window, node);
            let mut has_flow_child_fill_w = false;
            let mut has_flow_child_fill_h = false;
            for &child in &children {
                let child_style = layout_style_for_node(app, window, child);
                if child_style.position == crate::element::PositionStyle::Absolute {
                    continue;
                }
                has_flow_child_fill_w |=
                    matches!(child_style.size.width, crate::element::Length::Fill);
                has_flow_child_fill_h |=
                    matches!(child_style.size.height, crate::element::Length::Fill);
            }
            if matches!(wrapper_style.size.width, crate::element::Length::Auto)
                && has_flow_child_fill_w
            {
                style.size.width = Dimension::percent(1.0);
            }
            if matches!(wrapper_style.size.height, crate::element::Length::Auto)
                && has_flow_child_fill_h
            {
                style.size.height = Dimension::percent(1.0);
            }

            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                build_flow_subtree(
                    engine,
                    app,
                    tree,
                    window,
                    sf,
                    ParentLayoutKind::Overlay,
                    child,
                );
            }
        }
        Some(ElementInstance::Scroll(_)) => {
            let layout_style = layout_style_for_node(app, window, node);
            let mut style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Block,
                root_override_size,
            );
            let mut measured = true;

            let has_definite_size =
                !matches!(layout_style.size.width, crate::element::Length::Auto)
                    && !matches!(layout_style.size.height, crate::element::Length::Auto);
            if has_definite_size || layout_style.flex.grow > 0.0 {
                // Barrier elements like Scroll/VirtualList frequently participate in flex layouts
                // as "fill the remaining space" items. When `flex-basis` is `Auto`, Taffy uses the
                // item's intrinsic size, which triggers our (potentially expensive) widget-level
                // measurement even though the final size is dominated by `flex-grow`.
                //
                // Prefer `flex-basis: 0` so the flex distribution is determined by grow factors
                // and the parent can size the barrier without invoking intrinsic measurement.
                if matches!(parent_kind, ParentLayoutKind::Flex { .. })
                    && matches!(layout_style.flex.basis, crate::element::Length::Auto)
                {
                    style.flex_basis = Dimension::length(0.0);
                }
                measured = false;
            }
            engine.set_style(node, style);
            engine.set_children(node, &[]);
            engine.set_measured(node, measured);

            // Barriers are explicit layout systems and must not couple their children into the
            // parent's flow solve, but we still want the engine to retain stable identity for the
            // mounted subtree across frames (GPUI-aligned request/build phase).
            let children = tree.children(node).to_vec();
            for child in children {
                build_flow_subtree(engine, app, tree, window, sf, ParentLayoutKind::Root, child);
            }
        }
        Some(ElementInstance::VirtualList(_) | ElementInstance::ResizablePanelGroup(_)) => {
            let style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Block,
                root_override_size,
            );
            engine.set_style(node, style);
            engine.set_children(node, &[]);
            engine.set_measured(node, true);

            // Barriers are explicit layout systems and must not couple their children into the
            // parent's flow solve, but we still want the engine to retain stable identity for the
            // mounted subtree across frames (GPUI-aligned request/build phase).
            let children = tree.children(node).to_vec();
            for child in children {
                build_flow_subtree(engine, app, tree, window, sf, ParentLayoutKind::Root, child);
            }
        }
        _ => {
            let style = style_for_item_in_parent(
                app,
                window,
                sf,
                parent_kind,
                node,
                Display::Block,
                root_override_size,
            );
            engine.set_style(node, style);
            engine.set_children(node, &[]);
            engine.set_measured(node, true);
        }
    }
}

fn hover_region_has_absolute_child<H: UiHost>(
    app: &mut H,
    tree: &UiTree<H>,
    window: AppWindowId,
    node: NodeId,
) -> bool {
    tree.children(node).iter().copied().any(|child| {
        layout_style_for_node(app, window, child).position
            == crate::element::PositionStyle::Absolute
    })
}

fn style_for_item_in_parent<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    scale_factor: f32,
    parent_kind: ParentLayoutKind,
    node: NodeId,
    display: Display,
    root_override_size: Option<Size>,
) -> Style {
    let sf = sanitize_scale_factor(scale_factor);
    let layout_style = layout_style_for_node(app, window, node);

    let mut min_w = layout_style.size.min_width.map(|p| p.0);
    let mut min_h = layout_style.size.min_height.map(|p| p.0);
    if let ParentLayoutKind::Flex { direction } = parent_kind {
        let spacer_min = element_record_for_node(app, window, node).and_then(|r| {
            if let ElementInstance::Spacer(p) = r.instance {
                Some(p.min)
            } else {
                None
            }
        });
        if let Some(min) = spacer_min {
            let min = min.0.max(0.0);
            match direction {
                fret_core::Axis::Horizontal => {
                    min_w = Some(min_w.unwrap_or(0.0).max(min));
                }
                fret_core::Axis::Vertical => {
                    min_h = Some(min_h.unwrap_or(0.0).max(min));
                }
            }
        }
    }

    let mut style = Style {
        display,
        position: taffy_position(layout_style.position),
        inset: taffy_rect_lpa_from_inset(scale_factor, layout_style.position, layout_style.inset),
        size: TaffySize {
            width: taffy_dimension(scale_factor, layout_style.size.width),
            height: taffy_dimension(scale_factor, layout_style.size.height),
        },
        aspect_ratio: layout_style.aspect_ratio,
        min_size: TaffySize {
            width: min_w
                .map(|v| Dimension::length(v * sf))
                .unwrap_or_else(Dimension::auto),
            height: min_h
                .map(|v| Dimension::length(v * sf))
                .unwrap_or_else(Dimension::auto),
        },
        max_size: TaffySize {
            width: layout_style
                .size
                .max_width
                .map(|p| Dimension::length(p.0 * sf))
                .unwrap_or_else(Dimension::auto),
            height: layout_style
                .size
                .max_height
                .map(|p| Dimension::length(p.0 * sf))
                .unwrap_or_else(Dimension::auto),
        },
        margin: taffy_rect_lpa_from_margin_edges(scale_factor, layout_style.margin),
        ..Default::default()
    };

    if layout_style.position == crate::element::PositionStyle::Absolute {
        if matches!(layout_style.size.width, crate::element::Length::Fill)
            && layout_style.inset.left.is_some()
            && layout_style.inset.right.is_some()
        {
            style.size.width = Dimension::auto();
        }
        if matches!(layout_style.size.height, crate::element::Length::Fill)
            && layout_style.inset.top.is_some()
            && layout_style.inset.bottom.is_some()
        {
            style.size.height = Dimension::auto();
        }
    }

    match parent_kind {
        ParentLayoutKind::Flex { .. } => {
            style.flex_grow = layout_style.flex.grow;
            style.flex_shrink = layout_style.flex.shrink;
            style.flex_basis = taffy_dimension(scale_factor, layout_style.flex.basis);
            style.align_self = layout_style.flex.align_self.map(taffy_align_self);
        }
        ParentLayoutKind::Grid => {
            style.grid_column = taffy_grid_line(layout_style.grid.column);
            style.grid_row = taffy_grid_line(layout_style.grid.row);
        }
        ParentLayoutKind::PassthroughOverlayStretch
        | ParentLayoutKind::PassthroughOverlayNoStretch
        | ParentLayoutKind::Overlay => {
            style.grid_column = overlay_grid_line();
            style.grid_row = overlay_grid_line();
        }
        ParentLayoutKind::Root => {}
    }

    match parent_kind {
        ParentLayoutKind::PassthroughOverlayStretch => {
            // Passthrough wrappers should remain layout-transparent: their single child should
            // inherit the wrapper's resolved box when possible, without forcing intrinsic leaves
            // (e.g. Spacer) to stretch.
            let instance = element_record_for_node(app, window, node).map(|r| r.instance);
            match instance {
                Some(ElementInstance::Spacer(_)) => {
                    style.align_self = Some(AlignSelf::FlexStart);
                    style.justify_self = Some(AlignSelf::FlexStart);
                }
                _ => {
                    style.align_self = Some(AlignSelf::Stretch);
                    style.justify_self = Some(AlignSelf::Stretch);
                }
            }
        }
        ParentLayoutKind::PassthroughOverlayNoStretch => {
            // Behavioral wrappers (Semantics/Opacity/Container/...) should not stretch their
            // single child; leave sizing to the child so shrink-wrapped widgets (e.g. button
            // triggers) keep intrinsic bounds.
            //
            // Note: per-item stretching is controlled by the wrapper node's `align_items` /
            // `justify_items` (set in `build_flow_subtree_impl`). Keeping `*_self` unset here lets
            // the wrapper decide when it's appropriate to stretch layout containers vs. keep
            // intrinsic sizing.
        }
        _ => {}
    }

    if let Some(size) = root_override_size {
        style.size.width = Dimension::length(scale_nonneg(size.width.0, scale_factor));
        style.size.height = Dimension::length(scale_nonneg(size.height.0, scale_factor));
        style.max_size.width = Dimension::length(scale_nonneg(size.width.0, scale_factor));
        style.max_size.height = Dimension::length(scale_nonneg(size.height.0, scale_factor));
    }

    style
}

fn overlay_grid_line() -> TaffyLine<GridPlacement> {
    TaffyLine {
        start: taffy::style_helpers::line::<GridPlacement>(1),
        end: GridPlacement::Span(1),
    }
}

fn passthrough_wrapper_child<H: UiHost>(
    app: &mut H,
    tree: &UiTree<H>,
    window: AppWindowId,
    node: NodeId,
) -> Option<(NodeId, ParentLayoutKind)> {
    let layout_style = layout_style_for_node(app, window, node);
    if layout_style.position != crate::element::PositionStyle::Static {
        return None;
    }
    if layout_style.inset.left.is_some()
        || layout_style.inset.right.is_some()
        || layout_style.inset.top.is_some()
        || layout_style.inset.bottom.is_some()
    {
        return None;
    }

    let children = tree.children(node);
    if children.len() != 1 {
        return None;
    }
    let child = children[0];
    let child_style = layout_style_for_node(app, window, child);
    if child_style.position != crate::element::PositionStyle::Static {
        return None;
    }

    let instance = element_record_for_node(app, window, node).map(|r| r.instance)?;
    match instance {
        ElementInstance::InteractivityGate(gate) if gate.present => {
            Some((child, ParentLayoutKind::PassthroughOverlayNoStretch))
        }
        ElementInstance::Pressable(_) => Some((child, ParentLayoutKind::PassthroughOverlayStretch)),
        ElementInstance::Container(_)
        | ElementInstance::PointerRegion(_)
        | ElementInstance::HoverRegion(_)
        | ElementInstance::WheelRegion(_)
        | ElementInstance::Opacity(_)
        | ElementInstance::VisualTransform(_)
        | ElementInstance::RenderTransform(_)
        | ElementInstance::FractionalRenderTransform(_)
        | ElementInstance::Semantics(_)
        | ElementInstance::LayoutQueryRegion(_)
        | ElementInstance::ViewCache(_)
        | ElementInstance::FocusScope(_) => {
            Some((child, ParentLayoutKind::PassthroughOverlayNoStretch))
        }
        _ => None,
    }
}

fn taffy_position(position: crate::element::PositionStyle) -> TaffyPosition {
    match position {
        crate::element::PositionStyle::Static | crate::element::PositionStyle::Relative => {
            TaffyPosition::Relative
        }
        crate::element::PositionStyle::Absolute => TaffyPosition::Absolute,
    }
}

fn taffy_dimension(scale_factor: f32, length: crate::element::Length) -> Dimension {
    let sf = sanitize_scale_factor(scale_factor);
    match length {
        crate::element::Length::Auto => Dimension::auto(),
        crate::element::Length::Fill => Dimension::percent(1.0),
        crate::element::Length::Px(px) => Dimension::length(px.0 * sf),
    }
}

fn taffy_lpa(scale_factor: f32, px: Option<Px>) -> LengthPercentageAuto {
    let sf = sanitize_scale_factor(scale_factor);
    match px {
        Some(px) => LengthPercentageAuto::length(px.0 * sf),
        None => LengthPercentageAuto::auto(),
    }
}

fn taffy_rect_lpa_from_inset(
    scale_factor: f32,
    position: crate::element::PositionStyle,
    inset: crate::element::InsetStyle,
) -> TaffyRect<LengthPercentageAuto> {
    if position == crate::element::PositionStyle::Static {
        return TaffyRect {
            left: LengthPercentageAuto::auto(),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
    }
    TaffyRect {
        left: taffy_lpa(scale_factor, inset.left),
        right: taffy_lpa(scale_factor, inset.right),
        top: taffy_lpa(scale_factor, inset.top),
        bottom: taffy_lpa(scale_factor, inset.bottom),
    }
}

fn taffy_lpa_margin_edge(
    scale_factor: f32,
    edge: crate::element::MarginEdge,
) -> LengthPercentageAuto {
    let sf = sanitize_scale_factor(scale_factor);
    match edge {
        crate::element::MarginEdge::Px(px) => LengthPercentageAuto::length(px.0 * sf),
        crate::element::MarginEdge::Auto => LengthPercentageAuto::auto(),
    }
}

fn taffy_rect_lpa_from_margin_edges(
    scale_factor: f32,
    margin: crate::element::MarginEdges,
) -> TaffyRect<LengthPercentageAuto> {
    TaffyRect {
        left: taffy_lpa_margin_edge(scale_factor, margin.left),
        right: taffy_lpa_margin_edge(scale_factor, margin.right),
        top: taffy_lpa_margin_edge(scale_factor, margin.top),
        bottom: taffy_lpa_margin_edge(scale_factor, margin.bottom),
    }
}

fn taffy_grid_line(line: crate::element::GridLine) -> TaffyLine<GridPlacement> {
    let start = line
        .start
        .map(taffy::style_helpers::line::<GridPlacement>)
        .unwrap_or(GridPlacement::Auto);
    let end = line
        .span
        .map(GridPlacement::Span)
        .unwrap_or(GridPlacement::Auto);
    TaffyLine { start, end }
}

fn taffy_align_items(align: crate::element::CrossAlign) -> AlignItems {
    match align {
        crate::element::CrossAlign::Start => AlignItems::FlexStart,
        crate::element::CrossAlign::Center => AlignItems::Center,
        crate::element::CrossAlign::End => AlignItems::FlexEnd,
        crate::element::CrossAlign::Stretch => AlignItems::Stretch,
    }
}

fn taffy_align_self(align: crate::element::CrossAlign) -> AlignSelf {
    match align {
        crate::element::CrossAlign::Start => AlignSelf::FlexStart,
        crate::element::CrossAlign::Center => AlignSelf::Center,
        crate::element::CrossAlign::End => AlignSelf::FlexEnd,
        crate::element::CrossAlign::Stretch => AlignSelf::Stretch,
    }
}

fn taffy_justify(justify: crate::element::MainAlign) -> JustifyContent {
    match justify {
        crate::element::MainAlign::Start => JustifyContent::FlexStart,
        crate::element::MainAlign::Center => JustifyContent::Center,
        crate::element::MainAlign::End => JustifyContent::FlexEnd,
        crate::element::MainAlign::SpaceBetween => JustifyContent::SpaceBetween,
        crate::element::MainAlign::SpaceAround => JustifyContent::SpaceAround,
        crate::element::MainAlign::SpaceEvenly => JustifyContent::SpaceEvenly,
    }
}

fn sanitize_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn scale_nonneg(value: f32, scale_factor: f32) -> f32 {
    let sf = sanitize_scale_factor(scale_factor);
    value.max(0.0) * sf
}

fn scale_nonneg_px(value: Px, scale_factor: f32) -> f32 {
    scale_nonneg(value.0, scale_factor)
}
