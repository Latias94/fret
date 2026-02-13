use super::super::super::super::*;

pub(in crate::ui) fn preview_hit_test_only_paint_cache_probe(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui::element::SemanticsProps;

    fn with_alpha(mut color: CoreColor, alpha: f32) -> CoreColor {
        color.a = alpha;
        color
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: deterministically trigger HitTestOnly invalidation on a cache-eligible subtree."),
                cx.text("Pointer moves over the probe region call `host.invalidate(Invalidation::HitTestOnly)` while layout and painted content remain stable."),
                cx.text("Use this page to validate `paint_cache_hit_test_only_replay_*` counters."),
            ]
        },
    );

    let panel = cx
        .semantics_with_id(
            SemanticsProps {
                role: fret_core::SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-hit-test-only-probe-region")),
                ..Default::default()
            },
            move |cx, _id| {
                let on_move: fret_ui::action::OnPointerMove =
                    Arc::new(move |host, action_cx, _mv| {
                        host.invalidate(fret_ui::Invalidation::HitTestOnly);
                        host.request_redraw(action_cx.window);
                        true
                    });

                let mut pointer = fret_ui::element::PointerRegionProps::default();
                pointer.layout.size.width = fret_ui::element::Length::Fill;
                pointer.layout.size.height = fret_ui::element::Length::Fill;
                pointer.layout.overflow = fret_ui::element::Overflow::Clip;

                let mut canvas = CanvasProps::default();
                canvas.layout.size.width = fret_ui::element::Length::Fill;
                canvas.layout.size.height = fret_ui::element::Length::Fill;
                canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                let region = cx.pointer_region(pointer, move |cx| {
                    cx.pointer_region_on_pointer_move(on_move.clone());

                    vec![
                        cx.container(
                            decl_style::container_props(
                                theme,
                                ChromeRefinement::default()
                                    .border_1()
                                    .rounded(Radius::Md)
                                    .bg(ColorRef::Color(theme.color_required("background"))),
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_px(MetricRef::Px(Px(320.0))),
                            ),
                            move |cx| {
                                vec![
                                    cx.canvas(canvas, move |p| {
                                        let bounds = p.bounds();
                                        let accent_bg =
                                            with_alpha(p.theme().color_required("accent"), 0.10);
                                        let border_color = p.theme().color_required("border");
                                        let secondary_bg =
                                            with_alpha(p.theme().color_required("secondary"), 0.16);
                                        let muted_border = with_alpha(
                                            p.theme().color_required("muted-foreground"),
                                            0.35,
                                        );

                                        p.scene().push(SceneOp::Quad {
                                            order: DrawOrder(0),
                                            rect: bounds,
                                            background: fret_core::Paint::Solid(accent_bg),

                                            border: Edges::all(Px(1.0)),
                                            border_paint: fret_core::Paint::Solid(border_color),
                                            corner_radii: Corners::all(Px(8.0)),
                                        });

                                        let guide = Rect::new(
                                            Point::new(
                                                Px(bounds.origin.x.0 + 48.0),
                                                Px(bounds.origin.y.0 + 36.0),
                                            ),
                                            Size::new(
                                                Px((bounds.size.width.0 - 96.0).max(0.0)),
                                                Px((bounds.size.height.0 - 72.0).max(0.0)),
                                            ),
                                        );
                                        p.scene().push(SceneOp::Quad {
                                            order: DrawOrder(0),
                                            rect: guide,
                                            background: fret_core::Paint::Solid(secondary_bg),

                                            border: Edges::all(Px(1.0)),
                                            border_paint: fret_core::Paint::Solid(muted_border),

                                            corner_radii: Corners::all(Px(6.0)),
                                        });
                                    })
                                    .test_id("ui-gallery-hit-test-only-probe-canvas"),
                                ]
                            },
                        )
                        .test_id("ui-gallery-hit-test-only-probe-region"),
                    ]
                });

                vec![region]
            },
        )
        .test_id("ui-gallery-hit-test-only-probe-region");

    vec![header, panel]
}
