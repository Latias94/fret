use super::super::super::super::super::*;

pub(in crate::ui) fn preview_text_bidi_rtl_conformance(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct BidiSample {
        label: &'static str,
        text: &'static str,
    }

    const SAMPLES: &[BidiSample] = &[
        BidiSample {
            label: "LTR baseline",
            text: "The quick brown fox (123) jumps.",
        },
        BidiSample {
            label: "Hebrew (RTL)",
            text: "עברית (123) אבגדה",
        },
        BidiSample {
            label: "Arabic (RTL)",
            text: "مرحبا بالعالم (123) أهلاً",
        },
        BidiSample {
            label: "Mixed LTR + Hebrew",
            text: "abc אבג DEF 123",
        },
        BidiSample {
            label: "Mixed punctuation + numbers",
            text: "abc (אבג) - 12:34 - xyz",
        },
        BidiSample {
            label: "Mixed LTR + Arabic",
            text: "hello مرحبا (123) world",
        },
        BidiSample {
            label: "Grapheme + RTL",
            text: "emoji 😀 אבג café",
        },
        BidiSample {
            label: "Controls (RLM)",
            text: "RLM:\u{200F}abc אבג 123",
        },
    ];

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        sample: usize,
        max_width_bits: u32,
        scale_bits: u32,
    }

    struct BidiState {
        selected_sample: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
        anchor: usize,
        caret: usize,
        affinity: CaretAffinity,
        pending_down: Option<(Point, bool)>,
        last_drag_pos: Option<Point>,
        dragging: bool,
    }

    impl Default for BidiState {
        fn default() -> Self {
            Self {
                selected_sample: 0,
                prepared_key: None,
                blob: None,
                metrics: None,
                anchor: 0,
                caret: 0,
                affinity: CaretAffinity::Downstream,
                pending_down: None,
                last_drag_pos: None,
                dragging: false,
            }
        }
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(BidiState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: sanity-check BiDi/RTL geometry queries (hit-test, caret rects, selection rects)."),
                cx.text("Use the selectable samples to validate editor-like selection behavior."),
                cx.text("Use the diagnostic panel to verify `hit_test_point` → caret/selection rendering under mixed-direction strings."),
            ]
        },
    );

    let sample_buttons = {
        let mut buttons: Vec<AnyElement> = Vec::new();
        for (i, s) in SAMPLES.iter().enumerate() {
            buttons.push(cx.keyed(format!("bidi-sample-btn-{i}"), |cx| {
                let state_for_click = state.clone();
                let is_selected = state.borrow().selected_sample == i;

                let variant = if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Outline
                };

                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let mut st = state_for_click.borrow_mut();
                        st.selected_sample = i;
                        st.anchor = 0;
                        st.caret = 0;
                        st.affinity = CaretAffinity::Downstream;
                        st.pending_down = None;
                        st.last_drag_pos = None;
                        st.dragging = false;
                        host.request_redraw(action_cx.window);
                    });

                shadcn::Button::new(s.label)
                    .variant(variant)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_activate)
                    .into_element(cx)
            }));
        }

        let mut props = fret_ui::element::FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(8.0);
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |_cx| buttons)
    };

    let selectable_samples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(cx.text("SelectableText samples:"));

            for (i, s) in SAMPLES.iter().enumerate() {
                out.push(cx.keyed(format!("bidi-sample-row-{i}"), |cx| {
                    let rich = AttributedText::new(
                        Arc::<str>::from(s.text),
                        Arc::<[TextSpan]>::from([TextSpan::new(s.text.len())]),
                    );

                    let mut props = fret_ui::element::SelectableTextProps::new(rich);
                    props.style = Some(TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    });
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    props.layout.size.width = fret_ui::element::Length::Fill;

                    let text = cx.selectable_text_props(props);

                    let row = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |cx| {
                            vec![
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: Arc::<str>::from(format!("{}:", s.label)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.container(
                                    decl_style::container_props(
                                        theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Md)
                                            .p(Space::N2)
                                            .bg(ColorRef::Color(
                                                theme.color_required("background"),
                                            )),
                                        LayoutRefinement::default().w_full(),
                                    ),
                                    move |_cx| vec![text],
                                ),
                            ]
                        },
                    );

                    row
                }));
            }

            out
        },
    );

    let diagnostic = {
        let state_for_handlers = state.clone();
        let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, action_cx, down| {
            let mut st = state_for_handlers.borrow_mut();
            st.pending_down = Some((down.position, down.modifiers.shift));
            st.last_drag_pos = Some(down.position);
            st.dragging = true;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        let state_for_handlers = state.clone();
        let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, action_cx, mv| {
            let mut st = state_for_handlers.borrow_mut();
            if st.dragging && mv.buttons.left {
                st.last_drag_pos = Some(mv.position);
                host.invalidate(fret_ui::Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let state_for_handlers = state.clone();
        let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, action_cx, _up| {
            let mut st = state_for_handlers.borrow_mut();
            st.dragging = false;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(220.0))),
            ),
            move |cx| {
                let mut pointer = fret_ui::element::PointerRegionProps::default();
                pointer.layout.size.width = fret_ui::element::Length::Fill;
                pointer.layout.size.height = fret_ui::element::Length::Fill;
                pointer.layout.overflow = fret_ui::element::Overflow::Clip;

                let paint_state = state.clone();

                let content = cx.pointer_region(pointer, move |cx| {
                    cx.pointer_region_on_pointer_down(on_down.clone());
                    cx.pointer_region_on_pointer_move(on_move.clone());
                    cx.pointer_region_on_pointer_up(on_up.clone());

                    let mut canvas = CanvasProps::default();
                    canvas.layout.size.width = fret_ui::element::Length::Fill;
                    canvas.layout.size.height = fret_ui::element::Length::Fill;
                    canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                    canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let canvas = cx.canvas(canvas, move |p| {
                        fn format_utf8_context(text: &str, index: usize) -> String {
                            let idx = index.min(text.len());
                            let mut prev = 0usize;
                            let mut next = text.len();

                            for (i, _) in text.char_indices() {
                                if i <= idx {
                                    prev = i;
                                }
                                if i >= idx {
                                    next = i;
                                    break;
                                }
                            }

                            let left = text[..prev].chars().rev().take(12).collect::<String>();
                            let left = left.chars().rev().collect::<String>();
                            let right = text[next..].chars().take(12).collect::<String>();
                            format!("{left}|{right}")
                        }

                        let bounds = p.bounds();
                        let pad = Px(12.0);

                        let inner = Rect::new(
                            Point::new(
                                Px(bounds.origin.x.0 + pad.0),
                                Px(bounds.origin.y.0 + pad.0),
                            ),
                            Size::new(
                                Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                                Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                            ),
                        );

                        let max_width = inner.size.width;
                        if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                            return;
                        }

                        let scale_factor = p.scale_factor();
                        let selection_bg = p.theme().color_required("selection.background");
                        let fg = p.theme().color_required("foreground");
                        let muted = p.theme().color_required("muted-foreground");

                        let (stats, stats_origin) = {
                            let (services, scene) = p.services_and_scene();
                            let mut st = paint_state.borrow_mut();

                            let sample = SAMPLES
                                .get(st.selected_sample)
                                .copied()
                                .unwrap_or(SAMPLES[0]);

                            let key = PreparedKey {
                                sample: st.selected_sample,
                                max_width_bits: max_width.0.to_bits(),
                                scale_bits: scale_factor.to_bits(),
                            };

                            let needs_prepare = st.blob.is_none()
                                || st.metrics.is_none()
                                || st.prepared_key != Some(key);
                            if needs_prepare {
                                if let Some(blob) = st.blob.take() {
                                    services.text().release(blob);
                                }

                                let style = TextStyle {
                                    font: FontId::ui(),
                                    size: Px(18.0),
                                    ..Default::default()
                                };

                                let constraints = TextConstraints {
                                    max_width: Some(max_width),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                    scale_factor,
                                };

                                let (blob, metrics) =
                                    services.text().prepare_str(sample.text, &style, constraints);
                                st.prepared_key = Some(key);
                                st.blob = Some(blob);
                                st.metrics = Some(metrics);
                                st.anchor = 0;
                                st.caret = 0;
                                st.affinity = CaretAffinity::Downstream;
                            }

                            let Some(blob) = st.blob else {
                                return;
                            };
                            let Some(metrics) = st.metrics else {
                                return;
                            };

                            let click_to_local = |global: Point| -> Point {
                                Point::new(
                                    Px(global.x.0 - inner.origin.x.0),
                                    Px(global.y.0 - inner.origin.y.0),
                                )
                            };

                            if let Some((pos, extend)) = st.pending_down.take() {
                                let local = click_to_local(pos);
                                let hit = services.hit_test_point(blob, local);
                                st.caret = hit.index;
                                st.affinity = hit.affinity;
                                if !extend {
                                    st.anchor = st.caret;
                                }
                            }

                            if st.dragging {
                                if let Some(pos) = st.last_drag_pos {
                                    let local = click_to_local(pos);
                                    let hit = services.hit_test_point(blob, local);
                                    st.caret = hit.index;
                                    st.affinity = hit.affinity;
                                }
                            }

                            let range = if st.anchor <= st.caret {
                                (st.anchor, st.caret)
                            } else {
                                (st.caret, st.anchor)
                            };

                            let clip = Rect::new(Point::new(Px(0.0), Px(0.0)), inner.size);
                            let mut rects: Vec<Rect> = Vec::new();
                            services.selection_rects_clipped(blob, range, clip, &mut rects);

                            scene.push(SceneOp::PushClipRect { rect: inner });
                            for r in rects {
                                let rect = Rect::new(
                                    Point::new(
                                        Px(inner.origin.x.0 + r.origin.x.0),
                                        Px(inner.origin.y.0 + r.origin.y.0),
                                    ),
                                    r.size,
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(0),
                                    rect,
                                    background: fret_core::Paint::Solid(selection_bg),

                                    border: Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }

                            let text_origin = Point::new(inner.origin.x, Px(inner.origin.y.0 + metrics.baseline.0));
                            scene.push(SceneOp::Text {
                                order: DrawOrder(1),
                                origin: text_origin,
                                text: blob,
                                color: fg,
                            });

                            let caret_rect = services.caret_rect(blob, st.caret, st.affinity);
                            let caret_rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + caret_rect.origin.x.0),
                                    Px(inner.origin.y.0 + caret_rect.origin.y.0),
                                ),
                                caret_rect.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(2),
                                rect: caret_rect,
                                background: fret_core::Paint::Solid(fg),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: Corners::all(Px(0.0)),
                            });

                            if let Some(pos) = st.last_drag_pos {
                                let dot = Rect::new(
                                    Point::new(Px(pos.x.0 - 2.0), Px(pos.y.0 - 2.0)),
                                    Size::new(Px(4.0), Px(4.0)),
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: dot,
                                    background: fret_core::Paint::Solid(fg),

                                    border: Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: Corners::all(Px(2.0)),
                                });
                            }

                            scene.push(SceneOp::PopClip);

                            let sample_text: &str = sample.text;
                            let context = format_utf8_context(sample_text, st.caret);

                            let stats = format!(
                                "sample: {} | caret: {} ({:?}) | anchor: {} | range: {:?} | context: {}",
                                sample.label, st.caret, st.affinity, st.anchor, range, context
                            );
                            let stats_origin = Point::new(
                                Px(bounds.origin.x.0 + 12.0),
                                Px(bounds.origin.y.0 + 10.0),
                            );
                            (stats, stats_origin)
                        };

                        let stats_style = TextStyle {
                            font: FontId::ui(),
                            size: Px(12.0),
                            ..Default::default()
                        };
                        let _ = p.text(
                            p.key(&"text_bidi_rtl_conformance_stats"),
                            DrawOrder(10),
                            stats_origin,
                            stats,
                            stats_style,
                            muted,
                            fret_ui::canvas::CanvasTextConstraints::default(),
                            scale_factor,
                        );
                    });

                    vec![canvas]
                });

                vec![content]
            },
        )
    };

    let panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |_cx| vec![sample_buttons, selectable_samples, diagnostic],
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-bidi-rtl-conformance-root"),
    );

    vec![header, panel]
}
