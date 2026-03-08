use super::*;

#[test]
fn text_word_wrap_uses_near_zero_wrap_width_under_min_content_constraints() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            let metrics = self.measure(input, constraints);
            (fret_core::TextBlobId::default(), metrics)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            let base_w = (input.text().chars().count() as f32) * 10.0;
            let w = match (constraints.wrap, constraints.max_width) {
                (fret_core::TextWrap::Word, Some(max_w)) if max_w.0.abs() < 0.01 => input
                    .text()
                    .split_whitespace()
                    .map(|seg| seg.chars().count() as f32 * 10.0)
                    .fold(0.0f32, f32::max),
                (_, Some(max_w)) => base_w.min(max_w.0.max(0.0)),
                (_, None) => base_w,
            };
            TextMetrics {
                size: Size::new(Px(w), Px(10.0)),
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-min-content-word-wrap",
        |cx| vec![cx.text("Clear Button")],
    );
    ui.set_root(root);
    let text = ui.children(root)[0];

    let min_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MinContent, AvailableSpace::MinContent),
    );
    let size = ui.measure_in(&mut app, &mut services, text, min_constraints, 1.0);

    assert!(
        size.width.0 > 0.01,
        "expected min-content text measurement to produce a non-zero width; size={size:?}, measured={:?}",
        services.measured
    );
    assert!(
        services.measured.iter().any(|c| {
            matches!(c.wrap, fret_core::TextWrap::Word)
                && c.max_width.is_some_and(|w| w.0.abs() < 0.01)
        }),
        "expected TextWrap::Word to use a near-zero wrap width under min-content constraints; measured={:?}",
        services.measured
    );
}

#[test]
fn text_word_wrap_treats_zero_available_width_as_unknown_for_intrinsic_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            let metrics = self.measure(input, constraints);
            (fret_core::TextBlobId::default(), metrics)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            let base_w = (input.text().chars().count() as f32) * 10.0;
            let w = constraints
                .max_width
                .map(|max| base_w.min(max.0))
                .unwrap_or(base_w);
            TextMetrics {
                size: Size::new(Px(w), Px(10.0)),
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-zero-available-word-wrap",
        |cx| vec![cx.text("Clear Button")],
    );
    ui.set_root(root);
    let text = ui.children(root)[0];

    let constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(0.0)),
            AvailableSpace::MinContent,
        ),
    );
    let size = ui.measure_in(&mut app, &mut services, text, constraints, 1.0);

    assert!(
        size.width.0 > 0.01,
        "expected intrinsic text measurement to produce a non-zero width when available.width=0 is a placeholder; size={size:?}, measured={:?}",
        services.measured
    );
    assert!(
        !services.measured.iter().any(|c| {
            matches!(c.wrap, fret_core::TextWrap::Word)
                && c.max_width.is_some_and(|w| w.0.abs() < 0.01)
        }),
        "expected TextWrap::Word not to force max_width=0.0 when available.width=0 is a placeholder; measured={:?}",
        services.measured
    );
}

#[test]
fn text_word_wrap_fill_width_treats_zero_available_width_as_unknown_for_intrinsic_measurement() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            let metrics = self.measure(input, constraints);
            (fret_core::TextBlobId::default(), metrics)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            _input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            let collapsed = matches!(constraints.wrap, fret_core::TextWrap::Word)
                && constraints.max_width.is_some_and(|w| w.0.abs() < 0.01);
            TextMetrics {
                size: Size::new(Px(80.0), if collapsed { Px(0.0) } else { Px(10.0) }),
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-zero-available-fill-width-word-wrap",
        |cx| {
            let mut props = crate::element::TextProps::new("Clear Button");
            props.layout.size.width = Length::Fill;
            props.wrap = fret_core::TextWrap::Word;
            vec![cx.text_props(props)]
        },
    );
    ui.set_root(root);
    let text = ui.children(root)[0];

    let constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(0.0)),
            AvailableSpace::MinContent,
        ),
    );
    let size = ui.measure_in(&mut app, &mut services, text, constraints, 1.0);

    assert!(
        size.height.0 > 0.01,
        "expected fill-width intrinsic text measurement to preserve a non-zero height when available.width=0 is a placeholder; size={size:?}, measured={:?}",
        services.measured
    );
    assert!(
        !services.measured.iter().any(|c| {
            matches!(c.wrap, fret_core::TextWrap::Word)
                && c.max_width.is_some_and(|w| w.0.abs() < 0.01)
        }),
        "expected fill-width TextWrap::Word not to force max_width=0.0 when available.width=0 is a placeholder; measured={:?}",
        services.measured
    );
}

#[test]
fn horizontal_flex_row_tracks_wrapped_text_height() {
    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
        prepared: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            self.prepared.push(constraints);
            (
                fret_core::TextBlobId::default(),
                self.measure(input, constraints),
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            let char_w = 6.0;
            let line_h = 10.0;
            let text_w = input.text().chars().count() as f32 * char_w;
            let max_w = constraints.max_width.map(|w| w.0.max(char_w));
            let lines = match (constraints.wrap, max_w) {
                (fret_core::TextWrap::WordBreak, Some(max_w)) if max_w + 0.01 < text_w => {
                    (text_w / max_w).ceil().max(1.0)
                }
                _ => 1.0,
            };
            let width = max_w.unwrap_or(text_w).min(text_w);
            TextMetrics {
                size: Size::new(Px(width), Px(line_h * lines)),
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(240.0)),
    );
    let mut text = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "horizontal-flex-row-tracks-wrapped-text-height",
        |cx| {
            let mut row_layout = crate::element::LayoutStyle::default();
            row_layout.size.width = crate::element::Length::Fill;

            let mut wrapped_layout = crate::element::LayoutStyle::default();
            wrapped_layout.flex.grow = 1.0;
            wrapped_layout.flex.shrink = 1.0;
            wrapped_layout.size.min_width = Some(crate::element::Length::Px(Px(0.0)));

            vec![cx.flex(
                crate::element::FlexProps {
                    layout: row_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(4.0).into(),
                    justify: crate::element::MainAlign::Start,
                    align: crate::element::CrossAlign::Start,
                    wrap: false,
                    ..Default::default()
                },
                |cx| {
                    vec![
                        cx.text("•"),
                        cx.text_props(crate::element::TextProps {
                            layout: wrapped_layout,
                            text: Arc::from(
                                "Core parity is already in a good place: default size, circular clipping, fallback timing, and overlap geometry match the upstream references we audit against.",
                            ),
                            style: None,
                            color: None,
                            wrap: fret_core::TextWrap::WordBreak,
                            overflow: fret_core::TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: crate::element::TextInkOverflow::None,
                        }),
                    ]
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let row = ui.children(root)[0];
    let bullet = ui.children(row)[0];
    let wrapped = ui.children(row)[1];

    let row_bounds = ui.debug_node_bounds(row).expect("row bounds");
    let bullet_bounds = ui.debug_node_bounds(bullet).expect("bullet bounds");
    let wrapped_bounds = ui.debug_node_bounds(wrapped).expect("wrapped bounds");

    assert!(
        wrapped_bounds.size.height.0 > bullet_bounds.size.height.0 + 0.5,
        "expected wrapped text to span multiple lines: bullet={bullet_bounds:?} wrapped={wrapped_bounds:?} row={row_bounds:?}"
    );
    assert!(
        row_bounds.size.height.0 + 0.5 >= wrapped_bounds.size.height.0,
        "expected horizontal flex row height to include wrapped text height: row={row_bounds:?} wrapped={wrapped_bounds:?} bullet={bullet_bounds:?}"
    );
}

#[test]
fn max_width_container_constrains_wrapped_text_children() {
    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
        prepared: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            self.prepared.push(constraints);
            (
                fret_core::TextBlobId::default(),
                self.measure(input, constraints),
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            let char_w = 6.0;
            let line_h = 10.0;
            let text_w = input.text().chars().count() as f32 * char_w;
            let max_w = constraints.max_width.map(|w| w.0.max(char_w));
            let lines = match (constraints.wrap, max_w) {
                (fret_core::TextWrap::WordBreak, Some(max_w)) if max_w + 0.01 < text_w => {
                    (text_w / max_w).ceil().max(1.0)
                }
                _ => 1.0,
            };
            let width = max_w.unwrap_or(text_w).min(text_w);
            TextMetrics {
                size: Size::new(Px(width), Px(line_h * lines)),
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(240.0)),
    );
    let mut text = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "max-width-container-constrains-wrapped-text-children",
        |cx| {
            let mut container_layout = crate::element::LayoutStyle::default();
            container_layout.size.width = crate::element::Length::Fill;
            container_layout.size.max_width = Some(crate::element::Length::Px(Px(220.0)));

            let mut row_layout = crate::element::LayoutStyle::default();
            row_layout.size.width = crate::element::Length::Fill;

            let mut wrapped_layout = crate::element::LayoutStyle::default();
            wrapped_layout.flex.grow = 1.0;
            wrapped_layout.flex.shrink = 1.0;
            wrapped_layout.size.min_width = Some(crate::element::Length::Px(Px(0.0)));

            vec![cx.container(
                crate::element::ContainerProps {
                    layout: container_layout,
                    ..Default::default()
                },
                |cx| {
                    vec![cx.flex(
                        crate::element::FlexProps {
                            layout: row_layout,
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(4.0).into(),
                            justify: crate::element::MainAlign::Start,
                            align: crate::element::CrossAlign::Start,
                            wrap: false,
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                cx.text("•"),
                                cx.text_props(crate::element::TextProps {
                                    layout: wrapped_layout,
                                    text: Arc::from(
                                        "Core parity is already in a good place: default size, circular clipping, fallback timing, and overlap geometry match the upstream references we audit against.",
                                    ),
                                    style: None,
                                    color: None,
                                    wrap: fret_core::TextWrap::WordBreak,
                                    overflow: fret_core::TextOverflow::Clip,
                                    align: fret_core::TextAlign::Start,
                                    ink_overflow: crate::element::TextInkOverflow::None,
                                }),
                            ]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container = ui.children(root)[0];
    let row = ui.children(container)[0];
    let bullet = ui.children(row)[0];
    let wrapped = ui.children(row)[1];

    let container_bounds = ui.debug_node_bounds(container).expect("container bounds");
    let row_bounds = ui.debug_node_bounds(row).expect("row bounds");
    let bullet_bounds = ui.debug_node_bounds(bullet).expect("bullet bounds");
    let wrapped_bounds = ui.debug_node_bounds(wrapped).expect("wrapped bounds");

    assert!(
        (container_bounds.size.width.0 - 220.0).abs() <= 0.5,
        "expected outer container to honor max-width: container={container_bounds:?}"
    );
    assert!(
        row_bounds.size.width.0 <= container_bounds.size.width.0 + 0.5,
        "expected row width to stay within max-width container: container={container_bounds:?} row={row_bounds:?}"
    );
    assert!(
        wrapped_bounds.size.height.0 > bullet_bounds.size.height.0 + 0.5,
        "expected wrapped text inside max-width container to span multiple lines: container={container_bounds:?} row={row_bounds:?} bullet={bullet_bounds:?} wrapped={wrapped_bounds:?} measured={:?}",
        text.measured
    );
}

#[test]
fn text_word_wrap_does_not_wrap_to_single_glyph_width_under_items_start_probe_layout() {
    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            let metrics = self.measure(input, constraints);
            (fret_core::TextBlobId::default(), metrics)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            let base_w = (input.text().chars().count() as f32) * 10.0;
            let w = constraints
                .max_width
                .map(|max| base_w.min(max.0))
                .unwrap_or(base_w);
            TextMetrics {
                size: Size::new(Px(w), Px(10.0)),
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
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
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-items-start-probe-word-wrap",
        |cx| {
            let mut col = crate::element::ColumnProps::default();
            col.layout.size.width = Length::Fill;
            col.align = crate::element::CrossAlign::Start;
            vec![cx.column(col, |cx| vec![cx.text("Description")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let col = ui.children(root)[0];
    let text_node = ui.children(col)[0];
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");

    assert!(
        (text_bounds.size.width.0 - 110.0).abs() < 0.01,
        "expected items_start column to keep word-wrapped text at its intrinsic width; bounds={text_bounds:?}, measured={:?}",
        services.measured
    );
    assert!(
        !services.measured.iter().any(|c| {
            matches!(c.wrap, fret_core::TextWrap::Word)
                && c.max_width.is_some_and(|w| w.0.abs() < 0.01)
        }),
        "expected Probe text measurement not to force max_width=0.0 under items_start; measured={:?}",
        services.measured
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
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
fn text_measurement_and_paint_agree_on_wrap_width_in_max_width_row() {
    #[derive(Default)]
    struct RecordingTextService {
        measured: Vec<TextConstraints>,
        prepared: Vec<TextConstraints>,
        prepared_metrics: Vec<TextMetrics>,
    }

    impl RecordingTextService {
        fn fake_metrics(input: &fret_core::TextInput, constraints: TextConstraints) -> TextMetrics {
            let char_w = 6.0;
            let line_h = 10.0;
            let text_w = input.text().chars().count() as f32 * char_w;
            let max_w = constraints.max_width.map(|w| w.0.max(char_w));
            let lines = match (constraints.wrap, max_w) {
                (fret_core::TextWrap::WordBreak, Some(max_w)) if max_w + 0.01 < text_w => {
                    (text_w / max_w).ceil().max(1.0)
                }
                _ => 1.0,
            };
            let width = max_w.unwrap_or(text_w).min(text_w);
            TextMetrics {
                size: Size::new(Px(width), Px(line_h * lines)),
                baseline: Px(8.0),
            }
        }
    }

    impl TextService for RecordingTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            let metrics = Self::fake_metrics(input, constraints);
            self.prepared.push(constraints);
            self.prepared_metrics.push(metrics);
            (fret_core::TextBlobId::default(), metrics)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}

        fn measure(
            &mut self,
            input: &fret_core::TextInput,
            constraints: TextConstraints,
        ) -> TextMetrics {
            self.measured.push(constraints);
            Self::fake_metrics(input, constraints)
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(160.0)),
    );
    let mut services = RecordingTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-measure-paint-wrap-width-max-width-row",
        |cx| {
            let mut container_layout = crate::element::LayoutStyle::default();
            container_layout.size.width = Length::Fill;
            container_layout.size.max_width = Some(Length::Px(Px(220.0)));

            let mut row_layout = crate::element::LayoutStyle::default();
            row_layout.size.width = Length::Fill;

            let mut wrapped_layout = crate::element::LayoutStyle::default();
            wrapped_layout.flex.grow = 1.0;
            wrapped_layout.flex.shrink = 1.0;
            wrapped_layout.size.min_width = Some(Length::Px(Px(0.0)));

            let mut sibling_props = crate::element::ContainerProps::default();
            sibling_props.layout.size.width = Length::Fill;
            sibling_props.layout.size.height = Length::Px(Px(10.0));

            vec![cx.container(
                crate::element::ContainerProps {
                    layout: container_layout,
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        vec![
                            cx.flex(
                                crate::element::FlexProps {
                                    layout: row_layout,
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(4.0).into(),
                                    justify: crate::element::MainAlign::Start,
                                    align: crate::element::CrossAlign::Start,
                                    wrap: false,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        cx.text("•"),
                                        cx.text_props(crate::element::TextProps {
                                            layout: wrapped_layout,
                                            text: Arc::from(
                                                "Core parity is already in a good place: default size, circular clipping, fallback timing, and overlap geometry match the upstream references we audit against.",
                                            ),
                                            style: None,
                                            color: None,
                                            wrap: fret_core::TextWrap::WordBreak,
                                            overflow: fret_core::TextOverflow::Clip,
                                            align: fret_core::TextAlign::Start,
                                            ink_overflow: crate::element::TextInkOverflow::None,
                                        }),
                                    ]
                                },
                            ),
                            cx.container(sibling_props, |_| Vec::new()),
                        ]
                    })]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let container = ui.children(root)[0];
    let col = ui.children(container)[0];
    let row = ui.children(col)[0];
    let sibling = ui.children(col)[1];
    let row_children = ui.children(row);
    let wrapped = row_children[1];
    let wrapped_bounds = ui.debug_node_bounds(wrapped).expect("wrapped bounds");
    let sibling_bounds = ui.debug_node_bounds(sibling).expect("sibling bounds");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let measured = services
        .measured
        .iter()
        .filter(|c| matches!(c.wrap, fret_core::TextWrap::WordBreak))
        .max_by(|a, b| {
            a.max_width
                .unwrap_or(Px(0.0))
                .0
                .total_cmp(&b.max_width.unwrap_or(Px(0.0)).0)
        })
        .copied()
        .expect("measured constraints");
    let prepared = services
        .prepared
        .iter()
        .filter(|c| matches!(c.wrap, fret_core::TextWrap::WordBreak))
        .max_by(|a, b| {
            a.max_width
                .unwrap_or(Px(0.0))
                .0
                .total_cmp(&b.max_width.unwrap_or(Px(0.0)).0)
        })
        .copied()
        .expect("prepared constraints");

    assert!(
        (measured.max_width.unwrap().0 - prepared.max_width.unwrap().0).abs() < 0.01,
        "expected measure/paint to use the same wrap width inside max-width row; measured={measured:?} prepared={prepared:?}"
    );

    let prepared_metrics = services
        .prepared_metrics
        .last()
        .copied()
        .expect("prepared metrics");
    assert!(
        sibling_bounds.origin.y.0 + 0.01
            >= wrapped_bounds.origin.y.0 + prepared_metrics.size.height.0,
        "expected following sibling to sit below painted wrapped text height inside max-width row; wrapped={wrapped_bounds:?} sibling={sibling_bounds:?} painted_h={:?}",
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

    impl fret_core::MaterialService for RecordingTextService {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
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
