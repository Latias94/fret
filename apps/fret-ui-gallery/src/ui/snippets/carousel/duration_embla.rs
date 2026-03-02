// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    duration_fast_api_snapshot: Option<Model<shadcn::CarouselApiSnapshot>>,
    duration_slow_api_snapshot: Option<Model<shadcn::CarouselApiSnapshot>>,
    duration_fast_settling: Option<Model<bool>>,
    duration_slow_settling: Option<Model<bool>>,
    duration_fast_at_snap: Option<Model<bool>>,
    duration_slow_at_snap: Option<Model<bool>>,
    duration_fast_can_next: Option<Model<bool>>,
    duration_slow_can_next: Option<Model<bool>>,
    duration_fast_selected_1: Option<Model<bool>>,
    duration_slow_selected_1: Option<Model<bool>>,
    duration_fast_engine_present: Option<Model<bool>>,
    duration_slow_engine_present: Option<Model<bool>>,
    duration_fast_scroll_duration_fast: Option<Model<bool>>,
    duration_slow_scroll_duration_slow: Option<Model<bool>>,
    duration_fast_selected_snap_large: Option<Model<bool>>,
    duration_slow_selected_snap_large: Option<Model<bool>>,
    duration_fast_embla_settling: Option<Model<bool>>,
    duration_slow_embla_settling: Option<Model<bool>>,
    duration_fast_embla_enabled: Option<Model<bool>>,
    duration_slow_embla_enabled: Option<Model<bool>>,
}

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

fn slide_card(cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let number = ui::text(cx, format!("{idx}"))
        .text_size_px(visual.text_px)
        .line_height_px(visual.line_height_px)
        .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
        .font_semibold()
        .into_element(cx);

    let content = cx.flex(
        FlexProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().aspect_ratio(1.0),
            ),
            direction: fret_core::Axis::Horizontal,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            padding: Edges::all(Px(24.0)).into(),
            ..Default::default()
        },
        move |_cx| vec![number],
    );

    shadcn::Card::new([content]).into_element(cx)
}

fn slide(cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual) -> AnyElement {
    let card = slide_card(cx, idx, visual);
    ui::container(cx, move |_cx| vec![card])
        .w_full()
        .p_1()
        .into_element(cx)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_xs = Px(320.0);

    // Embla duration: demonstrate that smaller durations settle faster for button navigation.
    // Note: drag release shaping uses Embla's hard-coded `baseDuration` (see workstream docs), so
    // this demo focuses on prev/next.
    let duration_fast_api_snapshot = cx
        .with_state(Models::default, |st| st.duration_fast_api_snapshot.clone())
        .unwrap_or_else(|| {
            let model: Model<shadcn::CarouselApiSnapshot> = cx
                .app
                .models_mut()
                .insert(shadcn::CarouselApiSnapshot::default());
            cx.with_state(Models::default, |st| {
                st.duration_fast_api_snapshot = Some(model.clone());
            });
            model
        });
    let duration_slow_api_snapshot = cx
        .with_state(Models::default, |st| st.duration_slow_api_snapshot.clone())
        .unwrap_or_else(|| {
            let model: Model<shadcn::CarouselApiSnapshot> = cx
                .app
                .models_mut()
                .insert(shadcn::CarouselApiSnapshot::default());
            cx.with_state(Models::default, |st| {
                st.duration_slow_api_snapshot = Some(model.clone());
            });
            model
        });
    let duration_fast_settling = cx
        .with_state(Models::default, |st| st.duration_fast_settling.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_settling = Some(model.clone());
            });
            model
        });
    let duration_slow_settling = cx
        .with_state(Models::default, |st| st.duration_slow_settling.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_settling = Some(model.clone());
            });
            model
        });
    let duration_fast_at_snap = cx
        .with_state(Models::default, |st| st.duration_fast_at_snap.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_at_snap = Some(model.clone());
            });
            model
        });
    let duration_slow_at_snap = cx
        .with_state(Models::default, |st| st.duration_slow_at_snap.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_at_snap = Some(model.clone());
            });
            model
        });
    let duration_fast_can_next = cx
        .with_state(Models::default, |st| st.duration_fast_can_next.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_can_next = Some(model.clone());
            });
            model
        });
    let duration_slow_can_next = cx
        .with_state(Models::default, |st| st.duration_slow_can_next.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_can_next = Some(model.clone());
            });
            model
        });
    let duration_fast_selected_1 = cx
        .with_state(Models::default, |st| st.duration_fast_selected_1.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_selected_1 = Some(model.clone());
            });
            model
        });
    let duration_slow_selected_1 = cx
        .with_state(Models::default, |st| st.duration_slow_selected_1.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_selected_1 = Some(model.clone());
            });
            model
        });

    let duration_fast_engine_present = cx
        .with_state(Models::default, |st| {
            st.duration_fast_engine_present.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_engine_present = Some(model.clone());
            });
            model
        });
    let duration_slow_engine_present = cx
        .with_state(Models::default, |st| {
            st.duration_slow_engine_present.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_engine_present = Some(model.clone());
            });
            model
        });
    let duration_fast_scroll_duration_fast = cx
        .with_state(Models::default, |st| {
            st.duration_fast_scroll_duration_fast.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_scroll_duration_fast = Some(model.clone());
            });
            model
        });
    let duration_slow_scroll_duration_slow = cx
        .with_state(Models::default, |st| {
            st.duration_slow_scroll_duration_slow.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_scroll_duration_slow = Some(model.clone());
            });
            model
        });

    let duration_fast_selected_snap_large = cx
        .with_state(Models::default, |st| {
            st.duration_fast_selected_snap_large.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_selected_snap_large = Some(model.clone());
            });
            model
        });
    let duration_slow_selected_snap_large = cx
        .with_state(Models::default, |st| {
            st.duration_slow_selected_snap_large.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_selected_snap_large = Some(model.clone());
            });
            model
        });

    let duration_fast_embla_settling = cx
        .with_state(Models::default, |st| {
            st.duration_fast_embla_settling.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_embla_settling = Some(model.clone());
            });
            model
        });
    let duration_slow_embla_settling = cx
        .with_state(Models::default, |st| {
            st.duration_slow_embla_settling.clone()
        })
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_embla_settling = Some(model.clone());
            });
            model
        });

    let duration_fast_embla_enabled = cx
        .with_state(Models::default, |st| st.duration_fast_embla_enabled.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_fast_embla_enabled = Some(model.clone());
            });
            model
        });
    let duration_slow_embla_enabled = cx
        .with_state(Models::default, |st| st.duration_slow_embla_enabled.clone())
        .unwrap_or_else(|| {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.duration_slow_embla_enabled = Some(model.clone());
            });
            model
        });

    let duration_fast_snapshot_now = cx
        .watch_model(&duration_fast_api_snapshot)
        .copied()
        .unwrap_or_default();
    let duration_slow_snapshot_now = cx
        .watch_model(&duration_slow_api_snapshot)
        .copied()
        .unwrap_or_default();

    let duration_fast_settling_now = cx
        .watch_model(&duration_fast_settling)
        .copied()
        .unwrap_or(false);
    if duration_fast_settling_now != duration_fast_snapshot_now.settling {
        let _ = cx.app.models_mut().update(&duration_fast_settling, |v| {
            *v = duration_fast_snapshot_now.settling
        });
    }
    let duration_slow_settling_now = cx
        .watch_model(&duration_slow_settling)
        .copied()
        .unwrap_or(false);
    if duration_slow_settling_now != duration_slow_snapshot_now.settling {
        let _ = cx.app.models_mut().update(&duration_slow_settling, |v| {
            *v = duration_slow_snapshot_now.settling
        });
    }
    let duration_fast_at_snap_now = cx
        .watch_model(&duration_fast_at_snap)
        .copied()
        .unwrap_or(false);
    if duration_fast_at_snap_now != duration_fast_snapshot_now.at_selected_snap {
        let _ = cx.app.models_mut().update(&duration_fast_at_snap, |v| {
            *v = duration_fast_snapshot_now.at_selected_snap
        });
    }
    let duration_slow_at_snap_now = cx
        .watch_model(&duration_slow_at_snap)
        .copied()
        .unwrap_or(false);
    if duration_slow_at_snap_now != duration_slow_snapshot_now.at_selected_snap {
        let _ = cx.app.models_mut().update(&duration_slow_at_snap, |v| {
            *v = duration_slow_snapshot_now.at_selected_snap
        });
    }

    let duration_fast_can_next_now = cx
        .watch_model(&duration_fast_can_next)
        .copied()
        .unwrap_or(false);
    if duration_fast_can_next_now != duration_fast_snapshot_now.can_scroll_next {
        let _ = cx.app.models_mut().update(&duration_fast_can_next, |v| {
            *v = duration_fast_snapshot_now.can_scroll_next
        });
    }
    let duration_slow_can_next_now = cx
        .watch_model(&duration_slow_can_next)
        .copied()
        .unwrap_or(false);
    if duration_slow_can_next_now != duration_slow_snapshot_now.can_scroll_next {
        let _ = cx.app.models_mut().update(&duration_slow_can_next, |v| {
            *v = duration_slow_snapshot_now.can_scroll_next
        });
    }

    let duration_fast_selected_1_now = cx
        .watch_model(&duration_fast_selected_1)
        .copied()
        .unwrap_or(false);
    let duration_fast_selected_1_next = duration_fast_snapshot_now.selected_index == 1;
    if duration_fast_selected_1_now != duration_fast_selected_1_next {
        let _ = cx.app.models_mut().update(&duration_fast_selected_1, |v| {
            *v = duration_fast_selected_1_next
        });
    }

    let duration_slow_selected_1_now = cx
        .watch_model(&duration_slow_selected_1)
        .copied()
        .unwrap_or(false);
    let duration_slow_selected_1_next = duration_slow_snapshot_now.selected_index == 1;
    if duration_slow_selected_1_now != duration_slow_selected_1_next {
        let _ = cx.app.models_mut().update(&duration_slow_selected_1, |v| {
            *v = duration_slow_selected_1_next
        });
    }

    let duration_fast_engine_present_now = cx
        .watch_model(&duration_fast_engine_present)
        .copied()
        .unwrap_or(false);
    let duration_fast_engine_present_next = duration_fast_snapshot_now.embla_engine_present;
    if duration_fast_engine_present_now != duration_fast_engine_present_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_fast_engine_present, |v| {
                *v = duration_fast_engine_present_next
            });
    }
    let duration_slow_engine_present_now = cx
        .watch_model(&duration_slow_engine_present)
        .copied()
        .unwrap_or(false);
    let duration_slow_engine_present_next = duration_slow_snapshot_now.embla_engine_present;
    if duration_slow_engine_present_now != duration_slow_engine_present_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_slow_engine_present, |v| {
                *v = duration_slow_engine_present_next
            });
    }

    let duration_fast_scroll_duration_fast_now = cx
        .watch_model(&duration_fast_scroll_duration_fast)
        .copied()
        .unwrap_or(false);
    let duration_fast_scroll_duration_fast_next = duration_fast_snapshot_now.embla_scroll_duration
        > 0.0
        && duration_fast_snapshot_now.embla_scroll_duration <= 20.0;
    if duration_fast_scroll_duration_fast_now != duration_fast_scroll_duration_fast_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_fast_scroll_duration_fast, |v| {
                *v = duration_fast_scroll_duration_fast_next
            });
    }

    let duration_slow_scroll_duration_slow_now = cx
        .watch_model(&duration_slow_scroll_duration_slow)
        .copied()
        .unwrap_or(false);
    let duration_slow_scroll_duration_slow_next =
        duration_slow_snapshot_now.embla_scroll_duration >= 100.0;
    if duration_slow_scroll_duration_slow_now != duration_slow_scroll_duration_slow_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_slow_scroll_duration_slow, |v| {
                *v = duration_slow_scroll_duration_slow_next
            });
    }

    let duration_fast_selected_snap_large_now = cx
        .watch_model(&duration_fast_selected_snap_large)
        .copied()
        .unwrap_or(false);
    let duration_fast_selected_snap_large_next =
        duration_fast_snapshot_now.selected_snap_px >= 50.0;
    if duration_fast_selected_snap_large_now != duration_fast_selected_snap_large_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_fast_selected_snap_large, |v| {
                *v = duration_fast_selected_snap_large_next
            });
    }

    let duration_slow_selected_snap_large_now = cx
        .watch_model(&duration_slow_selected_snap_large)
        .copied()
        .unwrap_or(false);
    let duration_slow_selected_snap_large_next =
        duration_slow_snapshot_now.selected_snap_px >= 50.0;
    if duration_slow_selected_snap_large_now != duration_slow_selected_snap_large_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_slow_selected_snap_large, |v| {
                *v = duration_slow_selected_snap_large_next
            });
    }

    let duration_fast_embla_settling_now = cx
        .watch_model(&duration_fast_embla_settling)
        .copied()
        .unwrap_or(false);
    let duration_fast_embla_settling_next = duration_fast_snapshot_now.embla_settling;
    if duration_fast_embla_settling_now != duration_fast_embla_settling_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_fast_embla_settling, |v| {
                *v = duration_fast_embla_settling_next
            });
    }

    let duration_slow_embla_settling_now = cx
        .watch_model(&duration_slow_embla_settling)
        .copied()
        .unwrap_or(false);
    let duration_slow_embla_settling_next = duration_slow_snapshot_now.embla_settling;
    if duration_slow_embla_settling_now != duration_slow_embla_settling_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_slow_embla_settling, |v| {
                *v = duration_slow_embla_settling_next
            });
    }

    let duration_fast_embla_enabled_now = cx
        .watch_model(&duration_fast_embla_enabled)
        .copied()
        .unwrap_or(false);
    let duration_fast_embla_enabled_next = duration_fast_snapshot_now.embla_engine_enabled;
    if duration_fast_embla_enabled_now != duration_fast_embla_enabled_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_fast_embla_enabled, |v| {
                *v = duration_fast_embla_enabled_next
            });
    }

    let duration_slow_embla_enabled_now = cx
        .watch_model(&duration_slow_embla_enabled)
        .copied()
        .unwrap_or(false);
    let duration_slow_embla_enabled_next = duration_slow_snapshot_now.embla_engine_enabled;
    if duration_slow_embla_enabled_now != duration_slow_embla_enabled_next {
        let _ = cx
            .app
            .models_mut()
            .update(&duration_slow_embla_enabled, |v| {
                *v = duration_slow_embla_enabled_next
            });
    }

    let duration_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let duration_items_fast = (1..=5)
        .map(|idx| slide(cx, idx, duration_visual))
        .collect::<Vec<_>>();
    let duration_items_slow = (1..=5)
        .map(|idx| slide(cx, idx, duration_visual))
        .collect::<Vec<_>>();
    let duration_fast = shadcn::Carousel::new(duration_items_fast)
        .opts(
            shadcn::CarouselOptions::new()
                .embla_engine(true)
                .embla_duration(6.0)
                .ignore_reduced_motion(true),
        )
        .api_snapshot_model(duration_fast_api_snapshot.clone())
        .refine_layout(LayoutRefinement::default().w_full())
        .test_id("ui-gallery-carousel-duration-fast")
        .into_element(cx);
    let duration_slow = shadcn::Carousel::new(duration_items_slow)
        .opts(
            shadcn::CarouselOptions::new()
                .embla_engine(true)
                .embla_duration(200.0)
                .ignore_reduced_motion(true),
        )
        .api_snapshot_model(duration_slow_api_snapshot.clone())
        .refine_layout(LayoutRefinement::default().w_full())
        .test_id("ui-gallery-carousel-duration-slow")
        .into_element(cx);

    cx.flex(
        FlexProps {
            layout: decl_style::layout_style(
                &Theme::global(&*cx.app).snapshot(),
                LayoutRefinement::default().w_full(),
            ),
            direction: fret_core::Axis::Horizontal,
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            // The shadcn Carousel controls sit outside the viewport (`-left-12` / `-right-12`).
            // When rendering multiple carousels side-by-side, keep enough horizontal spacing so
            // the controls do not overlap and become unclickable.
            gap: Px(96.0).into(),
            ..Default::default()
        },
        move |cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let fast_row = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                    direction: fret_core::Axis::Horizontal,
                    justify: MainAlign::SpaceBetween,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    let label = ui::text(cx, "Fast (embla_duration=6)")
                        .text_sm()
                        .font_semibold()
                        .into_element(cx);
                    let indicators = cx.flex(
                        FlexProps {
                            direction: fret_core::Axis::Horizontal,
                            justify: MainAlign::End,
                            align: CrossAlign::Center,
                            gap: Px(8.0).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            let can_next = shadcn::Checkbox::new(duration_fast_can_next.clone())
                                .a11y_label("canScrollNext")
                                .disabled(true)
                                .test_id("ui-gallery-carousel-duration-fast-can-next")
                                .into_element(cx);
                            let embla_enabled =
                                shadcn::Checkbox::new(duration_fast_embla_enabled.clone())
                                    .a11y_label("emblaEngineEnabled")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-fast-embla-enabled")
                                    .into_element(cx);
                            let engine_present =
                                shadcn::Checkbox::new(duration_fast_engine_present.clone())
                                    .a11y_label("emblaEnginePresent")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-fast-engine-present")
                                    .into_element(cx);
                            let scroll_duration_fast =
                                shadcn::Checkbox::new(duration_fast_scroll_duration_fast.clone())
                                    .a11y_label("emblaScrollDurationIsFast")
                                    .disabled(true)
                                    .test_id(
                                        "ui-gallery-carousel-duration-fast-scroll-duration-fast",
                                    )
                                    .into_element(cx);
                            let embla_settling =
                                shadcn::Checkbox::new(duration_fast_embla_settling.clone())
                                    .a11y_label("emblaSettling")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-fast-embla-settling")
                                    .into_element(cx);
                            let selected_1 =
                                shadcn::Checkbox::new(duration_fast_selected_1.clone())
                                    .a11y_label("selectedIndexIs1")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-fast-selected-1")
                                    .into_element(cx);
                            let snap_large =
                                shadcn::Checkbox::new(duration_fast_selected_snap_large.clone())
                                    .a11y_label("selectedSnapIsLarge")
                                    .disabled(true)
                                    .test_id(
                                        "ui-gallery-carousel-duration-fast-selected-snap-large",
                                    )
                                    .into_element(cx);
                            let settling = shadcn::Checkbox::new(duration_fast_settling.clone())
                                .a11y_label("settling")
                                .disabled(true)
                                .test_id("ui-gallery-carousel-duration-fast-settling")
                                .into_element(cx);
                            let at_snap = shadcn::Checkbox::new(duration_fast_at_snap.clone())
                                .a11y_label("atSelectedSnap")
                                .disabled(true)
                                .test_id("ui-gallery-carousel-duration-fast-at-snap")
                                .into_element(cx);
                            vec![
                                can_next,
                                embla_enabled,
                                engine_present,
                                scroll_duration_fast,
                                embla_settling,
                                selected_1,
                                snap_large,
                                settling,
                                at_snap,
                            ]
                        },
                    );
                    vec![label, indicators]
                },
            );
            let fast_col = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        // Carousel controls are positioned outside the viewport (`-left-12` /
                        // `-right-12`). Keep overflow visible so the controls remain clickable.
                        LayoutRefinement::default()
                            .w_full()
                            .max_w(max_w_xs)
                            .overflow_visible(),
                    ),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    gap: Px(12.0).into(),
                    ..Default::default()
                },
                move |_cx| vec![fast_row, duration_fast],
            );

            let slow_row = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                    direction: fret_core::Axis::Horizontal,
                    justify: MainAlign::SpaceBetween,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    let label = ui::text(cx, "Slow (embla_duration=200)")
                        .text_sm()
                        .font_semibold()
                        .into_element(cx);
                    let indicators = cx.flex(
                        FlexProps {
                            direction: fret_core::Axis::Horizontal,
                            justify: MainAlign::End,
                            align: CrossAlign::Center,
                            gap: Px(8.0).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            let can_next = shadcn::Checkbox::new(duration_slow_can_next.clone())
                                .a11y_label("canScrollNext")
                                .disabled(true)
                                .test_id("ui-gallery-carousel-duration-slow-can-next")
                                .into_element(cx);
                            let embla_enabled =
                                shadcn::Checkbox::new(duration_slow_embla_enabled.clone())
                                    .a11y_label("emblaEngineEnabled")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-slow-embla-enabled")
                                    .into_element(cx);
                            let engine_present =
                                shadcn::Checkbox::new(duration_slow_engine_present.clone())
                                    .a11y_label("emblaEnginePresent")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-slow-engine-present")
                                    .into_element(cx);
                            let scroll_duration_slow =
                                shadcn::Checkbox::new(duration_slow_scroll_duration_slow.clone())
                                    .a11y_label("emblaScrollDurationIsSlow")
                                    .disabled(true)
                                    .test_id(
                                        "ui-gallery-carousel-duration-slow-scroll-duration-slow",
                                    )
                                    .into_element(cx);
                            let embla_settling =
                                shadcn::Checkbox::new(duration_slow_embla_settling.clone())
                                    .a11y_label("emblaSettling")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-slow-embla-settling")
                                    .into_element(cx);
                            let selected_1 =
                                shadcn::Checkbox::new(duration_slow_selected_1.clone())
                                    .a11y_label("selectedIndexIs1")
                                    .disabled(true)
                                    .test_id("ui-gallery-carousel-duration-slow-selected-1")
                                    .into_element(cx);
                            let snap_large =
                                shadcn::Checkbox::new(duration_slow_selected_snap_large.clone())
                                    .a11y_label("selectedSnapIsLarge")
                                    .disabled(true)
                                    .test_id(
                                        "ui-gallery-carousel-duration-slow-selected-snap-large",
                                    )
                                    .into_element(cx);
                            let settling = shadcn::Checkbox::new(duration_slow_settling.clone())
                                .a11y_label("settling")
                                .disabled(true)
                                .test_id("ui-gallery-carousel-duration-slow-settling")
                                .into_element(cx);
                            let at_snap = shadcn::Checkbox::new(duration_slow_at_snap.clone())
                                .a11y_label("atSelectedSnap")
                                .disabled(true)
                                .test_id("ui-gallery-carousel-duration-slow-at-snap")
                                .into_element(cx);
                            vec![
                                can_next,
                                embla_enabled,
                                engine_present,
                                scroll_duration_slow,
                                embla_settling,
                                selected_1,
                                snap_large,
                                settling,
                                at_snap,
                            ]
                        },
                    );
                    vec![label, indicators]
                },
            );
            let slow_col = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .w_full()
                            .max_w(max_w_xs)
                            .overflow_visible(),
                    ),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    gap: Px(12.0).into(),
                    ..Default::default()
                },
                move |_cx| vec![slow_row, duration_slow],
            );

            vec![fast_col, slow_col]
        },
    )
}
// endregion: example
