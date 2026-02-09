use super::*;

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
