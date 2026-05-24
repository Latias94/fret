use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use fret::AppComponentCx;
use fret::app::AppRenderActionsExt as _;

pub(in crate::ui) fn preview_hit_test_only_paint_cache_probe(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui::element::{
        LayoutStyle, Length, PointerRegionProps, PositionStyle, SemanticsProps, StackProps,
    };
    use fret_ui_kit::prelude::CachedSubtreeProps;

    fn with_alpha(mut color: CoreColor, alpha: f32) -> CoreColor {
        color.a = alpha;
        color
    }

    fn absolute_pointer_region(left: Px, top: Px, width: Px, height: Px) -> PointerRegionProps {
        let mut pointer = PointerRegionProps::default();
        pointer.layout.position = PositionStyle::Absolute;
        pointer.layout.overflow = fret_ui::element::Overflow::Clip;
        pointer.layout.inset.left = Some(left).into();
        pointer.layout.inset.top = Some(top).into();
        pointer.layout.size.width = Length::Px(width);
        pointer.layout.size.height = Length::Px(height);
        pointer
    }

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
                                    .bg(ColorRef::Color(theme.color_token("background"))),
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_px(MetricRef::Px(Px(320.0))),
                            ),
                            move |cx| {
                                vec![
                                    cx.canvas(canvas, move |p| {
                                        let bounds = p.bounds();
                                        let accent_bg =
                                            with_alpha(p.theme().color_token("accent"), 0.10);
                                        let border_color = p.theme().color_token("border");
                                        let secondary_bg =
                                            with_alpha(p.theme().color_token("secondary"), 0.16);
                                        let muted_border = with_alpha(
                                            p.theme().color_token("muted-foreground"),
                                            0.35,
                                        );

                                        p.scene().push(SceneOp::Quad {
                                            order: DrawOrder(0),
                                            rect: bounds,
                                            background: fret_core::Paint::Solid(accent_bg).into(),

                                            border: Edges::all(Px(1.0)),
                                            border_paint: fret_core::Paint::Solid(border_color)
                                                .into(),
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
                                            background: fret_core::Paint::Solid(secondary_bg)
                                                .into(),

                                            border: Edges::all(Px(1.0)),
                                            border_paint: fret_core::Paint::Solid(muted_border)
                                                .into(),

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
        .test_id("ui-gallery-hit-test-only-probe-panel");

    let stale_path_cover_active =
        cx.local_model_keyed("hit_test_only_stale_path_cover_active", || false);
    let stale_path_last_hit = cx.local_model_keyed("hit_test_only_stale_path_last_hit", || {
        Arc::<str>::from("none")
    });
    let cover_active = cx
        .get_model_copied(&stale_path_cover_active, fret_ui::Invalidation::Layout)
        .unwrap_or(false);
    let last_hit = cx
        .get_model_cloned(&stale_path_last_hit, fret_ui::Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("none"));
    let status_label: Arc<str> = Arc::from(format!(
        "stale_path_hit={} cover_active={}",
        last_hit.as_ref(),
        cover_active as u8
    ));

    let status_label_for_text = status_label.clone();
    let status = doc_layout::control_readout_text(cx, status_label_for_text.clone())
        .a11y_role(fret_core::SemanticsRole::Status)
        .a11y_label(status_label)
        .test_id("ui-gallery-hit-test-only-stale-path-status");

    let reset_cover = stale_path_cover_active.clone();
    let reset_hit = stale_path_last_hit.clone();
    let reset = shadcn::Button::new("Reset stale-path probe")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host.models_mut().update(&reset_cover, |v| *v = false);
            let _ = host
                .models_mut()
                .update(&reset_hit, |v| *v = Arc::<str>::from("none"));
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-gallery-hit-test-only-stale-path-reset")
        .into_element(cx);

    let cached_cover = stale_path_cover_active.clone();
    let target_cover = stale_path_cover_active.clone();
    let target_hit = stale_path_last_hit.clone();
    let cover_hit = stale_path_last_hit.clone();
    let stale_path_stage = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        move |cx| {
            let cover_active = cx
                .get_model_copied(&cached_cover, fret_ui::Invalidation::Layout)
                .unwrap_or(false);

            let on_target_move: fret_ui::action::OnPointerMove =
                Arc::new(move |host, action_cx, _mv| {
                    let _ = host
                        .models_mut()
                        .update(&target_hit, |v| *v = Arc::<str>::from("target"));
                    let _ = host.models_mut().update(&target_cover, |v| *v = true);
                    host.invalidate(fret_ui::Invalidation::HitTestOnly);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                    true
                });

            let on_cover_move: fret_ui::action::OnPointerMove =
                Arc::new(move |host, action_cx, _mv| {
                    let _ = host
                        .models_mut()
                        .update(&cover_hit, |v| *v = Arc::<str>::from("cover"));
                    host.invalidate(fret_ui::Invalidation::HitTestOnly);
                    host.request_redraw(action_cx.window);
                    true
                });

            let mut stack_layout = LayoutStyle::default();
            stack_layout.size.width = Length::Fill;
            stack_layout.size.height = Length::Px(Px(220.0));
            stack_layout.overflow = fret_ui::element::Overflow::Clip;

            let stage = cx.stack_props(
                StackProps {
                    layout: stack_layout,
                },
                move |cx| {
                    let target = {
                        let pointer =
                            absolute_pointer_region(Px(80.0), Px(68.0), Px(260.0), Px(96.0));
                        let on_target_move = on_target_move.clone();
                        cx.pointer_region(pointer, move |cx| {
                            cx.pointer_region_on_pointer_move(on_target_move.clone());
                            vec![
                                ui::v_flex(|cx| {
                                    vec![
                                        doc_layout::control_label_text(cx, "Lower target"),
                                        doc_layout::control_readout_text(
                                            cx,
                                            "First sweep event primes the path cache here.",
                                        ),
                                    ]
                                })
                                .layout(LayoutRefinement::default().size_full())
                                .p(Space::N3)
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(with_alpha(
                                    cx.theme().color_token("primary"),
                                    0.12,
                                )))
                                .into_element(cx),
                            ]
                        })
                        .a11y_role(fret_core::SemanticsRole::Group)
                        .a11y_label("ui-gallery-hit-test-only-stale-path-target")
                        .test_id("ui-gallery-hit-test-only-stale-path-target")
                    };

                    let cover_left = if cover_active { Px(80.0) } else { Px(520.0) };
                    let cover_note = if cover_active {
                        "Moved over target; stale path must miss and fall back."
                    } else {
                        "Parked away from target before the sweep."
                    };
                    let cover = {
                        let pointer =
                            absolute_pointer_region(cover_left, Px(68.0), Px(260.0), Px(96.0));
                        let on_cover_move = on_cover_move.clone();
                        cx.pointer_region(pointer, move |cx| {
                            cx.pointer_region_on_pointer_move(on_cover_move.clone());
                            vec![
                                ui::v_flex(|cx| {
                                    vec![
                                        doc_layout::control_label_text(cx, "Higher-z cover"),
                                        doc_layout::control_readout_text(cx, cover_note),
                                    ]
                                })
                                .layout(LayoutRefinement::default().size_full())
                                .p(Space::N3)
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(with_alpha(
                                    cx.theme().color_token("destructive"),
                                    0.14,
                                )))
                                .into_element(cx),
                            ]
                        })
                        .a11y_role(fret_core::SemanticsRole::Group)
                        .a11y_label("ui-gallery-hit-test-only-stale-path-cover")
                        .test_id("ui-gallery-hit-test-only-stale-path-cover")
                    };

                    vec![target, cover]
                },
            );

            vec![
                stage
                    .a11y_role(fret_core::SemanticsRole::Group)
                    .a11y_label("ui-gallery-hit-test-only-stale-path-stage")
                    .test_id("ui-gallery-hit-test-only-stale-path-stage"),
            ]
        },
    );

    let stale_path_probe = ui::v_flex(|_cx| vec![status, reset, stale_path_stage])
        .layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(300.0))),
        )
        .gap(Space::N3)
        .into_element(cx)
        .test_id("ui-gallery-hit-test-only-stale-path-probe");

    let probe_region = DocSection::build(cx, "Probe region", panel)
        .descriptions([
            "Pointer moves over the probe region call `host.invalidate(Invalidation::HitTestOnly)` while layout and painted content remain stable.",
            "Use this page to validate `paint_cache_hit_test_only_replay_*` counters.",
        ])
        .no_shell()
        .max_w(Px(980.0));

    let stale_path_region = DocSection::build(cx, "Stale hit-path guard", stale_path_probe)
        .descriptions([
            "The first pointer-move event over the lower target moves a higher-z cover over the same point without clearing the move-only path cache.",
            "The next move must reject the stale cached target path, fall back to full hit-testing, and route to the higher-z cover.",
        ])
        .no_shell()
        .max_w(Px(980.0));

    let page = doc_layout::render_doc_page(
        cx,
        Some("Deterministically trigger `Invalidation::HitTestOnly` on a cache-eligible subtree."),
        vec![probe_region, stale_path_region],
    );

    vec![page.into_element(cx)]
}
