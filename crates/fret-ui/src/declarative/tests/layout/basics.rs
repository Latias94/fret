use super::*;

#[test]
fn fill_only_resolves_under_definite_available_space_in_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "fill-measure",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.container(props, |cx| vec![cx.text("x")])]
        },
    );
    ui.set_root(root);

    let container = ui.children(root)[0];

    let min_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MinContent, AvailableSpace::MinContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, container, min_constraints, 1.0);
    assert!(
        (measured.width.0 - 10.0).abs() < 0.01,
        "expected Fill to behave like auto under MinContent, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 10.0).abs() < 0.01,
        "expected Fill to behave like auto under MinContent, got {:?}",
        measured.height
    );

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, container, max_constraints, 1.0);
    assert!(
        (measured.width.0 - 10.0).abs() < 0.01,
        "expected Fill to behave like auto under MaxContent, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 10.0).abs() < 0.01,
        "expected Fill to behave like auto under MaxContent, got {:?}",
        measured.height
    );

    let definite_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(123.0)),
            AvailableSpace::Definite(Px(45.0)),
        ),
    );
    let measured = ui.measure_in(&mut app, &mut text, container, definite_constraints, 1.0);
    assert!(
        (measured.width.0 - 123.0).abs() < 0.01,
        "expected Fill to resolve width under definite available space, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 45.0).abs() < 0.01,
        "expected Fill to resolve height under definite available space, got {:?}",
        measured.height
    );
}

#[test]
fn fraction_only_resolves_under_definite_available_space_in_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "fraction-measure",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.size.width = Length::Fraction(0.5);
            props.layout.size.height = Length::Fraction(0.25);
            vec![cx.container(props, |cx| vec![cx.text("x")])]
        },
    );
    ui.set_root(root);

    let container = ui.children(root)[0];

    let min_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MinContent, AvailableSpace::MinContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, container, min_constraints, 1.0);
    assert!(
        (measured.width.0 - 10.0).abs() < 0.01,
        "expected Fraction to behave like auto under MinContent, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 10.0).abs() < 0.01,
        "expected Fraction to behave like auto under MinContent, got {:?}",
        measured.height
    );

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, container, max_constraints, 1.0);
    assert!(
        (measured.width.0 - 10.0).abs() < 0.01,
        "expected Fraction to behave like auto under MaxContent, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 10.0).abs() < 0.01,
        "expected Fraction to behave like auto under MaxContent, got {:?}",
        measured.height
    );

    let definite_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(200.0)),
            AvailableSpace::Definite(Px(80.0)),
        ),
    );
    let measured = ui.measure_in(&mut app, &mut text, container, definite_constraints, 1.0);
    assert!(
        (measured.width.0 - 100.0).abs() < 0.01,
        "expected Fraction to resolve width under definite available space, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 20.0).abs() < 0.01,
        "expected Fraction to resolve height under definite available space, got {:?}",
        measured.height
    );
}

#[test]
fn flex_fraction_basis_and_fill_basis_do_not_collapse_under_min_content_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "flex-fraction-basis-min-content",
        |cx| {
            let mut props = crate::element::FlexProps::default();
            props.direction = fret_core::Axis::Horizontal;

            let mut child = crate::element::ContainerProps::default();
            child.layout.size.height = Length::Px(Px(10.0));
            child.layout.aspect_ratio = Some(1.0);
            // If percent/fraction basis is treated as 0 under MinContent, this will collapse to 0.
            child.layout.flex.basis = Length::Fill;
            let fill_child = cx.container(child, |cx| vec![cx.text("x")]);

            let mut child = crate::element::ContainerProps::default();
            child.layout.size.height = Length::Px(Px(10.0));
            child.layout.aspect_ratio = Some(1.0);
            child.layout.flex.basis = Length::Fraction(0.5);
            let fraction_child = cx.container(child, |cx| vec![cx.text("x")]);

            vec![cx.flex(props, |_cx| vec![fill_child, fraction_child])]
        },
    );
    ui.set_root(root);

    let flex = ui.children(root)[0];
    let constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MinContent, AvailableSpace::MinContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, flex, constraints, 1.0);
    assert!(
        measured.width.0 > 0.01,
        "expected flex width to be non-zero under MinContent, got {:?}",
        measured.width
    );
    assert!(
        measured.height.0 > 0.01,
        "expected flex height to be non-zero under MinContent, got {:?}",
        measured.height
    );
}

#[test]
fn min_max_fraction_only_resolve_under_definite_available_space_in_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "min-max-fraction-measure",
        |cx| {
            let mut max_props = crate::element::ContainerProps::default();
            max_props.layout.size.width = Length::Px(Px(150.0));
            max_props.layout.size.max_width = Some(Length::Fraction(0.5));
            let max_node = cx.container(max_props, |cx| vec![cx.text("x")]);

            let mut min_props = crate::element::ContainerProps::default();
            min_props.layout.size.width = Length::Px(Px(10.0));
            min_props.layout.size.min_width = Some(Length::Fraction(0.5));
            let min_node = cx.container(min_props, |cx| vec![cx.text("x")]);

            vec![max_node, min_node]
        },
    );
    ui.set_root(root);

    let max_node = ui.children(root)[0];
    let min_node = ui.children(root)[1];

    let min_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MinContent, AvailableSpace::MinContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, max_node, min_constraints, 1.0);
    assert!(
        (measured.width.0 - 150.0).abs() < 0.01,
        "expected max_width Fraction to behave like auto under MinContent, got {:?}",
        measured.width
    );
    let measured = ui.measure_in(&mut app, &mut text, min_node, min_constraints, 1.0);
    assert!(
        (measured.width.0 - 10.0).abs() < 0.01,
        "expected min_width Fraction to behave like auto under MinContent, got {:?}",
        measured.width
    );

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, max_node, max_constraints, 1.0);
    assert!(
        (measured.width.0 - 150.0).abs() < 0.01,
        "expected max_width Fraction to behave like auto under MaxContent, got {:?}",
        measured.width
    );
    let measured = ui.measure_in(&mut app, &mut text, min_node, max_constraints, 1.0);
    assert!(
        (measured.width.0 - 10.0).abs() < 0.01,
        "expected min_width Fraction to behave like auto under MaxContent, got {:?}",
        measured.width
    );

    let definite_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(200.0)),
            AvailableSpace::Definite(Px(80.0)),
        ),
    );
    let measured = ui.measure_in(&mut app, &mut text, max_node, definite_constraints, 1.0);
    assert!(
        (measured.width.0 - 100.0).abs() < 0.01,
        "expected max_width Fraction to resolve under definite available space, got {:?}",
        measured.width
    );
    let measured = ui.measure_in(&mut app, &mut text, min_node, definite_constraints, 1.0);
    assert!(
        (measured.width.0 - 100.0).abs() < 0.01,
        "expected min_width Fraction to resolve under definite available space, got {:?}",
        measured.width
    );
}

#[test]
fn absolute_inset_fraction_resolves_against_containing_block() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "absolute-inset-fraction",
        |cx| {
            let mut outer = crate::element::ContainerProps::default();
            outer.layout.size.width = Length::Fill;
            outer.layout.size.height = Length::Fill;
            outer.layout.position = crate::element::PositionStyle::Relative;

            let mut child = crate::element::ContainerProps::default();
            child.layout.position = crate::element::PositionStyle::Absolute;
            child.layout.inset.left = crate::element::InsetEdge::Fraction(0.25);
            child.layout.inset.top = crate::element::InsetEdge::Fraction(0.1);
            child.layout.size.width = Length::Px(Px(20.0));
            child.layout.size.height = Length::Px(Px(10.0));

            vec![cx.container(outer, move |cx| vec![cx.container(child, |_cx| Vec::new())])]
        },
    );
    ui.set_root(root);

    let outer_node = ui.children(root)[0];
    let child_node = ui.children(outer_node)[0];

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let child_bounds = ui.debug_node_bounds(child_node).expect("child bounds");
    assert!(
        (child_bounds.origin.x.0 - 50.0).abs() < 0.01,
        "expected left=25% of 200px == 50px, got {:?}",
        child_bounds.origin.x
    );
    assert!(
        (child_bounds.origin.y.0 - 10.0).abs() < 0.01,
        "expected top=10% of 100px == 10px, got {:?}",
        child_bounds.origin.y
    );
    assert!(
        (child_bounds.size.width.0 - 20.0).abs() < 0.01,
        "expected width=20px, got {:?}",
        child_bounds.size.width
    );
    assert!(
        (child_bounds.size.height.0 - 10.0).abs() < 0.01,
        "expected height=10px, got {:?}",
        child_bounds.size.height
    );
}

#[test]
fn spacing_fraction_only_resolve_under_definite_available_space_in_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "spacing-fraction-measure",
        |cx| {
            let mut props = crate::element::RowProps::default();
            props.gap = crate::element::SpacingLength::Fraction(0.1);
            props.padding = crate::element::SpacingEdges {
                left: crate::element::SpacingLength::Fraction(0.1),
                right: crate::element::SpacingLength::Fraction(0.1),
                top: crate::element::SpacingLength::Fraction(0.1),
                bottom: crate::element::SpacingLength::Fraction(0.1),
            };
            props.layout.size.width = Length::Auto;
            props.layout.size.height = Length::Auto;

            vec![cx.row(props, |cx| vec![cx.text("a"), cx.text("b")])]
        },
    );
    ui.set_root(root);

    let row_node = ui.children(root)[0];

    let intrinsic_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MinContent, AvailableSpace::MinContent),
    );
    let measured = ui.measure_in(&mut app, &mut text, row_node, intrinsic_constraints, 1.0);
    assert!(
        (measured.width.0 - 20.0).abs() < 0.01,
        "expected percent spacing to resolve to 0 under MinContent, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 10.0).abs() < 0.01,
        "expected percent spacing to resolve to 0 under MinContent, got {:?}",
        measured.height
    );

    let definite_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(200.0)),
            AvailableSpace::Definite(Px(80.0)),
        ),
    );
    let measured = ui.measure_in(&mut app, &mut text, row_node, definite_constraints, 1.0);
    assert!(
        (measured.width.0 - 76.0).abs() < 0.01,
        "expected percent spacing to resolve under definite available space, got {:?}",
        measured.width
    );
    assert!(
        (measured.height.0 - 50.0).abs() < 0.01,
        "expected percent spacing to resolve under definite available space, got {:?}",
        measured.height
    );
}

#[test]
fn row_justify_center_and_align_end_positions_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(20.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "row-align",
        |cx| {
            let mut props = crate::element::RowProps {
                gap: Px(5.0).into(),
                justify: MainAlign::Center,
                align: CrossAlign::End,
                ..Default::default()
            };
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            vec![cx.row(props, |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let row_node = ui.children(root)[0];
    let children = ui.children(row_node);
    assert_eq!(children.len(), 3);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");
    let b2 = ui.debug_node_bounds(children[2]).expect("child2 bounds");

    // Each text measures to 10x10. With gap=5 and width=100:
    // content_w = 3*10 + 2*5 = 40; remaining=60; center => start_offset=30.
    assert!((b0.origin.x.0 - 30.0).abs() < 0.01, "x0={:?}", b0.origin.x);
    assert!((b1.origin.x.0 - 45.0).abs() < 0.01, "x1={:?}", b1.origin.x);
    assert!((b2.origin.x.0 - 60.0).abs() < 0.01, "x2={:?}", b2.origin.x);

    // align-end with row height 20 => y = 0 + (20-10)=10.
    assert!((b0.origin.y.0 - 10.0).abs() < 0.01, "y0={:?}", b0.origin.y);
    assert!((b1.origin.y.0 - 10.0).abs() < 0.01, "y1={:?}", b1.origin.y);
    assert!((b2.origin.y.0 - 10.0).abs() < 0.01, "y2={:?}", b2.origin.y);
}

#[test]
fn flex_wrap_positions_children_on_multiple_rows() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(25.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "flex-wrap",
        |cx| {
            let mut props = crate::element::FlexProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.wrap = true;
            vec![cx.flex(props, |cx| {
                vec![
                    cx.text("a"),
                    cx.text("b"),
                    cx.text("c"),
                    cx.text("d"),
                    cx.text("e"),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 5);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");
    let b2 = ui.debug_node_bounds(children[2]).expect("child2 bounds");
    let b3 = ui.debug_node_bounds(children[3]).expect("child3 bounds");
    let b4 = ui.debug_node_bounds(children[4]).expect("child4 bounds");

    assert!((b0.origin.x.0 - 0.0).abs() < 0.01, "x0={:?}", b0.origin.x);
    assert!((b1.origin.x.0 - 10.0).abs() < 0.01, "x1={:?}", b1.origin.x);
    assert!((b2.origin.x.0 - 0.0).abs() < 0.01, "x2={:?}", b2.origin.x);
    assert!((b3.origin.x.0 - 10.0).abs() < 0.01, "x3={:?}", b3.origin.x);
    assert!((b4.origin.x.0 - 0.0).abs() < 0.01, "x4={:?}", b4.origin.x);

    assert!((b0.origin.y.0 - 0.0).abs() < 0.01, "y0={:?}", b0.origin.y);
    assert!((b1.origin.y.0 - 0.0).abs() < 0.01, "y1={:?}", b1.origin.y);
    assert!((b2.origin.y.0 - 10.0).abs() < 0.01, "y2={:?}", b2.origin.y);
    assert!((b3.origin.y.0 - 10.0).abs() < 0.01, "y3={:?}", b3.origin.y);
    assert!((b4.origin.y.0 - 20.0).abs() < 0.01, "y4={:?}", b4.origin.y);
}

#[test]
fn layout_viewport_root_defers_child_layout_until_after_parent() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct ParentWithDeferredViewport {
        viewport: Rect,
        child: NodeId,
        saw_default_bounds: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for ParentWithDeferredViewport {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            let bounds = cx
                .tree
                .debug_node_bounds(self.child)
                .unwrap_or_else(|| Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default()));
            self.saw_default_bounds
                .store(bounds.size == Size::default(), Ordering::Relaxed);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-child",
        |cx| vec![cx.text("child")],
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let saw_default_bounds = Arc::new(AtomicBool::new(false));
    let parent = ui.create_node(ParentWithDeferredViewport {
        viewport,
        child,
        saw_default_bounds: saw_default_bounds.clone(),
    });
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        saw_default_bounds.load(Ordering::Relaxed),
        "expected viewport child to be laid out after parent, not during Parent.layout()"
    );

    let child_bounds = ui.debug_node_bounds(child).expect("child bounds");
    assert!((child_bounds.origin.x.0 - viewport.origin.x.0).abs() < 0.01);
    assert!((child_bounds.size.width.0 - viewport.size.width.0).abs() < 0.01);
}

#[test]
fn opacity_does_not_stretch_spacer_child_in_engine_tree() {
    struct RegistersViewportRoot {
        viewport: Rect,
    }

    impl<H: UiHost> Widget<H> for RegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let child = cx.children[0];
            let _ = cx.layout_viewport_root(child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(140.0)),
    );
    let viewport = Rect::new(
        fret_core::Point::new(Px(7.0), Px(11.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let mut text = FakeTextService::default();

    let child_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "opacity-engine-no-stretch",
        |cx| {
            let mut props = crate::element::OpacityProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.opacity_props(props, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let opacity = ui.children(child_root)[0];
    let spacer = ui.children(opacity)[0];

    let opacity_bounds = ui.debug_node_bounds(opacity).expect("opacity bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer).expect("spacer bounds");

    assert_eq!(opacity_bounds, viewport);
    assert_eq!(spacer_bounds.origin, viewport.origin);
    assert!(spacer_bounds.size.width.0.abs() < 0.01);
    assert!(spacer_bounds.size.height.0.abs() < 0.01);
}

#[test]
fn visual_transform_does_not_stretch_spacer_child_in_engine_tree() {
    struct RegistersViewportRoot {
        viewport: Rect,
    }

    impl<H: UiHost> Widget<H> for RegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let child = cx.children[0];
            let _ = cx.layout_viewport_root(child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(140.0)),
    );
    let viewport = Rect::new(
        fret_core::Point::new(Px(7.0), Px(11.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let mut text = FakeTextService::default();

    let child_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "visual-transform-engine-no-stretch",
        |cx| {
            let mut props = crate::element::VisualTransformProps {
                transform: fret_core::Transform2D::scale_uniform(1.0),
                ..Default::default()
            };
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.visual_transform_props(props, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let transform = ui.children(child_root)[0];
    let spacer = ui.children(transform)[0];

    let transform_bounds = ui.debug_node_bounds(transform).expect("transform bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer).expect("spacer bounds");

    assert_eq!(transform_bounds, viewport);
    assert_eq!(spacer_bounds.origin, viewport.origin);
    assert!(spacer_bounds.size.width.0.abs() < 0.01);
    assert!(spacer_bounds.size.height.0.abs() < 0.01);
}

#[test]
fn wrapper_chain_padding_is_applied_via_engine_rects() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let flex = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            align: CrossAlign::Start,
            ..Default::default()
        };

        let outer = crate::element::ContainerProps {
            padding: fret_core::Edges {
                left: Px(8.0),
                right: Px(4.0),
                top: Px(6.0),
                bottom: Px(2.0),
            }
            .into(),
            ..Default::default()
        };

        vec![cx.flex(flex, |cx| {
            vec![cx.container(outer, |cx| {
                vec![cx.opacity(1.0, |cx| {
                    vec![
                        cx.semantics(crate::element::SemanticsProps::default(), |cx| {
                            let inner = crate::element::ContainerProps {
                                layout: crate::element::LayoutStyle {
                                    size: crate::element::SizeStyle {
                                        width: Length::Px(Px(10.0)),
                                        height: Length::Px(Px(10.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            };
                            vec![cx.container(inner, |_cx| Vec::new())]
                        }),
                    ]
                })]
            })]
        })]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "wrapper-chain-padding",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let outer_container = ui.children(flex_node)[0];
    let opacity = ui.children(outer_container)[0];
    let semantics = ui.children(opacity)[0];
    let inner_container = ui.children(semantics)[0];

    let outer_bounds = ui
        .debug_node_bounds(outer_container)
        .expect("outer container bounds");
    let inner_bounds = ui
        .debug_node_bounds(inner_container)
        .expect("inner container bounds");

    assert!((inner_bounds.origin.x.0 - (outer_bounds.origin.x.0 + 8.0)).abs() < 0.01);
    assert!((inner_bounds.origin.y.0 - (outer_bounds.origin.y.0 + 6.0)).abs() < 0.01);
    assert!((inner_bounds.size.width.0 - 10.0).abs() < 0.01);
    assert!((inner_bounds.size.height.0 - 10.0).abs() < 0.01);

    assert!((outer_bounds.size.width.0 - (10.0 + 8.0 + 4.0)).abs() < 0.01);
    assert!((outer_bounds.size.height.0 - (10.0 + 6.0 + 2.0)).abs() < 0.01);
}

#[test]
fn image_paints_image_op() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    let img = fret_core::ImageId::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-image",
        |cx| {
            let mut p = crate::element::ImageProps::new(img);
            p.layout.size.width = crate::element::Length::Px(Px(160.0));
            p.layout.size.height = crate::element::Length::Px(Px(80.0));
            vec![cx.image_props(p)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Image { image, .. } if *image == img)),
        "expected an Image op for the declarative image element"
    );
}

#[test]
fn overflow_clip_pushes_clip_rect_for_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-clip",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.overflow = crate::element::Overflow::Clip;
            vec![cx.container(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let pushes = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
        .count();
    let pops = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PopClip))
        .count();

    assert!(
        pushes >= 1,
        "expected container overflow clip to push a clip rect"
    );
    assert!(
        pops >= 1,
        "expected container overflow clip to pop a clip rect"
    );
}

#[test]
fn overflow_clip_with_corner_radii_pushes_rounded_clip_rect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-clip-rounded",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.overflow = crate::element::Overflow::Clip;
            props.corner_radii = fret_core::Corners::all(Px(8.0));
            vec![cx.container(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::PushClipRRect { .. })),
        "expected container overflow clip + corner radii to push a rounded clip rect"
    );
}

#[test]
fn overflow_visible_does_not_push_clip_rect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-visible",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("child")])],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        !scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::PushClipRect { .. } | SceneOp::PushClipRRect { .. }
        )),
        "expected no clip ops by default"
    );
}

#[test]
fn fill_respects_max_width_constraint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-max-width",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Fill,
                            max_width: Some(crate::element::Length::Px(Px(100.0))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| vec![],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let rect = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(rect.size.width, Px(100.0));
}

#[test]
fn flex_child_margin_affects_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-flex-margin",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Px(Px(5.0));

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(15.0));
}

#[test]
fn flex_child_auto_margins_center_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-mx-auto",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));
                    a.layout.margin.left = crate::element::MarginEdge::Auto;
                    a.layout.margin.right = crate::element::MarginEdge::Auto;
                    vec![cx.container(a, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert_eq!(flex_bounds.size.width, Px(100.0));
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 1);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");

    assert_eq!(a_bounds.origin.x, Px(45.0));
}

#[test]
fn flex_child_negative_margin_shifts_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-negative-margin",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Px(Px(-5.0));

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(5.0));
}

#[test]
fn flex_margin_fraction_uses_containing_block_width_for_top() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-margin-fraction",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    gap: Px(0.0).into(),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l.size.height = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Fraction(0.25);
                    b.layout.margin.top = crate::element::MarginEdge::Fraction(0.1);

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);

    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin, Point::new(Px(0.0), Px(0.0)));
    assert_eq!(b_bounds.origin, Point::new(Px(50.0), Px(30.0)));
}

#[test]
fn grid_places_children_in_columns() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-grid",
        |cx| {
            vec![cx.grid(
                crate::element::GridProps {
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l.size.height = crate::element::Length::Fill;
                        l
                    },
                    cols: 2,
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Fill;
                    a.layout.size.height = crate::element::Length::Fill;

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Fill;
                    b.layout.size.height = crate::element::Length::Fill;

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let grid_node = ui.children(root)[0];
    let children = ui.children(grid_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(100.0));
    assert_eq!(a_bounds.size.width, Px(100.0));
    assert_eq!(b_bounds.size.width, Px(100.0));
}

#[test]
fn flex_defaults_to_fit_content_under_constraints() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "flex-fit",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(5.0).into(),
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)).into(),
                    ..Default::default()
                },
                |cx| vec![cx.text("a"), cx.text("b")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");

    // FakeTextService measures each text to 10x10. With gap=5 and padding (4,6):
    // inner_w = 10 + 5 + 10 = 25, outer_w = 25 + 8 = 33
    // inner_h = 10, outer_h = 10 + 12 = 22
    assert!(
        (flex_bounds.size.width.0 - 33.0).abs() < 0.01,
        "w={:?}",
        flex_bounds.size.width
    );
    assert!(
        (flex_bounds.size.height.0 - 22.0).abs() < 0.01,
        "h={:?}",
        flex_bounds.size.height
    );
}
