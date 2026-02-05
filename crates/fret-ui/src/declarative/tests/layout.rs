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
fn scroll_intrinsic_viewport_mode_does_not_measure_children() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

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
        "scroll-intrinsic-viewport-mode",
        |cx| {
            let mut props = crate::element::ScrollProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Viewport;
            vec![cx.scroll(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let _measured = ui.measure_in(&mut app, &mut text, scroll, max_constraints, 1.0);

    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll),
        0,
        "expected viewport-mode scroll intrinsic measurement to avoid measuring children"
    );
}

#[test]
fn scroll_intrinsic_content_mode_measures_children() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

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
        "scroll-intrinsic-content-mode",
        |cx| {
            let mut props = crate::element::ScrollProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;
            vec![cx.scroll(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let _measured = ui.measure_in(&mut app, &mut text, scroll, max_constraints, 1.0);

    assert!(
        ui.debug_measure_child_calls_for_parent(scroll) > 0,
        "expected content-mode scroll intrinsic measurement to measure children"
    );
}

#[test]
fn text_measurement_and_paint_agree_on_wrap_width_in_a_column() {
    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
        prepared: Vec<TextConstraints>,
        prepared_metrics: Vec<TextMetrics>,
    }

    impl RecordingTextService {
        fn fake_metrics(constraints: TextConstraints) -> TextMetrics {
            let w = constraints.max_width.unwrap_or(Px(1_000.0)).0.max(0.0);
            let lines = if matches!(
                constraints.wrap,
                fret_core::TextWrap::Word | fret_core::TextWrap::Grapheme
            ) && w < 60.0
            {
                2.0
            } else {
                1.0
            };
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0 * lines)),
                baseline: Px(8.0),
            }
        }
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            let metrics = Self::fake_metrics(constraints);
            self.prepared.push(constraints);
            self.prepared_metrics.push(metrics);
            (fret_core::TextBlobId::default(), metrics)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            _input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            Self::fake_metrics(constraints)
        }
    }

    impl fret_core::PathService for RecordingTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for RecordingTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-measure-paint-wrap-width",
        |cx| {
            let mut text_props = crate::element::TextProps::new("wrap me please");
            text_props.layout.size.width = Length::Px(Px(40.0));
            text_props.wrap = fret_core::TextWrap::Word;

            let mut sibling_props = crate::element::ContainerProps::default();
            sibling_props.layout.size.width = Length::Fill;
            sibling_props.layout.size.height = Length::Px(Px(10.0));

            vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                vec![
                    cx.text_props(text_props),
                    cx.container(sibling_props, |_| Vec::new()),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let col = ui.children(root)[0];
    let text_node = ui.children(col)[0];
    let sibling_node = ui.children(col)[1];
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
    let sibling_bounds = ui.debug_node_bounds(sibling_node).expect("sibling bounds");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let measured = services
        .measured
        .iter()
        .find(|c| c.max_width.is_some_and(|w| (w.0 - 40.0).abs() < 0.01))
        .copied()
        .expect("measured constraints");
    let prepared = services
        .prepared
        .iter()
        .find(|c| c.max_width.is_some_and(|w| (w.0 - 40.0).abs() < 0.01))
        .copied()
        .expect("prepared constraints");

    assert!(
        (measured.max_width.unwrap().0 - prepared.max_width.unwrap().0).abs() < 0.01,
        "expected measure/paint to use the same wrap width; measured={measured:?} prepared={prepared:?}"
    );

    let prepared_metrics = services
        .prepared_metrics
        .last()
        .copied()
        .expect("prepared metrics");
    assert!(
        sibling_bounds.origin.y.0 + 0.01 >= text_bounds.origin.y.0 + prepared_metrics.size.height.0,
        "expected following sibling to be laid out below painted text height; text={text_bounds:?} sibling={sibling_bounds:?} painted_h={:?}",
        prepared_metrics.size.height
    );
}

#[test]
fn text_measurement_and_paint_agree_on_overflow_and_scale_factor() {
    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
        prepared: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            self.prepared.push(constraints);
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            _input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            }
        }
    }

    impl fret_core::PathService for RecordingTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for RecordingTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-measure-paint-overflow-scale",
        |cx| {
            let mut text_props = crate::element::TextProps::new("ellipsis me please");
            text_props.layout.size.width = Length::Px(Px(40.0));
            text_props.wrap = fret_core::TextWrap::None;
            text_props.overflow = fret_core::TextOverflow::Ellipsis;

            vec![cx.text_props(text_props)]
        },
    );
    ui.set_root(root);

    let scale_factor = 1.25_f32;
    ui.layout_all(&mut app, &mut services, bounds, scale_factor);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

    let measured = services
        .measured
        .iter()
        .find(|c| c.max_width.is_some_and(|w| (w.0 - 40.0).abs() < 0.01))
        .copied()
        .expect("measured constraints");
    let prepared = services
        .prepared
        .iter()
        .find(|c| c.max_width.is_some_and(|w| (w.0 - 40.0).abs() < 0.01))
        .copied()
        .expect("prepared constraints");

    assert_eq!(measured.wrap, prepared.wrap);
    assert_eq!(measured.overflow, prepared.overflow);
    assert!(
        (measured.scale_factor - prepared.scale_factor).abs() < 1e-6,
        "expected measure/paint to use the same scale_factor; measured={measured:?} prepared={prepared:?}"
    );
    assert!(
        (measured.max_width.unwrap().0 - prepared.max_width.unwrap().0).abs() < 0.01,
        "expected measure/paint to use the same max_width; measured={measured:?} prepared={prepared:?}"
    );
}

#[test]
fn hover_region_reports_hovered_even_when_child_is_pressable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.hover_region(
            crate::element::HoverRegionProps::default(),
            |cx, hovered| {
                let trigger = cx
                    .pressable(crate::element::PressableProps::default(), |cx, _state| {
                        vec![cx.text("trigger")]
                    });

                let mut children = vec![trigger];
                if hovered {
                    children.push(cx.text("hovered"));
                }
                children
            },
        )]
    }

    // Frame 0: not hovered yet, so only the trigger is present.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover_region_node = ui.children(root)[0];
    assert_eq!(ui.children(hover_region_node).len(), 1);
    let trigger_node = ui.children(hover_region_node)[0];
    let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

    let pos = fret_core::Point::new(
        Px(trigger_bounds.origin.x.0 + 2.0),
        Px(trigger_bounds.origin.y.0 + 2.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Frame 1: hover_region should now observe hovered=true even though the hit node is a Pressable.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover_region_node = ui.children(root)[0];
    assert_eq!(ui.children(hover_region_node).len(), 2);
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
                gap: Px(5.0),
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
fn pressable_keyboard_activation_dispatches_click_command() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let cmd = CommandId::new("test.pressable.click");

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let cmd = cmd.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        host.dispatch_command(Some(acx.window), cmd.clone());
                    }));
                    vec![cx.text("ok")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));
    assert_eq!(ui.focus(), Some(pressable_node));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );
    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Command { command, .. } if *command == cmd)),
        "expected click command effect"
    );
}

struct TwoViewportRects {
    a: Rect,
    b: Rect,
}

impl TwoViewportRects {
    fn new(a: Rect, b: Rect) -> Self {
        Self { a, b }
    }
}

impl<H: UiHost> Widget<H> for TwoViewportRects {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        if let Some(&a) = cx.children.first() {
            let _ = cx.layout_viewport_root(a, self.a);
        }
        if let Some(&b) = cx.children.get(1) {
            let _ = cx.layout_viewport_root(b, self.b);
        }
        cx.available
    }
}

#[test]
fn viewport_rects_do_not_couple_fill_semantics_across_subtrees() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_viewport(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let mut props = crate::element::FlexProps::default();
        props.layout.size.width = crate::element::Length::Fill;
        props.layout.size.height = crate::element::Length::Fill;

        vec![cx.flex(props, |cx| {
            let mut child = crate::element::ContainerProps::default();
            child.layout.size.width = crate::element::Length::Fill;
            child.layout.size.height = crate::element::Length::Fill;
            vec![cx.container(child, |_| Vec::new())]
        })]
    }

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-a",
        build_viewport,
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-b",
        build_viewport,
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_a = ui.children(root_a)[0];
    let container_a = ui.children(flex_a)[0];
    let b_a = ui
        .debug_node_bounds(container_a)
        .expect("viewport a container bounds");
    assert!((b_a.origin.x.0 - viewport_a.origin.x.0).abs() < 0.01);
    assert!((b_a.size.width.0 - viewport_a.size.width.0).abs() < 0.01);

    let flex_b = ui.children(root_b)[0];
    let container_b = ui.children(flex_b)[0];
    let b_b = ui
        .debug_node_bounds(container_b)
        .expect("viewport b container bounds");
    assert!((b_b.origin.x.0 - viewport_b.origin.x.0).abs() < 0.01);
    assert!((b_b.size.width.0 - viewport_b.size.width.0).abs() < 0.01);
}

#[test]
fn attach_semantics_is_layout_transparent_for_flex_items() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_row(cx: &mut ElementContext<'_, TestHost>, decorate: bool) -> Vec<AnyElement> {
        let mut row = crate::element::FlexProps::default();
        row.layout.size.width = crate::element::Length::Fill;
        row.layout.size.height = crate::element::Length::Fill;
        row.direction = fret_core::Axis::Horizontal;

        vec![cx.flex(row, |cx| {
            let left = cx.text("x");

            let mut right_props = crate::element::TextProps::new("a much longer title");
            right_props.wrap = fret_core::TextWrap::None;
            right_props.overflow = fret_core::TextOverflow::Ellipsis;
            right_props.layout.flex.grow = 1.0;
            right_props.layout.flex.shrink = 1.0;
            right_props.layout.flex.basis = crate::element::Length::Px(Px(0.0));
            right_props.layout.size.min_width = Some(Px(0.0));

            let mut right = cx.text_props(right_props);
            if decorate {
                right = right.attach_semantics(
                    crate::element::SemanticsDecoration::default().test_id("row-title"),
                );
            }

            vec![left, right]
        })]
    }

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "attach-semantics-layout-a",
        |cx| build_row(cx, false),
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "attach-semantics-layout-b",
        |cx| build_row(cx, true),
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(220.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(260.0), Px(0.0)),
        Size::new(Px(220.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_a = ui.children(root_a)[0];
    let right_a = ui.children(flex_a)[1];
    let bounds_a = ui
        .debug_node_bounds(right_a)
        .expect("decorator-free right text bounds");
    assert!(
        bounds_a.size.width.0 > 50.0,
        "expected flex child to expand, got {:?}",
        bounds_a
    );

    let flex_b = ui.children(root_b)[0];
    let right_b = ui.children(flex_b)[1];
    let bounds_b = ui
        .debug_node_bounds(right_b)
        .expect("decorated right text bounds");
    assert!(
        (bounds_a.size.width.0 - bounds_b.size.width.0).abs() < 0.01,
        "expected attach_semantics to be layout-transparent"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_registration_is_recorded_in_layout_all() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-a",
        |cx| vec![cx.text("a")],
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-b",
        |cx| vec![cx.text("b")],
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(ui.viewport_roots().len(), 2);
    assert!(ui.viewport_roots().contains(&(root_a, viewport_a)));
    assert!(ui.viewport_roots().contains(&(root_b, viewport_b)));
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn layout_engine_solve_stats_are_per_call_and_bounded_for_two_viewport_roots() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.5, 0.5]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-engine-solve-stats-bounded",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.flex(crate::element::FlexProps::default(), |cx| {
                        vec![cx.text("left")]
                    }),
                    cx.flex(crate::element::FlexProps::default(), |cx| {
                        vec![cx.text("right")]
                    }),
                ]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let s1 = ui.debug_stats().layout_engine_solves;
    assert!(
        (1..=64).contains(&s1),
        "expected a small, non-zero solve count; got {s1}"
    );

    // A second call with identical inputs should not report the cumulative engine totals from the
    // prior call.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let s2 = ui.debug_stats().layout_engine_solves;
    assert_eq!(s2, 0, "expected per-call solve stats (not cumulative)");

    // Change the window bounds; this must force some engine work again, and should still be
    // bounded and per-call.
    let bounds2 = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(60.0)),
    );
    ui.layout_all(&mut app, &mut text, bounds2, 1.0);
    let s3 = ui.debug_stats().layout_engine_solves;
    assert!(
        (1..=64).contains(&s3),
        "expected a small, non-zero solve count after bounds change; got {s3}"
    );

    ui.layout_all(&mut app, &mut text, bounds2, 1.0);
    let s4 = ui.debug_stats().layout_engine_solves;
    assert_eq!(s4, 0, "expected per-call solve stats (not cumulative)");
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn resizable_panel_group_does_not_register_viewport_roots_during_probe_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.5, 0.5]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "panel-group-probe-does-not-register-viewport-roots",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| vec![cx.text("left"), cx.text("right")])]
        },
    );
    ui.set_root(root);
    ui.layout_all_with_pass_kind(
        &mut app,
        &mut text,
        bounds,
        1.0,
        crate::layout_pass::LayoutPassKind::Probe,
    );

    assert!(
        ui.viewport_roots().is_empty(),
        "expected probe layout to avoid registering viewport roots"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn probe_layout_does_not_prune_layout_engine_nodes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "probe-layout-does-not-prune-engine-nodes",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("hello")])],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let text_node = ui.children(container_node)[0];
    assert!(
        ui.layout_engine_has_node(text_node),
        "expected a final layout to register nodes in the layout engine"
    );

    ui.layout_all_with_pass_kind(
        &mut app,
        &mut text,
        bounds,
        1.0,
        crate::layout_pass::LayoutPassKind::Probe,
    );
    assert!(
        ui.layout_engine_has_node(text_node),
        "expected probe layouts to avoid pruning layout engine nodes"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_flush_only_lays_out_invalidated_roots() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountsLayout {
        count: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for CountsLayout {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            self.count.fetch_add(1, Ordering::Relaxed);
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

    let left_count = Arc::new(AtomicUsize::new(0));
    let right_count = Arc::new(AtomicUsize::new(0));

    let left = ui.create_node(CountsLayout {
        count: left_count.clone(),
    });
    ui.set_children(left, Vec::new());
    let right = ui.create_node(CountsLayout {
        count: right_count.clone(),
    });
    ui.set_children(right, Vec::new());

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![left, right]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(left_count.load(Ordering::Relaxed), 1);
    assert_eq!(right_count.load(Ordering::Relaxed), 1);

    // Invalidate only the left root subtree; the parent must re-register viewport roots, but the
    // flush loop should only lay out the invalidated root.
    ui.invalidate(left, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(left_count.load(Ordering::Relaxed), 2);
    assert_eq!(
        right_count.load(Ordering::Relaxed),
        1,
        "expected right viewport root to be skipped when clean"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_request_build_keeps_engine_nodes_alive_when_skipped() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_viewport(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.container(Default::default(), |cx| vec![cx.text("hello")])]
    }

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-keeps-engine-nodes-a",
        build_viewport,
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-keeps-engine-nodes-b",
        build_viewport,
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let b_container = ui.children(root_b)[0];
    let b_text = ui.children(b_container)[0];
    assert!(
        ui.layout_engine_has_node(b_text),
        "expected viewport subtree nodes to be registered in the engine after layout"
    );

    // Only invalidate the left viewport; the right root should be skipped by the flush loop, but
    // still kept alive via the request/build phase.
    ui.invalidate(root_a, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        ui.layout_engine_has_node(b_text),
        "expected skipped viewport roots to remain registered in the engine (stable identity)"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn solve_barrier_flow_root_reuses_solved_root_even_after_other_solves() {
    struct PrecomputesSameRootTwice {
        a: NodeId,
        b: NodeId,
        rect: Rect,
    }

    impl<H: UiHost> Widget<H> for PrecomputesSameRootTwice {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.solve_barrier_child_root(self.a, self.rect);
            cx.solve_barrier_child_root(self.b, self.rect);
            cx.solve_barrier_child_root(self.a, self.rect);

            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-a",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("a"), cx.text("aa")])],
    );
    let b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-b",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("b")])],
    );

    let rect = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(150.0), Px(40.0)),
    );
    let parent = ui.create_node(PrecomputesSameRootTwice { a, b, rect });
    ui.set_children(parent, vec![a, b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        2,
        "expected the third precompute to reuse cached solve results"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn solve_barrier_flow_root_if_needed_skips_translation_only_bounds_changes() {
    struct PrecomputeThenTranslate {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenTranslate {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let rect = if self.calls == 0 {
                cx.solve_barrier_child_root(self.child, self.rect_a);
                self.rect_a
            } else {
                cx.solve_barrier_child_root_if_needed(self.child, self.rect_b);
                self.rect_b
            };
            self.calls = self.calls.saturating_add(1);

            let _ = cx.layout_in(self.child, rect);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-translate-child",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("a"), cx.text("b")])],
    );

    let rect_a = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(150.0), Px(40.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(10.0), Px(15.0)),
        Size::new(Px(150.0), Px(40.0)),
    );

    let parent = ui.create_node(PrecomputeThenTranslate {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "expected the first precompute to solve at least once"
    );

    // Force the parent to re-run layout within the same frame, while keeping the child subtree
    // clean. A translation-only bounds change for the child should not trigger an engine solve.
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "expected translation-only layout to avoid triggering engine solves"
    );

    let child_bounds = ui.debug_node_bounds(child).expect("child bounds");
    assert!((child_bounds.origin.y.0 - rect_b.origin.y.0).abs() < 0.01);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn resizable_panel_group_viewport_roots_match_panel_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.5, 0.5]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "panel-group-viewport-roots-match-panel-bounds",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| vec![cx.text("left"), cx.text("right")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let group = ui.children(root)[0];
    let panel_a = ui.children(group)[0];
    let panel_b = ui.children(group)[1];

    let bounds_a = ui
        .debug_node_bounds(panel_a)
        .expect("expected panel a bounds");
    let bounds_b = ui
        .debug_node_bounds(panel_b)
        .expect("expected panel b bounds");

    assert_eq!(ui.viewport_roots().len(), 2);
    assert!(ui.viewport_roots().contains(&(panel_a, bounds_a)));
    assert!(ui.viewport_roots().contains(&(panel_b, bounds_b)));
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_registration_is_flushed_when_registered_from_another_viewport_root() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct RegistersViewport {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for RegistersViewport {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    struct MarksLayout {
        did_layout: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for MarksLayout {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            self.did_layout.store(true, Ordering::Relaxed);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let base = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-registration-is-flushed-nested",
        |cx| vec![cx.text("base")],
    );

    let inner_did_layout = Arc::new(AtomicBool::new(false));
    let inner = ui.create_node(MarksLayout {
        did_layout: inner_did_layout.clone(),
    });
    ui.set_children(inner, Vec::new());

    let outer_viewport = Rect::new(
        Point::new(Px(10.0), Px(10.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let inner_viewport = Rect::new(
        Point::new(Px(20.0), Px(30.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let outer = ui.create_node(RegistersViewport {
        viewport: inner_viewport,
        child: inner,
    });
    ui.set_children(outer, Vec::new());

    let root = ui.create_node(RegistersViewport {
        viewport: outer_viewport,
        child: outer,
    });
    ui.set_children(root, vec![base]);
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        inner_did_layout.load(Ordering::Relaxed),
        "expected nested viewport root to be laid out in the same frame"
    );
    assert!(ui.viewport_roots().contains(&(outer, outer_viewport)));
    assert!(ui.viewport_roots().contains(&(inner, inner_viewport)));
}

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_layout_is_applied_before_overlay_root_layout() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
        saw_default_bounds: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
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

    struct OverlayReadsViewportBounds {
        viewport: Rect,
        child: NodeId,
        saw_expected_bounds: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for OverlayReadsViewportBounds {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let bounds = cx
                .tree
                .debug_node_bounds(self.child)
                .unwrap_or_else(|| Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default()));
            let ok = (bounds.origin.x.0 - self.viewport.origin.x.0).abs() < 0.01
                && (bounds.origin.y.0 - self.viewport.origin.y.0).abs() < 0.01
                && (bounds.size.width.0 - self.viewport.size.width.0).abs() < 0.01
                && (bounds.size.height.0 - self.viewport.size.height.0).abs() < 0.01;
            self.saw_expected_bounds.store(ok, Ordering::Relaxed);
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
        "viewport-overlay-order-child",
        |cx| vec![cx.text("child")],
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let saw_default_bounds = Arc::new(AtomicBool::new(false));
    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child,
        saw_default_bounds: saw_default_bounds.clone(),
    });
    ui.set_root(base);

    let saw_expected_bounds = Arc::new(AtomicBool::new(false));
    let overlay = ui.create_node(OverlayReadsViewportBounds {
        viewport,
        child,
        saw_expected_bounds: saw_expected_bounds.clone(),
    });
    ui.push_overlay_root(overlay, false);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        saw_default_bounds.load(Ordering::Relaxed),
        "expected viewport root to be laid out after base root, not during Base.layout()"
    );
    assert!(
        saw_expected_bounds.load(Ordering::Relaxed),
        "expected viewport root bounds to be available during overlay root layout"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_roots_do_not_couple_fill_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-fill-a",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-fill-b",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_a = ui.children(root_a)[0];
    let spacer_a = ui.children(flex_a)[0];
    let a = ui
        .debug_node_bounds(spacer_a)
        .expect("viewport a spacer bounds");
    assert!((a.size.width.0 - viewport_a.size.width.0).abs() < 0.01);
    assert!((a.size.height.0 - viewport_a.size.height.0).abs() < 0.01);

    let flex_b = ui.children(root_b)[0];
    let spacer_b = ui.children(flex_b)[0];
    let b = ui
        .debug_node_bounds(spacer_b)
        .expect("viewport b spacer bounds");
    assert!((b.size.width.0 - viewport_b.size.width.0).abs() < 0.01);
    assert!((b.size.height.0 - viewport_b.size.height.0).abs() < 0.01);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn layout_engine_v2_scales_px_styles_with_scale_factor() {
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

    fn run(scale_factor: f32) -> (Rect, Rect) {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(140.0)),
        );
        let viewport = Rect::new(
            fret_core::Point::new(Px(5.0), Px(3.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let mut text = FakeTextService::default();

        let child_root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "scale-factor-px-style",
            |cx| {
                let flex = crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    gap: Px(8.0),
                    padding: fret_core::Edges::all(Px(10.0)),
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let child = |cx: &mut ElementContext<'_, TestHost>| {
                    let props = crate::element::ContainerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(10.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    cx.container(props, |_cx| vec![])
                };

                vec![cx.flex(flex, |cx| vec![child(cx), child(cx)])]
            },
        );

        let base = ui.create_node(RegistersViewportRoot { viewport });
        ui.set_children(base, vec![child_root]);
        ui.set_root(base);

        ui.layout_all(&mut app, &mut text, bounds, scale_factor);

        let flex_node = ui.children(child_root)[0];
        let first = ui.children(flex_node)[0];
        let second = ui.children(flex_node)[1];

        (
            ui.debug_node_bounds(first).expect("first bounds"),
            ui.debug_node_bounds(second).expect("second bounds"),
        )
    }

    let (first_1x, second_1x) = run(1.0);
    let (first_2x, second_2x) = run(2.0);

    assert_eq!(first_1x, first_2x, "expected scale-factor invariant bounds");
    assert_eq!(
        second_1x, second_2x,
        "expected scale-factor invariant bounds"
    );

    let expected_first = Rect::new(
        fret_core::Point::new(Px(15.0), Px(13.0)),
        Size::new(Px(180.0), Px(10.0)),
    );
    let expected_second = Rect::new(
        fret_core::Point::new(Px(15.0), Px(31.0)),
        Size::new(Px(180.0), Px(10.0)),
    );

    assert_eq!(first_1x, expected_first);
    assert_eq!(second_1x, expected_second);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn stack_does_not_stretch_spacer_children_in_engine_tree() {
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
        "stack-engine-no-stretch",
        |cx| {
            let mut props = crate::element::StackProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.stack_props(props, |cx| {
                vec![
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                ]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack = ui.children(child_root)[0];
    let a = ui.children(stack)[0];
    let b = ui.children(stack)[1];

    let a_bounds = ui.debug_node_bounds(a).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(b).expect("b bounds");

    assert_eq!(a_bounds.origin, viewport.origin);
    assert_eq!(b_bounds.origin, viewport.origin);

    assert!(a_bounds.size.width.0.abs() < 0.01);
    assert!(a_bounds.size.height.0.abs() < 0.01);
    assert!(b_bounds.size.width.0.abs() < 0.01);
    assert!(b_bounds.size.height.0.abs() < 0.01);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn container_does_not_stretch_spacer_child_in_engine_tree() {
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
        "container-engine-no-stretch",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.container(props, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container = ui.children(child_root)[0];
    let spacer = ui.children(container)[0];

    let container_bounds = ui.debug_node_bounds(container).expect("container bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer).expect("spacer bounds");

    assert_eq!(container_bounds, viewport);
    assert_eq!(spacer_bounds.origin, viewport.origin);
    assert!(spacer_bounds.size.width.0.abs() < 0.01);
    assert!(spacer_bounds.size.height.0.abs() < 0.01);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn pressable_does_not_stretch_spacer_child_in_engine_tree() {
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
        "pressable-engine-no-stretch",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pressable(props, |cx, _state| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable = ui.children(child_root)[0];
    let spacer = ui.children(pressable)[0];

    let pressable_bounds = ui.debug_node_bounds(pressable).expect("pressable bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer).expect("spacer bounds");

    assert_eq!(pressable_bounds, viewport);
    assert_eq!(spacer_bounds.origin, viewport.origin);
    assert!(spacer_bounds.size.width.0.abs() < 0.01);
    assert!(spacer_bounds.size.height.0.abs() < 0.01);
}

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_nested_flow_is_solved_once() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-nested-flow-solve-count",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let engine = ui.take_layout_engine();
    assert_eq!(
        engine.solve_count(),
        1,
        "expected viewport root flow subtree to be solved exactly once"
    );
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_hover_region_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner_a = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        let inner_b = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.hover_region(
                crate::element::HoverRegionProps::default(),
                |cx, _hovered| {
                    vec![
                        cx.flex(inner_a, |cx| vec![cx.text("hello")]),
                        cx.flex(inner_b, |cx| vec![cx.text("world")]),
                    ]
                },
            )]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-hover-region-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let hover = ui.children(outer)[0];
    let inner_a = ui.children(hover)[0];
    let inner_b = ui.children(hover)[1];
    let text_a = ui.children(inner_a)[0];
    let text_b = ui.children(inner_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(hover).is_some());
    assert!(engine.layout_id_for_node(inner_a).is_some());
    assert!(engine.layout_id_for_node(inner_b).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_pointer_region_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
                    vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pointer-region-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let inner = ui.children(region)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_pointer_region_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let a = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            ..Default::default()
        };

        let b = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
                    vec![
                        cx.flex(a, |cx| vec![cx.text("a")]),
                        cx.flex(b, |cx| vec![cx.text("b")]),
                    ]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pointer-region-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let flex_a = ui.children(region)[0];
    let flex_b = ui.children(region)[1];
    let text_a = ui.children(flex_a)[0];
    let text_b = ui.children(flex_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(flex_a).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(flex_b).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_pointer_region_absolute_child_fills_region() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let mut region = crate::element::PointerRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Fill;

        let mut barrier = crate::element::ContainerProps::default();
        barrier.layout.position = crate::element::PositionStyle::Absolute;
        barrier.layout.inset.top = Some(Px(0.0));
        barrier.layout.inset.right = Some(Px(0.0));
        barrier.layout.inset.bottom = Some(Px(0.0));
        barrier.layout.inset.left = Some(Px(0.0));
        barrier.layout.size.width = Length::Fill;
        barrier.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.pointer_region(region, |cx| vec![cx.container(barrier, |_cx| vec![])])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pointer-region-absolute-only",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let barrier = ui.children(region)[0];

    let region_bounds = ui.debug_node_bounds(region).expect("region bounds");
    let barrier_bounds = ui.debug_node_bounds(barrier).expect("barrier bounds");

    assert_eq!(region_bounds, viewport);
    assert_eq!(barrier_bounds, viewport);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn positioned_container_precomputes_flow_islands_for_multiple_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![
            cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
                vec![
                    cx.hover_region(
                        crate::element::HoverRegionProps::default(),
                        |cx, _hovered| vec![cx.text("left")],
                    ),
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Horizontal,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("right")],
                    ),
                ]
            }),
        ]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "positioned-container-multi-child-flow-islands",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let region = ui.children(root)[0];
    let hover = ui.children(region)[0];
    let flex = ui.children(region)[1];
    let hover_text = ui.children(hover)[0];
    let flex_text = ui.children(flex)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(hover).is_some());
    assert!(engine.layout_id_for_node(hover_text).is_some());
    assert!(engine.layout_id_for_node(flex).is_some());
    assert!(engine.layout_id_for_node(flex_text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn hover_region_precomputes_flow_islands_for_multiple_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.hover_region(
            crate::element::HoverRegionProps::default(),
            |cx, _hovered| {
                vec![
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("a")],
                    ),
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("b")],
                    ),
                ]
            },
        )]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region-multi-child-flow-islands",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover = ui.children(root)[0];
    let flex_a = ui.children(hover)[0];
    let flex_b = ui.children(hover)[1];
    let text_a = ui.children(flex_a)[0];
    let text_b = ui.children(flex_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(flex_a).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(flex_b).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn pressable_wraps_multiple_children_in_engine_tree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.flex(
            crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                ..Default::default()
            },
            |cx| {
                vec![
                    cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                        vec![cx.text("a"), cx.text("b")]
                    }),
                ]
            },
        )]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-engine-children",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex = ui.children(root)[0];
    let pressable = ui.children(flex)[0];
    let a = ui.children(pressable)[0];
    let b = ui.children(pressable)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(pressable).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_pressable_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pressable-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let pressable = ui.children(outer)[0];
    let a = ui.children(pressable)[0];
    let b = ui.children(pressable)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(pressable).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_semantics_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.semantics(crate::element::SemanticsProps::default(), |cx| {
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-semantics-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let semantics = ui.children(outer)[0];
    let a = ui.children(semantics)[0];
    let b = ui.children(semantics)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(semantics).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_focus_scope_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.focus_scope(crate::element::FocusScopeProps::default(), |cx| {
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-focus-scope-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let focus = ui.children(outer)[0];
    let a = ui.children(focus)[0];
    let b = ui.children(focus)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(focus).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_opacity_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.opacity(0.5, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-opacity-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let opacity = ui.children(outer)[0];
    let a = ui.children(opacity)[0];
    let b = ui.children(opacity)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(opacity).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_visual_transform_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let transform =
            fret_core::Transform2D::translation(fret_core::Point::new(Px(1.0), Px(2.0)));
        vec![cx.flex(outer, |cx| {
            vec![cx.visual_transform(transform, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-visual-transform-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let transform = ui.children(outer)[0];
    let a = ui.children(transform)[0];
    let b = ui.children(transform)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(transform).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_interactivity_gate_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.interactivity_gate(true, true, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-interactivity-gate-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let gate = ui.children(outer)[0];
    let a = ui.children(gate)[0];
    let b = ui.children(gate)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(gate).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_container_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let mut container = crate::element::ContainerProps::default();
        container.layout.size.width = Length::Fill;
        container.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.container(container, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-container-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let container = ui.children(outer)[0];
    let a = ui.children(container)[0];
    let b = ui.children(container)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(container).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_stack_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let mut stack = crate::element::StackProps::default();
        stack.layout.size.width = Length::Fill;
        stack.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.stack_props(stack, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-stack-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let stack = ui.children(outer)[0];
    let a = ui.children(stack)[0];
    let b = ui.children(stack)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(stack).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_render_transform_passthrough_fill_does_not_collapse() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let transform = Transform2D::translation(Point::new(Px(5.0), Px(0.0)));

        let mut region = crate::element::PointerRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.render_transform(transform, |cx| {
                vec![cx.pointer_region(region, |_cx| vec![])]
            })]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-render-transform-passthrough-fill",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let wrapper = ui.children(outer)[0];
    let region = ui.children(wrapper)[0];

    let region_bounds = ui.debug_node_bounds(region).expect("region bounds");
    assert_eq!(region_bounds.origin, viewport.origin);
    assert_eq!(region_bounds.size, viewport.size);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_wheel_region_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.wheel_region(crate::element::WheelRegionProps::default(), |cx| {
                    vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-wheel-region-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let inner = ui.children(region)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_auto_wrapper_promotes_fill_when_flow_child_requests_fill() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            align: CrossAlign::Start,
            ..Default::default()
        };

        let wrapper = crate::element::ContainerProps::default();

        let fill_child = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.container(wrapper, |cx| {
                vec![cx.flex(fill_child, |cx| vec![cx.text("fill-child")])]
            })]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-auto-wrapper-promotes-fill",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let wrapper = ui.children(outer)[0];

    let wrapper_bounds = ui.debug_node_bounds(wrapper).expect("wrapper bounds");
    assert_eq!(wrapper_bounds.origin, viewport.origin);
    assert_eq!(wrapper_bounds.size.width, viewport.size.width);
    assert!(wrapper_bounds.size.width.0 > 0.0);

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(wrapper).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_wheel_region_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let a = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            ..Default::default()
        };

        let b = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.wheel_region(crate::element::WheelRegionProps::default(), |cx| {
                    vec![
                        cx.flex(a, |cx| vec![cx.text("a")]),
                        cx.flex(b, |cx| vec![cx.text("b")]),
                    ]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-wheel-region-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let flex_a = ui.children(region)[0];
    let flex_b = ui.children(region)[1];
    let text_a = ui.children(flex_a)[0];
    let text_b = ui.children(flex_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(flex_a).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(flex_b).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_wheel_region_absolute_child_fills_region() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let mut region = crate::element::WheelRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Fill;

        let mut barrier = crate::element::ContainerProps::default();
        barrier.layout.position = crate::element::PositionStyle::Absolute;
        barrier.layout.inset.top = Some(Px(0.0));
        barrier.layout.inset.right = Some(Px(0.0));
        barrier.layout.inset.bottom = Some(Px(0.0));
        barrier.layout.inset.left = Some(Px(0.0));
        barrier.layout.size.width = Length::Fill;
        barrier.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.wheel_region(region, |cx| vec![cx.container(barrier, |_cx| vec![])])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-wheel-region-absolute-only",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let barrier = ui.children(region)[0];

    let region_bounds = ui.debug_node_bounds(region).expect("region bounds");
    let barrier_bounds = ui.debug_node_bounds(barrier).expect("barrier bounds");

    assert_eq!(region_bounds, viewport);
    assert_eq!(barrier_bounds, viewport);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_roving_flex_is_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let mut roving = crate::element::RovingFlexProps::default();
        roving.flex.layout.size.width = Length::Fill;
        roving.flex.layout.size.height = Length::Auto;
        roving.flex.direction = fret_core::Axis::Horizontal;
        roving.flex.gap = Px(4.0);

        vec![cx.flex(outer, |cx| {
            vec![cx.roving_flex(roving, |cx| vec![cx.text("hello")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-roving-flex-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let roving = ui.children(outer)[0];
    let text = ui.children(roving)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(roving).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn viewport_root_anchored_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        let anchored = crate::element::AnchoredProps {
            anchor: Rect::new(
                Point::new(Px(10.0), Px(10.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.anchored_props(anchored, |cx| {
                vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
            })]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-anchored-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let anchored = ui.children(outer)[0];
    let inner = ui.children(anchored)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(anchored).is_some());
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn overlay_root_dismissible_layer_precomputes_child_flow_islands() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );

    let base = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-underlay",
        |cx| vec![cx.text("underlay")],
    );
    ui.set_root(base);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-dismissible-precompute",
        |cx| {
            let anchored = crate::element::AnchoredProps {
                anchor: Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(10.0), Px(10.0)),
                ),
                ..Default::default()
            };

            let inner = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(4.0),
                ..Default::default()
            };

            vec![cx.anchored_props(anchored, |cx| {
                vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
            })]
        },
    );
    let _layer = ui.push_overlay_root_ex(overlay_root, false, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let anchored = ui.children(overlay_root)[0];
    let inner = ui.children(anchored)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(
        engine.layout_id_for_node(overlay_root).is_some(),
        "expected dismissible root to be represented in the engine tree"
    );
    assert!(
        engine.layout_id_for_node(anchored).is_some(),
        "expected overlay subtree to be precomputed into the engine tree"
    );
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn overlay_root_scroll_precomputes_child_flow_islands() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );

    let base = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-scroll-underlay",
        |cx| vec![cx.text("underlay")],
    );
    ui.set_root(base);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-scroll-precompute",
        |cx| {
            let anchored = crate::element::AnchoredProps {
                anchor: Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(10.0), Px(10.0)),
                ),
                ..Default::default()
            };

            let inner = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(4.0),
                ..Default::default()
            };

            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Fill;
            scroll.layout.size.height = Length::Fill;
            scroll.probe_unbounded = true;

            vec![cx.scroll(scroll, |cx| {
                vec![cx.anchored_props(anchored, |cx| {
                    vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
                })]
            })]
        },
    );
    let _layer = ui.push_overlay_root_ex(overlay_root, false, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll = ui.children(overlay_root)[0];
    let anchored = ui.children(scroll)[0];
    let inner = ui.children(anchored)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(
        engine.layout_id_for_node(anchored).is_some(),
        "expected scroll to precompute its child flow island into the engine tree"
    );
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[cfg(feature = "layout-engine-v2")]
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
            },
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

#[cfg(feature = "layout-engine-v2")]
#[test]
fn nested_flow_is_solved_once_per_island() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
        })]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "nested-flow-solve-count",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let engine = ui.take_layout_engine();
    assert_eq!(
        engine.solve_count(),
        1,
        "expected nested flex subtree to be solved once as a single flow island"
    );
    ui.put_layout_engine(engine);
}

#[test]
fn pressable_disabled_is_not_focusable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-disabled-focus",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: false,
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("disabled")],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(ui.first_focusable_descendant(root), None);
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
fn scroll_wheel_updates_offset_and_shifts_child_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");
    #[cfg(feature = "layout-engine-v2")]
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected scroll content subtree nodes to remain registered in the layout engine"
    );

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn scroll_translation_does_not_force_layout_engine_solves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel-solve-stats",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "expected scroll translation to avoid triggering layout engine solves"
    );
    #[cfg(feature = "layout-engine-v2")]
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected translation-only scroll to keep engine nodes alive (stable identity)"
    );

    // Even when the tree is fully clean (no invalidation, no translation), the request/build phase
    // must keep barrier-mounted subtrees registered so identity remains stable across frames.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    #[cfg(feature = "layout-engine-v2")]
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected steady-state frames to keep scroll content nodes registered in the engine"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn scroll_axis_both_probe_unbounded_keeps_content_at_least_viewport_width() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-axis-both-min-content-width",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Fill;
            p.axis = crate::element::ScrollAxis::Both;
            p.probe_unbounded = true;

            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a")],
                )]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let column_bounds = ui.debug_node_bounds(column_node).expect("column bounds");

    assert_eq!(
        column_bounds.size.width, bounds.size.width,
        "expected scroll content bounds to be at least the viewport width; got={:?} want={:?}",
        column_bounds.size.width, bounds.size.width
    );
}

#[test]
fn scroll_thumb_drag_updates_offset() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            // Ensure the scrollbar has enough room for Radix-style padding + 18px minimum thumb.
            // With very small tracks, Radix clamps the thumb to the available space and dragging
            // cannot change the scroll offset.
            stack_layout.size.height = crate::element::Length::Fill;

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        cx.text("a"),
                                        cx.text("b"),
                                        cx.text("c"),
                                        cx.text("d"),
                                        cx.text("e"),
                                        cx.text("f"),
                                    ]
                                },
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: None,
                        },
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    // Click/drag the scrollbar thumb down (thumb starts at the top at offset=0).
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let thumb = crate::declarative::paint_helpers::scrollbar_thumb_rect(
        scrollbar_bounds,
        scroll_handle.viewport_size().height,
        scroll_handle.content_size().height,
        scroll_handle.offset().y,
        crate::element::ScrollbarStyle::default().track_padding,
    )
    .expect("thumb rect");
    let down_pos = fret_core::Point::new(Px(thumb.origin.x.0 + 1.0), Px(thumb.origin.y.0 + 1.0));
    let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.captured(),
        Some(scrollbar_node),
        "expected thumb down to capture the pointer on the scrollbar node"
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().y
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after drag");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after thumb drag: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_handle_set_offset_triggers_visual_scroll_without_manual_invalidate() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            cx.text("a"),
                            cx.text("b"),
                            cx.text("c"),
                            cx.text("d"),
                            cx.text("e"),
                            cx.text("f"),
                        ]
                    },
                )]
            },
        )]
    }

    // Frame 0: establish viewport and content extent.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: stable mount (no intentional invalidations).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    // Outside the UI runtime, programmatically update the handle.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(20.0)));
    app.advance_frame();

    // Frame 2: the scroll change should invalidate bound nodes implicitly via handle bindings.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after offset");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after programmatic scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_thumb_drag_updates_offset_horizontal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag-x",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            // Ensure the scrollbar track has enough room for Radix-aligned padding + min thumb.
            stack_layout.size.height = crate::element::Length::Px(Px(30.0));

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            axis: crate::element::ScrollAxis::X,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            let mut content_layout = crate::element::LayoutStyle::default();
                            content_layout.size.width = crate::element::Length::Px(Px(300.0));
                            content_layout.size.height = crate::element::Length::Fill;

                            vec![cx.container(
                                crate::element::ContainerProps {
                                    layout: content_layout,
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("abc")],
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: None,
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: crate::element::SizeStyle {
                            height: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Horizontal,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let content_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(content_node)
        .expect("content visual bounds");

    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let down_pos = fret_core::Point::new(
        Px(scrollbar_bounds.origin.x.0 + 2.0),
        Px(scrollbar_bounds.origin.y.0 + 1.0),
    );
    let move_pos = fret_core::Point::new(Px(down_pos.x.0 + 12.0), down_pos.y);
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().x.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().x
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(content_node)
        .expect("content visual bounds after drag");

    assert!(
        after.origin.x.0 < before.origin.x.0,
        "expected content to move left after thumb drag: before={:?} after={:?}",
        before.origin.x,
        after.origin.x
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
                            max_width: Some(Px(100.0)),
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
                    gap: Px(0.0),
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
                    gap: Px(0.0),
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
                    gap: Px(0.0),
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
fn fixed_split_registers_viewport_roots_to_avoid_widget_fallback_solves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let left = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "fixed-split-left",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = Length::Fill;
                        l.size.height = Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| vec![cx.text("left")],
            )]
        },
    );

    let right = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "fixed-split-right",
        |cx| {
            vec![cx.grid(
                crate::element::GridProps {
                    cols: 1,
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = Length::Fill;
                        l.size.height = Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| vec![cx.text("right")],
            )]
        },
    );

    let split = crate::FixedSplit::create_node_with_children(
        &mut ui,
        crate::FixedSplit::horizontal(0.5),
        left,
        right,
    );
    ui.set_root(split);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(ui.debug_stats().layout_engine_widget_fallback_solves, 0);
}

#[test]
fn container_absolute_inset_positions_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-stack-absolute",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(0.0));
                    badge.layout.inset.right = Some(Px(0.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(container_bounds.size.width, Px(100.0));
    assert_eq!(container_bounds.size.height, Px(80.0));

    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(90.0));
    assert_eq!(badge_bounds.origin.y, Px(0.0));
}

#[test]
fn container_absolute_negative_inset_offsets_outside_parent() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-stack-absolute-negative-inset",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(-5.0));
                    badge.layout.inset.left = Some(Px(-6.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(-6.0));
    assert_eq!(badge_bounds.origin.y, Px(-5.0));
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
fn focus_ring_is_focus_visible_only() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(64.0), Px(32.0)),
    );
    let mut text = FakeTextService::default();

    let ring = crate::element::RingStyle {
        placement: crate::element::RingPlacement::Outset,
        width: Px(2.0),
        offset: Px(2.0),
        color: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        offset_color: None,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-focus-visible",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Fill,
                            height: crate::element::Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    focus_ring: Some(ring),
                    ..Default::default()
                },
                |_cx, _st| vec![],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let pressable_node = ui.children(root)[0];

    // Focus the pressable via pointer: should *not* show focus-visible ring.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: fret_core::Point::new(Px(4.0), Px(4.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected pressable to be focused after pointer down"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert_eq!(
        scene.ops().len(),
        0,
        "expected no ring ops for mouse-focused control"
    );

    // Enable focus-visible via keyboard navigation: ring should appear for focused control.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected focus to remain on pressable after keydown"
    );
    assert!(
        crate::focus_visible::is_focus_visible(&mut app, Some(window)),
        "expected focus-visible to be enabled after Tab keydown"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert!(
        !scene.ops().is_empty(),
        "expected ring ops for keyboard navigation focus-visible"
    );
}

#[test]
fn declarative_elements_can_observe_models_for_invalidation() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root_name = "mvp50-observe-model";

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        root_name,
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                cx.observe_model(&model, Invalidation::Layout);
                let v = cx.app.models().get_copied(&model).unwrap_or_default();
                vec![cx.text(format!("Value {v}"))]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let stats0 = ui.debug_stats();
    assert!(
        stats0.layout_nodes_visited > 0,
        "expected layout traversal: visited={} performed={}",
        stats0.layout_nodes_visited,
        stats0.layout_nodes_performed
    );
    let performed0 = stats0.layout_nodes_performed;
    assert!(performed0 > 0, "expected initial layout work");

    // A second layout pass with no changes and no re-render should perform no node layouts.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let performed1 = ui.debug_stats().layout_nodes_performed;
    assert_eq!(performed1, 0, "expected no layout work when clean");

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    ui.propagate_model_changes(&mut app, &changed);

    // The observed model change should invalidate the declarative host, enabling layout work.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let performed2 = ui.debug_stats().layout_nodes_performed;
    assert!(performed2 > 0, "expected model change to trigger relayout");
}

#[test]
fn model_observation_persists_after_frame_advance_without_render_root() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-observe-contract-frame-advance",
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                cx.observe_model(&model, Invalidation::Layout);
                let v = cx.app.models().get_copied(&model).unwrap_or_default();
                vec![cx.text(format!("Value {v}"))]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Advance the frame but intentionally skip the render pass.
    app.advance_frame();

    // The first model change still invalidates because UiTree retains the previous observation
    // index until the next layout/paint pass records observations again.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected invalidation from the last recorded observation index"
    );

    // Layout now runs on the advanced frame. Without a new render pass, the declarative layer
    // has no per-frame observation data to re-register, but UiTree still retains the last known
    // observation sets (needed for cache-hit frames where a subtree is reused without being rebuilt).
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // A second model change still invalidates based on the previous observation set, even though
    // `render_root(...)` was not called on this frame.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected invalidation based on the last recorded observation set"
    );
}

#[test]
fn container_applies_padding_and_paints_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-container",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                    background: Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let text_node = ui.children(container_node)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
    assert_eq!(text_bounds.origin.x, Px(4.0));
    assert_eq!(text_bounds.origin.y, Px(6.0));
    assert_eq!(text_bounds.size.width, Px(10.0));
    assert_eq!(text_bounds.size.height, Px(10.0));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 2);
    match scene.ops()[0] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected quad op"),
    }
}

#[test]
fn container_shrink_wraps_to_max_child_under_definite_parent_bounds() {
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
        "mvp50-container-shrink-wraps",
        |cx| {
            let outer = crate::element::ContainerProps {
                padding: fret_core::Edges::all(Px(2.0)),
                ..crate::element::ContainerProps::default()
            };

            vec![cx.container(outer, |cx| {
                let mut fixed = crate::element::ContainerProps::default();
                fixed.layout.size.width = crate::element::Length::Px(Px(30.0));
                fixed.layout.size.height = crate::element::Length::Px(Px(15.0));

                vec![cx.container(fixed, |_| Vec::new()), cx.text("x")]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(root)[0];
    let outer_bounds = ui.debug_node_bounds(outer).expect("outer bounds");
    assert_eq!(outer_bounds.size.width, Px(34.0));
    assert_eq!(outer_bounds.size.height, Px(19.0));
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn container_nested_chains_do_not_trigger_extra_engine_solves_when_clean() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

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
        "container-nested-clean-solves",
        |cx| {
            let mut outer = crate::element::ContainerProps::default();
            outer.padding = fret_core::Edges::all(Px(2.0));

            let mut inner = crate::element::ContainerProps::default();
            inner.padding = fret_core::Edges::all(Px(1.0));

            vec![cx.container(outer, |cx| {
                vec![cx.container(inner, |cx| vec![cx.text("x")])]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(ui.debug_stats().layout_engine_solves, 1);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(ui.debug_stats().layout_engine_solves, 0);
}

#[test]
fn container_paints_shadow_before_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp60-shadow",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    background: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    shadow: Some(crate::element::ShadowStyle {
                        primary: crate::element::ShadowLayerStyle {
                            color: Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.5,
                            },
                            offset_x: Px(2.0),
                            offset_y: Px(3.0),
                            blur: Px(0.0),
                            spread: Px(1.0),
                        },
                        secondary: None,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    }),
                    corner_radii: fret_core::Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);

    let shadow_bounds = match scene.ops()[0] {
        SceneOp::Quad { rect, .. } => rect,
        _ => panic!("expected shadow quad first"),
    };
    match scene.ops()[1] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected background quad second"),
    }

    assert!(shadow_bounds.origin.x.0 > container_bounds.origin.x.0);
    assert!(shadow_bounds.origin.y.0 > container_bounds.origin.y.0);
    assert!(shadow_bounds.size.width.0 > container_bounds.size.width.0);
    assert!(shadow_bounds.size.height.0 > container_bounds.size.height.0);
}

#[test]
fn pressable_dispatches_click_command_when_released_over_self() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let command = CommandId::from("test.click");

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-pressable",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    ..Default::default()
                },
                |cx, _state| {
                    let command = command.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        host.dispatch_command(Some(acx.window), command.clone());
                    }));
                    vec![cx.container(
                        crate::element::ContainerProps {
                            padding: fret_core::Edges::all(Px(4.0)),
                            ..Default::default()
                        },
                        |cx| vec![cx.text("hi")],
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 10.0),
        Px(pressable_bounds.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Command { command: c, .. } if c.as_str() == "test.click")),
        "expected Effect::Command(test.click), got {effects:?}"
    );

    // Sanity: move outside should clear hover state for future interactions.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(200.0), Px(200.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
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
                    gap: Px(5.0),
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
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

#[test]
fn scroll_rounds_scrollable_extent_up_to_next_pixel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    let handle_for_root = handle.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-rounding",
        move |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;

            let mut child_layout = crate::element::LayoutStyle::default();
            child_layout.size.width = Length::Fill;
            child_layout.size.height = Length::Px(Px(100.2));

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(handle_for_root.clone()),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: child_layout,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("content")],
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max = handle.max_offset();
    assert!((max.y.0 - 51.0).abs() < 0.01, "max_offset.y={:?}", max.y);

    handle.scroll_to_offset(Point::new(Px(0.0), Px(60.0)));
    assert!(
        (handle.offset().y.0 - 51.0).abs() < 0.01,
        "offset.y={:?}",
        handle.offset().y
    );
}
