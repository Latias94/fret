use super::super::super::super::super::*;

pub(in crate::ui) fn preview_text_measure_overlay(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct Case {
        label: &'static str,
        text: &'static str,
        wrap: TextWrap,
        overflow: TextOverflow,
        height: Px,
    }

    const CASES: &[Case] = &[
        Case {
            label: "Wrap=None, Overflow=Clip (expect overflow past measured width)",
            text: "Left (fill) • A_very_long_token_without_spaces_that_should_not_wrap_but_can_overflow_the_box",
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            height: Px(56.0),
        },
        Case {
            label: "Wrap=Word, Overflow=Clip (expect multi-line height growth)",
            text: "Word wrap should break on spaces and increase measured height when max_width is tight.",
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=Grapheme, Overflow=Clip (expect long tokens to wrap)",
            text: "GraphemeWrap: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (and emoji 😀😀😀) should wrap without whitespace.",
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=None, Overflow=Ellipsis (expect measured width ~= max_width)",
            text: "Ellipsis overflow should clamp the visual width and replace the suffix…",
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            height: Px(56.0),
        },
    ];

    #[derive(Default)]
    struct MeasureOverlayState {
        last_metrics: Vec<Option<fret_core::TextMetrics>>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(MeasureOverlayState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: visualize measured text bounds vs allocated container bounds."),
                cx.text("Green = container bounds; Yellow = measured TextMetrics.size; Cyan = baseline."),
            ]
        },
    );

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(440.0))),
        ),
        move |cx| {
            let mut canvas = CanvasProps::default();
            canvas.layout.size.width = fret_ui::element::Length::Fill;
            canvas.layout.size.height = fret_ui::element::Length::Fill;
            canvas.layout.overflow = fret_ui::element::Overflow::Clip;
            canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let paint_state = state.clone();

            let canvas = cx.canvas(canvas, move |p| {
                let bounds = p.bounds();
                let pad = Px(14.0);
                let gap = Px(14.0);

                let outer = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + pad.0), Px(bounds.origin.y.0 + pad.0)),
                    Size::new(
                        Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                        Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                    ),
                );
                if outer.size.width.0 <= 0.0 || outer.size.height.0 <= 0.0 {
                    return;
                }

                let green = fret_core::Color {
                    r: 0.20,
                    g: 0.85,
                    b: 0.35,
                    a: 1.0,
                };
                let yellow = fret_core::Color {
                    r: 0.95,
                    g: 0.85,
                    b: 0.10,
                    a: 1.0,
                };
                let cyan = fret_core::Color {
                    r: 0.10,
                    g: 0.80,
                    b: 0.95,
                    a: 1.0,
                };

                let fg = p.theme().color_required("foreground");
                let muted = p.theme().color_required("muted-foreground");
                let bg = p.theme().color_required("background");
                let border = p.theme().color_required("border");

                let scale = p.scale_factor();
                let mut y = outer.origin.y;
                let scope = p.key_scope(&"text_measure_overlay");

                let mut st = paint_state.borrow_mut();
                if st.last_metrics.len() < CASES.len() {
                    st.last_metrics.resize(CASES.len(), None);
                }

                for (i, case) in CASES.iter().enumerate() {
                    let case_rect = Rect::new(
                        Point::new(outer.origin.x, y),
                        Size::new(outer.size.width, case.height),
                    );

                    // Case chrome.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: case_rect,
                        background: fret_core::Paint::Solid(bg),

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(border),

                        corner_radii: Corners::all(Px(8.0)),
                    });

                    let label_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let label_metrics = p.text(
                        p.child_key(scope, &format!("label_{i}")).0,
                        DrawOrder(1),
                        Point::new(case_rect.origin.x + Px(10.0), case_rect.origin.y + Px(16.0)),
                        case.label,
                        label_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(Px((case_rect.size.width.0 - 20.0).max(0.0))),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        },
                        scale,
                    );

                    let text_box = Rect::new(
                        Point::new(
                            case_rect.origin.x + Px(10.0),
                            Px(case_rect.origin.y.0 + 16.0 + label_metrics.size.height.0 + 8.0),
                        ),
                        Size::new(
                            Px((case_rect.size.width.0 - 20.0).max(0.0)),
                            Px((case_rect.size.height.0
                                - 16.0
                                - label_metrics.size.height.0
                                - 18.0)
                                .max(0.0)),
                        ),
                    );

                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: text_box,
                        background: fret_core::Paint::TRANSPARENT,

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(green),

                        corner_radii: Corners::all(Px(6.0)),
                    });

                    let text_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    };

                    let baseline_y = match st.last_metrics[i] {
                        Some(m) => text_box.origin.y + m.baseline,
                        None => text_box.origin.y + Px(text_style.size.0 * 0.8),
                    };

                    let metrics = p.text(
                        p.child_key(scope, &format!("text_{i}")).0,
                        DrawOrder(2),
                        Point::new(text_box.origin.x, baseline_y),
                        case.text,
                        text_style,
                        fg,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(text_box.size.width),
                            wrap: case.wrap,
                            overflow: case.overflow,
                        },
                        scale,
                    );
                    st.last_metrics[i] = Some(metrics);

                    // Baseline.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: Rect::new(
                            Point::new(text_box.origin.x, text_box.origin.y + metrics.baseline),
                            Size::new(text_box.size.width, Px(1.0)),
                        ),
                        background: fret_core::Paint::Solid(cyan),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: Corners::all(Px(0.0)),
                    });

                    // Measured text box.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(4),
                        rect: Rect::new(text_box.origin, metrics.size),
                        background: fret_core::Paint::TRANSPARENT,

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(yellow),

                        corner_radii: Corners::all(Px(0.0)),
                    });

                    y = Px(y.0 + case.height.0 + gap.0);
                    if y.0 >= outer.origin.y.0 + outer.size.height.0 {
                        break;
                    }
                }
            });

            vec![canvas]
        },
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-measure-overlay-root"),
    );

    vec![header, panel]
}
