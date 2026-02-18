use super::super::super::super::super::*;
use crate::ui::doc_layout;
use fret_core::scene::TextOutlineV1;

pub(in crate::ui) fn preview_text_outline_stroke(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy, PartialEq)]
    struct PreparedKey {
        scale_bits: u32,
        size_bits: u32,
    }

    #[derive(Default)]
    struct OutlineStrokeState {
        injected: bool,
        outline_enabled: bool,
        outline_width_idx: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
    }

    const OUTLINE_WIDTHS: &[Px] = &[Px(1.0), Px(2.0), Px(3.0), Px(4.0), Px(6.0), Px(8.0)];

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(OutlineStrokeState::default())),
        |st| st.clone(),
    );

    {
        let mut st = state.borrow_mut();
        if !st.injected {
            let fonts = fret_fonts::default_fonts()
                .iter()
                .map(|b| b.to_vec())
                .collect::<Vec<_>>();
            cx.app
                .push_effect(fret_runtime::Effect::TextAddFonts { fonts });
            cx.app.request_redraw(cx.window);
            st.injected = true;
            st.outline_enabled = true;
            st.outline_width_idx = 2;
        }
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: exercise `SceneOp::Text.outline: Option<TextOutlineV1>` end-to-end."),
                cx.text("This page draws the same text twice: fill-only vs fill+outline, on a high-contrast backdrop."),
                cx.text("Tip: set FRET_TEXT_SYSTEM_FONTS=0 to validate deterministic bundled-font behavior."),
            ]
        },
    );

    fn toggle_button(
        cx: &mut ElementContext<'_, App>,
        label: &'static str,
        value: bool,
        test_id: &'static str,
        on_activate: fret_ui::action::OnActivate,
    ) -> AnyElement {
        let txt = format!("{label}: {}", if value { "on" } else { "off" });
        shadcn::Button::new(txt)
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(on_activate)
            .into_element(cx)
            .test_id(test_id)
    }

    let toolbar = {
        let state_outline = state.clone();
        let on_outline: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let mut st = state_outline.borrow_mut();
            st.outline_enabled = !st.outline_enabled;
            host.request_redraw(action_cx.window);
        });

        let state_width = state.clone();
        let on_width: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let mut st = state_width.borrow_mut();
            st.outline_width_idx = (st.outline_width_idx + 1) % OUTLINE_WIDTHS.len();
            host.request_redraw(action_cx.window);
        });

        let st = state.borrow();
        let outline_btn = toggle_button(
            cx,
            "outline",
            st.outline_enabled,
            "ui-gallery-text-outline-stroke-outline",
            on_outline,
        );

        let width_label = format!("width: {:.0}px", OUTLINE_WIDTHS[st.outline_width_idx].0);
        let width_btn = shadcn::Button::new(width_label)
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(on_width)
            .into_element(cx)
            .test_id("ui-gallery-text-outline-stroke-width");

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            |_cx| vec![outline_btn, width_btn],
        )
    };

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(420.0))),
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
                let inner = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + pad.0), Px(bounds.origin.y.0 + pad.0)),
                    Size::new(
                        Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                        Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                    ),
                );
                if inner.size.width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                    return;
                }

                let scale_factor = p.scale_factor();
                let muted = p.theme().color_token("muted-foreground");
                let label_key_fill = p.key(&"text_outline_stroke_label_fill");
                let label_key_outline = p.key(&"text_outline_stroke_label_outline");

                let baseline_y0 = Px(inner.origin.y.0 + 110.0);
                let baseline_y1 = Px(inner.origin.y.0 + 250.0);
                let x0 = Px(inner.origin.x.0 + 16.0);

                let label_style = TextStyle {
                    font: FontId::ui(),
                    size: Px(13.0),
                    ..Default::default()
                };

                let Some((label_y0, label_y1)) = (|| {
                    let (services, scene) = p.services_and_scene();

                    let mut style = fret_core::TextStyle {
                        font: fret_core::FontId::ui(),
                        size: Px(64.0),
                        ..Default::default()
                    };
                    style.weight = fret_core::FontWeight::SEMIBOLD;

                    let key = PreparedKey {
                        scale_bits: scale_factor.to_bits(),
                        size_bits: style.size.0.to_bits(),
                    };

                    let mut st = paint_state.borrow_mut();
                    let needs_prepare =
                        st.blob.is_none() || st.metrics.is_none() || st.prepared_key != Some(key);
                    if needs_prepare {
                        if let Some(blob) = st.blob.take() {
                            services.text().release(blob);
                        }
                        let constraints = fret_core::TextConstraints {
                            max_width: None,
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            scale_factor,
                            align: fret_core::TextAlign::Start,
                        };
                        let (blob, metrics) =
                            services
                                .text()
                                .prepare_str("Outline Stroke", &style, constraints);
                        st.prepared_key = Some(key);
                        st.blob = Some(blob);
                        st.metrics = Some(metrics);
                    }

                    let blob = st.blob?;
                    let metrics = st.metrics?;

                    // Backdrop: high-contrast tiles.
                    let tile = Px(24.0);
                    let cols = (inner.size.width.0 / tile.0).ceil().max(1.0) as i32;
                    let rows = (inner.size.height.0 / tile.0).ceil().max(1.0) as i32;
                    for y in 0..rows {
                        for x in 0..cols {
                            let is_dark = ((x + y) & 1) == 0;
                            let c = if is_dark {
                                fret_core::Color {
                                    r: 0.10,
                                    g: 0.10,
                                    b: 0.12,
                                    a: 1.0,
                                }
                            } else {
                                fret_core::Color {
                                    r: 0.22,
                                    g: 0.22,
                                    b: 0.26,
                                    a: 1.0,
                                }
                            };
                            let rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + x as f32 * tile.0),
                                    Px(inner.origin.y.0 + y as f32 * tile.0),
                                ),
                                Size::new(tile, tile),
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(c),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }
                    }

                    let fill = fret_core::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    };
                    let outline = fret_core::Color {
                        r: 0.05,
                        g: 0.10,
                        b: 0.95,
                        a: 1.0,
                    };

                    let outline_width = OUTLINE_WIDTHS
                        .get(st.outline_width_idx)
                        .copied()
                        .unwrap_or(Px(3.0));
                    let outline_desc = st.outline_enabled.then_some(TextOutlineV1 {
                        paint: fret_core::Paint::Solid(outline),
                        width_px: outline_width,
                    });

                    scene.push(SceneOp::Text {
                        order: DrawOrder(2),
                        origin: Point::new(x0, baseline_y0),
                        text: blob,
                        paint: fret_core::Paint::Solid(fill),
                        outline: None,
                        shadow: None,
                    });

                    scene.push(SceneOp::Text {
                        order: DrawOrder(2),
                        origin: Point::new(x0, baseline_y1),
                        text: blob,
                        paint: fret_core::Paint::Solid(fill),
                        outline: outline_desc,
                        shadow: None,
                    });

                    Some((
                        Px(baseline_y0.0 - metrics.baseline.0 - 18.0),
                        Px(baseline_y1.0 - metrics.baseline.0 - 18.0),
                    ))
                })() else {
                    return;
                };

                let _ = p.text(
                    label_key_fill,
                    DrawOrder(1),
                    Point::new(x0, label_y0),
                    "Fill only",
                    label_style.clone(),
                    muted,
                    fret_ui::canvas::CanvasTextConstraints::default(),
                    scale_factor,
                );
                let _ = p.text(
                    label_key_outline,
                    DrawOrder(1),
                    Point::new(x0, label_y1),
                    "Fill + outline",
                    label_style,
                    muted,
                    fret_ui::canvas::CanvasTextConstraints::default(),
                    scale_factor,
                );
            });

            vec![canvas]
        },
    );

    let page = doc_layout::wrap_preview_page(
        cx,
        None,
        "Text Outline/Stroke (v1)",
        vec![header, toolbar, panel],
    );

    vec![page]
}
