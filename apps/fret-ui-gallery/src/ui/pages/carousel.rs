use super::super::*;

use std::sync::Arc;
use std::time::Duration;

use crate::ui::doc_layout::{self, DocSection};

use fret_ui::element::{CrossAlign, FlexProps, MainAlign};

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.carousel_page", |cx| {
        #[derive(Default)]
        struct CarouselPageState {
            demo_inner_clicked: Option<Model<bool>>,
            demo_dnd_pointer: Option<Model<Option<fret_core::PointerId>>>,
            demo_dnd_dragging: Option<Model<bool>>,
            demo_dnd_long_press_pointer: Option<Model<Option<fret_core::PointerId>>>,
            api_snapshot: Option<Model<shadcn::CarouselApiSnapshot>>,
            expandable_selected: Option<Model<Option<usize>>>,
        }

    #[derive(Debug, Clone, Copy)]
    struct SlideVisual {
        text_px: Px,
        line_height_px: Px,
    }

    let slide_card = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
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
    };

    let slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        let card = slide_card(cx, idx, visual);
        ui::container(cx, move |_cx| vec![card]).p_1().into_element(cx)
    };

    // API demo matches upstream `carousel-api`: no `p-1` wrapper around the card.
    let slide_unwrapped = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        slide_card(cx, idx, visual)
    };

    // Match shadcn/ui v4 docs example widths:
    // - `max-w-xs` for demo + orientation.
    // - `max-w-sm` for sizing/spacing examples.
    let max_w_xs = Px(320.0);
    let max_w_sm = Px(384.0);

    // Demo: include a descendant pressable so diag scripts can gate pointer propagation
    // (drag-from-descendant should not activate; click should).
    let demo_inner_clicked =
        cx.with_state(CarouselPageState::default, |st| st.demo_inner_clicked.clone());
    let demo_inner_clicked = match demo_inner_clicked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(CarouselPageState::default, |st| {
                st.demo_inner_clicked = Some(model.clone());
            });
            model
        }
    };
    let demo_inner_clicked_now = cx
        .watch_model(&demo_inner_clicked)
        .copied()
        .unwrap_or(false);
    let toggle_demo_inner_clicked = {
        let demo_inner_clicked = demo_inner_clicked.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                let _ = host
                    .models_mut()
                    .update(&demo_inner_clicked, |v| *v = !*v);
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    // Demo DnD handle (MVP): show a handle-only DnD activation that does not compete with the
    // carousel swipe gesture.
    const DEMO_DND_KIND: fret_runtime::DragKindId = fret_runtime::DragKindId(101);
    let demo_dnd_pointer =
        cx.with_state(CarouselPageState::default, |st| st.demo_dnd_pointer.clone());
    let demo_dnd_pointer = match demo_dnd_pointer {
        Some(model) => model,
        None => {
            let model: Model<Option<fret_core::PointerId>> = cx.app.models_mut().insert(None);
            cx.with_state(CarouselPageState::default, |st| {
                st.demo_dnd_pointer = Some(model.clone());
            });
            model
        }
    };

    let demo_dnd_dragging =
        cx.with_state(CarouselPageState::default, |st| st.demo_dnd_dragging.clone());
    let demo_dnd_dragging = match demo_dnd_dragging {
        Some(model) => model,
        None => {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(CarouselPageState::default, |st| {
                st.demo_dnd_dragging = Some(model.clone());
            });
            model
        }
    };
    let demo_dnd_dragging_now = cx
        .watch_model(&demo_dnd_dragging)
        .copied()
        .unwrap_or(false);
    let demo_dnd_frame_id = cx.frame_id;
    let demo_dnd_scope = fret_ui_kit::dnd::DndScopeId(cx.root_id().0);
    let demo_dnd_service = fret_ui_kit::dnd::dnd_service_model(cx);

    let demo_dnd_long_press_pointer = cx.with_state(CarouselPageState::default, |st| {
        st.demo_dnd_long_press_pointer.clone()
    });
    let demo_dnd_long_press_pointer = match demo_dnd_long_press_pointer {
        Some(model) => model,
        None => {
            let model: Model<Option<fret_core::PointerId>> = cx.app.models_mut().insert(None);
            cx.with_state(CarouselPageState::default, |st| {
                st.demo_dnd_long_press_pointer = Some(model.clone());
            });
            model
        }
    };

    let demo_slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        let theme = Theme::global(&*cx.app).clone();

        let number = ui::text(cx, format!("{idx}"))
            .text_size_px(visual.text_px)
            .line_height_px(visual.line_height_px)
            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
            .font_semibold()
            .into_element(cx);

        let base = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, LayoutRefinement::default().size_full()),
                direction: fret_core::Axis::Horizontal,
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                padding: Edges::all(Px(24.0)).into(),
                ..Default::default()
            },
            move |_cx| vec![number],
        );

        let mut layered: Vec<AnyElement> = vec![base];

        if idx == 1 {
            let frame_id = demo_dnd_frame_id;
            let scope = demo_dnd_scope;

            let on_down_handle_pointer = demo_dnd_pointer.clone();
            let on_down_handle_dragging = demo_dnd_dragging.clone();
            let on_down_dnd_service = demo_dnd_service.clone();
            let on_down: fret_ui::action::OnPointerDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      down: fret_ui::action::PointerDownCx| {
                    if down.button != fret_core::MouseButton::Left {
                        return false;
                    }

                    host.capture_pointer();
                    let _ = host.models_mut().update(&on_down_handle_pointer, |v| {
                        *v = Some(down.pointer_id);
                    });
                    let _ = host
                        .models_mut()
                        .update(&on_down_handle_dragging, |v| *v = false);

                    let _ = fret_ui_kit::dnd::handle_pointer_down_in_scope(
                        host.models_mut(),
                        &on_down_dnd_service,
                        action_cx.window,
                        frame_id,
                        DEMO_DND_KIND,
                        scope,
                        down.pointer_id,
                        down.position,
                        down.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let on_move_handle_pointer = demo_dnd_pointer.clone();
            let on_move_handle_dragging = demo_dnd_dragging.clone();
            let on_move_dnd_service = demo_dnd_service.clone();
            let on_move: fret_ui::action::OnPointerMove = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      mv: fret_ui::action::PointerMoveCx| {
                    let tracked = host
                        .models_mut()
                        .read(&on_move_handle_pointer, |v| *v)
                        .ok()
                        .flatten()
                        .is_some_and(|id| id == mv.pointer_id);
                    if !tracked {
                        return false;
                    }

                    let update = fret_ui_kit::dnd::handle_pointer_move_in_scope(
                        host.models_mut(),
                        &on_move_dnd_service,
                        action_cx.window,
                        frame_id,
                        DEMO_DND_KIND,
                        scope,
                        mv.pointer_id,
                        mv.position,
                        mv.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    if matches!(
                        update.sensor,
                        fret_ui_kit::dnd::SensorOutput::DragStart { .. }
                            | fret_ui_kit::dnd::SensorOutput::DragMove { .. }
                    ) {
                        let _ =
                            host.models_mut()
                                .update(&on_move_handle_dragging, |v| *v = true);
                    }

                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let on_up_handle_pointer = demo_dnd_pointer.clone();
            let on_up_handle_dragging = demo_dnd_dragging.clone();
            let on_up_dnd_service = demo_dnd_service.clone();
            let on_up: fret_ui::action::OnPointerUp = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      up: fret_ui::action::PointerUpCx| {
                    let tracked = host
                        .models_mut()
                        .read(&on_up_handle_pointer, |v| *v)
                        .ok()
                        .flatten()
                        .is_some_and(|id| id == up.pointer_id);
                    if !tracked {
                        return false;
                    }

                    let _ = fret_ui_kit::dnd::handle_pointer_up_in_scope(
                        host.models_mut(),
                        &on_up_dnd_service,
                        action_cx.window,
                        frame_id,
                        DEMO_DND_KIND,
                        scope,
                        up.pointer_id,
                        up.position,
                        up.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&on_up_handle_pointer, |v| *v = None);
                    let _ =
                        host.models_mut()
                            .update(&on_up_handle_dragging, |v| *v = false);
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let on_cancel_handle_pointer = demo_dnd_pointer.clone();
            let on_cancel_handle_dragging = demo_dnd_dragging.clone();
            let on_cancel_dnd_service = demo_dnd_service.clone();
            let on_cancel: fret_ui::action::OnPointerCancel = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      cancel: fret_ui::action::PointerCancelCx| {
                    let tracked = host
                        .models_mut()
                        .read(&on_cancel_handle_pointer, |v| *v)
                        .ok()
                        .flatten()
                        .is_some_and(|id| id == cancel.pointer_id);
                    if !tracked {
                        return false;
                    }

                    let position = cancel.position.unwrap_or_else(|| host.bounds().origin);
                    let _ = fret_ui_kit::dnd::handle_pointer_cancel_in_scope(
                        host.models_mut(),
                        &on_cancel_dnd_service,
                        action_cx.window,
                        frame_id,
                        DEMO_DND_KIND,
                        scope,
                        cancel.pointer_id,
                        position,
                        cancel.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.release_pointer_capture();
                    let _ =
                        host.models_mut()
                            .update(&on_cancel_handle_pointer, |v| *v = None);
                    let _ =
                        host.models_mut()
                            .update(&on_cancel_handle_dragging, |v| *v = false);
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let mut props = fret_ui::element::PointerRegionProps::default();
            props.layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .absolute()
                    .top(Space::N2)
                    .right(Space::N2)
                    .w_px(Px(28.0))
                    .h_px(Px(28.0)),
            );

            let handle = cx
                .pointer_region(props, move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    cx.pointer_region_on_pointer_cancel(on_cancel);
                    Vec::new()
                })
                .test_id("ui-gallery-carousel-demo-dnd-handle");

            layered.push(handle);

            // Touch-friendly long-press DnD region. We gate this via a delay+distance activation
            // constraint and keep it visually simple so it is easy to target in diag scripts.
            let long_press_frame_id = demo_dnd_frame_id;
            let long_press_scope = demo_dnd_scope;

            let on_long_press_down_pointer = demo_dnd_long_press_pointer.clone();
            let on_long_press_down_dragging = demo_dnd_dragging.clone();
            let on_long_press_down_service = demo_dnd_service.clone();
            let on_long_press_down: fret_ui::action::OnPointerDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      down: fret_ui::action::PointerDownCx| {
                    if down.button != fret_core::MouseButton::Left {
                        return false;
                    }

                    host.capture_pointer();
                    let _ = host.models_mut().update(&on_long_press_down_pointer, |v| {
                        *v = Some(down.pointer_id);
                    });
                    let _ = host
                        .models_mut()
                        .update(&on_long_press_down_dragging, |v| *v = false);

                    let _ = fret_ui_kit::dnd::handle_pointer_down_in_scope(
                        host.models_mut(),
                        &on_long_press_down_service,
                        action_cx.window,
                        long_press_frame_id,
                        DEMO_DND_KIND,
                        long_press_scope,
                        down.pointer_id,
                        down.position,
                        down.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::DelayAndDistance {
                            ticks: 12,
                            px: 6.0,
                        },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let on_long_press_move_pointer = demo_dnd_long_press_pointer.clone();
            let on_long_press_move_dragging = demo_dnd_dragging.clone();
            let on_long_press_move_service = demo_dnd_service.clone();
            let on_long_press_move: fret_ui::action::OnPointerMove = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      mv: fret_ui::action::PointerMoveCx| {
                    let tracked = host
                        .models_mut()
                        .read(&on_long_press_move_pointer, |v| *v)
                        .ok()
                        .flatten()
                        .is_some_and(|id| id == mv.pointer_id);
                    if !tracked {
                        return false;
                    }

                    let update = fret_ui_kit::dnd::handle_pointer_move_in_scope(
                        host.models_mut(),
                        &on_long_press_move_service,
                        action_cx.window,
                        long_press_frame_id,
                        DEMO_DND_KIND,
                        long_press_scope,
                        mv.pointer_id,
                        mv.position,
                        mv.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::DelayAndDistance {
                            ticks: 12,
                            px: 6.0,
                        },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    if matches!(
                        update.sensor,
                        fret_ui_kit::dnd::SensorOutput::DragStart { .. }
                            | fret_ui_kit::dnd::SensorOutput::DragMove { .. }
                    ) {
                        let _ =
                            host.models_mut()
                                .update(&on_long_press_move_dragging, |v| *v = true);
                    }

                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let on_long_press_up_pointer = demo_dnd_long_press_pointer.clone();
            let on_long_press_up_dragging = demo_dnd_dragging.clone();
            let on_long_press_up_service = demo_dnd_service.clone();
            let on_long_press_up: fret_ui::action::OnPointerUp = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      up: fret_ui::action::PointerUpCx| {
                    let tracked = host
                        .models_mut()
                        .read(&on_long_press_up_pointer, |v| *v)
                        .ok()
                        .flatten()
                        .is_some_and(|id| id == up.pointer_id);
                    if !tracked {
                        return false;
                    }

                    let _ = fret_ui_kit::dnd::handle_pointer_up_in_scope(
                        host.models_mut(),
                        &on_long_press_up_service,
                        action_cx.window,
                        long_press_frame_id,
                        DEMO_DND_KIND,
                        long_press_scope,
                        up.pointer_id,
                        up.position,
                        up.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::DelayAndDistance {
                            ticks: 12,
                            px: 6.0,
                        },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.release_pointer_capture();
                    let _ =
                        host.models_mut()
                            .update(&on_long_press_up_pointer, |v| *v = None);
                    let _ =
                        host.models_mut()
                            .update(&on_long_press_up_dragging, |v| *v = false);
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let on_long_press_cancel_pointer = demo_dnd_long_press_pointer.clone();
            let on_long_press_cancel_dragging = demo_dnd_dragging.clone();
            let on_long_press_cancel_service = demo_dnd_service.clone();
            let on_long_press_cancel: fret_ui::action::OnPointerCancel = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      cancel: fret_ui::action::PointerCancelCx| {
                    let tracked = host
                        .models_mut()
                        .read(&on_long_press_cancel_pointer, |v| *v)
                        .ok()
                        .flatten()
                        .is_some_and(|id| id == cancel.pointer_id);
                    if !tracked {
                        return false;
                    }

                    let position = cancel.position.unwrap_or_else(|| host.bounds().origin);
                    let _ = fret_ui_kit::dnd::handle_pointer_cancel_in_scope(
                        host.models_mut(),
                        &on_long_press_cancel_service,
                        action_cx.window,
                        long_press_frame_id,
                        DEMO_DND_KIND,
                        long_press_scope,
                        cancel.pointer_id,
                        position,
                        cancel.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::DelayAndDistance {
                            ticks: 12,
                            px: 6.0,
                        },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.release_pointer_capture();
                    let _ = host
                        .models_mut()
                        .update(&on_long_press_cancel_pointer, |v| *v = None);
                    let _ =
                        host.models_mut()
                            .update(&on_long_press_cancel_dragging, |v| *v = false);
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let mut long_press_props = fret_ui::element::PointerRegionProps::default();
            long_press_props.layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .absolute()
                    .bottom(Space::N2)
                    .left(Space::N2)
                    .w_px(Px(96.0))
                    .h_px(Px(28.0)),
            );

            let long_press = cx
                .pointer_region(long_press_props, move |cx| {
                    cx.pointer_region_on_pointer_down(on_long_press_down);
                    cx.pointer_region_on_pointer_move(on_long_press_move);
                    cx.pointer_region_on_pointer_up(on_long_press_up);
                    cx.pointer_region_on_pointer_cancel(on_long_press_cancel);
                    Vec::new()
                })
                .test_id("ui-gallery-carousel-demo-dnd-long-press");
            layered.push(long_press);

            layered.push(
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::IconSm)
                    .refine_layout(
                        LayoutRefinement::default()
                            .absolute()
                            .bottom(Space::N2)
                            .right(Space::N2),
                    )
                    .on_activate(toggle_demo_inner_clicked.clone())
                    .test_id("ui-gallery-carousel-demo-inner-button")
                    .into_element(cx),
            );

            if demo_inner_clicked_now {
                let props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default(),
                    LayoutRefinement::default()
                        .absolute()
                        .top(Space::N2)
                        .left(Space::N2)
                        .w_px(Px(1.0))
                        .h_px(Px(1.0)),
                );
                layered.push(
                    cx.container(props, |_cx| Vec::new()).attach_semantics(
                        SemanticsDecoration::default()
                            .role(fret_core::SemanticsRole::Group)
                            .test_id("ui-gallery-carousel-demo-inner-clicked"),
                    ),
                );
            }

            if demo_dnd_dragging_now {
                let props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default(),
                    LayoutRefinement::default()
                        .absolute()
                        .top(Space::N2)
                        .left(Space::N6)
                        .w_px(Px(1.0))
                        .h_px(Px(1.0)),
                );
                layered.push(
                    cx.container(props, |_cx| Vec::new())
                        .test_id("ui-gallery-carousel-demo-dnd-active"),
                );
            }
        }

        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().relative().w_full().aspect_ratio(1.0),
        );
        let content = cx.container(props, move |_cx| layered);

        let card = shadcn::Card::new([content]).into_element(cx);
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    };

    let demo_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let demo_items = (1..=5)
        .map(|idx| demo_slide(cx, idx, demo_visual))
        .collect::<Vec<_>>();
    let demo = shadcn::Carousel::new(demo_items)
        // Web goldens: track width accounts for the negative start margin (`-ml-4`).
        .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-demo")
        .into_element(cx);

    let basic_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let basic_items = (1..=5)
        .map(|idx| slide(cx, idx, basic_visual))
        .collect::<Vec<_>>();
    let basic = shadcn::Carousel::new(basic_items)
        .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-basic")
        .into_element(cx);

    let align_start_visual = SlideVisual {
        text_px: Px(30.0),
        line_height_px: Px(36.0),
    };
    let align_start_items = (1..=5)
        .map(|idx| slide(cx, idx, align_start_visual))
        .collect::<Vec<_>>();
    let sizes = shadcn::Carousel::new(align_start_items)
        .opts(shadcn::CarouselOptions::new().align(shadcn::CarouselAlign::Start))
        // Approximate the `lg:basis-1/3` docs example deterministically (see web-vs-fret harness).
        .item_basis_main_px(Px(133.328))
        .refine_track_layout(LayoutRefinement::default().w_px(Px(400.0)))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-sizes")
        .into_element(cx);

    let spacing_visual = SlideVisual {
        text_px: Px(24.0),
        line_height_px: Px(32.0),
    };
    let spacing_items = (1..=5)
        .map(|idx| slide(cx, idx, spacing_visual))
        .collect::<Vec<_>>();
    let spacing = shadcn::Carousel::new(spacing_items)
        .item_basis_main_px(Px(129.328))
        .refine_track_layout(LayoutRefinement::default().w_px(Px(388.0)))
        .track_start_neg_margin(Space::N1)
        .item_padding_start(Space::N1)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-spacing")
        .into_element(cx);

    // API: expose a small snapshot surface so demos can render slide counters without wiring an
    // Embla-like imperative API.
    let api_snapshot = cx.with_state(CarouselPageState::default, |st| st.api_snapshot.clone());
    let api_snapshot = match api_snapshot {
        Some(model) => model,
        None => {
            let model: Model<shadcn::CarouselApiSnapshot> =
                cx.app.models_mut().insert(shadcn::CarouselApiSnapshot::default());
            cx.with_state(CarouselPageState::default, |st| {
                st.api_snapshot = Some(model.clone());
            });
            model
        }
    };
    let api_snapshot_now = cx
        .watch_model(&api_snapshot)
        .copied()
        .unwrap_or_default();

    let api_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let api_items = (1..=5)
        .map(|idx| slide_unwrapped(cx, idx, api_visual))
        .collect::<Vec<_>>();
    let api_carousel = shadcn::Carousel::new(api_items)
        .api_snapshot_model(api_snapshot.clone())
        .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
        .refine_layout(
            LayoutRefinement::default().w_full().max_w(max_w_xs),
        )
        .test_id("ui-gallery-carousel-api")
        .into_element(cx);

    let api_counter_text = if api_snapshot_now.snap_count > 0 {
        format!(
            "Slide {} of {}",
            api_snapshot_now.selected_index.saturating_add(1),
            api_snapshot_now.snap_count
        )
    } else {
        "Slide 0 of 0".to_string()
    };
    let api_counter = {
        let theme = Theme::global(&*cx.app);
        let style = fret_ui_kit::typography::control_text_style(
            theme,
            fret_ui_kit::typography::UiTextSize::Sm,
        );
        let color = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));

        let text = cx.text_props(TextProps {
            layout: {
                let mut layout = fret_ui::element::LayoutStyle::default();
                layout.size.width = fret_ui::element::Length::Fill;
                layout
            },
            text: Arc::from(api_counter_text),
            style: Some(style),
            color: Some(color),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Center,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        ui::container(cx, move |_cx| vec![text]).py_2().into_element(cx)
    };

    let api = cx.flex(
        FlexProps {
            layout: decl_style::layout_style(
                &Theme::global(&*cx.app).snapshot(),
                LayoutRefinement::default()
                    .w_full()
                    .max_w(max_w_xs)
                    .mx_auto(),
            ),
            direction: fret_core::Axis::Vertical,
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            ..Default::default()
        },
        move |_cx| vec![api_carousel, api_counter],
    );

    // Plugin (autoplay): matches shadcn/ui `carousel-plugin` (Embla autoplay plugin outcome).
    let plugin_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let plugin_items = (1..=5)
        .map(|idx| slide(cx, idx, plugin_visual))
        .collect::<Vec<_>>();
    let plugin = shadcn::Carousel::new(plugin_items)
        .autoplay(shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000)))
        .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-plugin")
        .into_element(cx);

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Carousel demo: Basic, Sizes, and Spacing.",
            "The upstream demo uses responsive item widths (`md:basis-1/2` / `lg:basis-1/3`). Fret uses a fixed `item_basis_main_px` to keep geometry deterministic in native builds.",
            "Spacing parity depends on pairing `track_start_neg_margin` with `item_padding_start`.",
        ],
    );

    // Expandable: used by the motion pilot suite to exercise content-driven resizing while the
    // carousel remains interactive.
    let expandable_selected =
        cx.with_state(CarouselPageState::default, |st| st.expandable_selected.clone());
    let expandable_selected = match expandable_selected {
        Some(model) => model,
        None => {
            let model: Model<Option<usize>> = cx.app.models_mut().insert(None);
            cx.with_state(CarouselPageState::default, |st| {
                st.expandable_selected = Some(model.clone());
            });
            model
        }
    };
    let expandable_selected_now = cx
        .watch_model(&expandable_selected)
        .copied()
        .unwrap_or(None);

    let set_expandable_selected = |next: Option<usize>| {
        let expandable_selected = expandable_selected.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                let next = next;
                let _ = host
                    .models_mut()
                    .update(&expandable_selected, |cur| *cur = next);
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let expandable_items = (1..=5)
        .map(|idx| {
            let expanded = expandable_selected_now == Some(idx);
            let height = if expanded { Px(260.0) } else { Px(140.0) };

            let theme = Theme::global(&*cx.app).clone();
            let gap = decl_style::space(&theme, Space::N2);

            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().h_px(height),
                    ),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    gap: gap.into(),
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |cx| {
                    let mut out = vec![
                        ui::text(cx, format!("Item\u{00A0}{idx}"))
                            .text_base()
                            .font_semibold()
                            .nowrap()
                            .into_element(cx),
                        shadcn::Button::new(if expanded { "Collapse" } else { "Expand" })
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_activate(set_expandable_selected(Some(idx)))
                            .into_element(cx),
                    ];

                    if expanded {
                        out.push(ui::text(cx, "Expandable body").text_sm().into_element(cx));
                    }

                    out
                },
            );

            let card = shadcn::Card::new([body]).into_element(cx);
            ui::container(cx, move |_cx| vec![card])
                .p_1()
                .into_element(cx)
        })
        .collect::<Vec<_>>();

    let expandable = shadcn::Carousel::new(expandable_items)
        .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-expandable")
        .into_element(cx);

    // Orientation (vertical): aligns with upstream docs, and is used by the existing screenshot
    // diag script.
    let vertical_items = (1..=5)
        .map(|idx| {
            let theme = Theme::global(&*cx.app).clone();
            let number = ui::text(cx, format!("{idx}"))
                .text_size_px(Px(30.0))
                .line_height_px(Px(36.0))
                .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
                .font_semibold()
                .into_element(cx);

            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                    direction: fret_core::Axis::Horizontal,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |_cx| vec![number],
            );

            let card = shadcn::Card::new([body]).into_element(cx);
            ui::container(cx, move |_cx| vec![card])
                .p_1()
                .into_element(cx)
        })
        .collect::<Vec<_>>();

    let orientation_vertical = shadcn::Carousel::new(vertical_items)
        .orientation(shadcn::CarouselOrientation::Vertical)
        .opts(shadcn::CarouselOptions::new().align(shadcn::CarouselAlign::Start))
        // Match the shadcn/ui docs outcome on desktop widths (`md:basis-1/2`) in a deterministic
        // way (we do not currently apply breakpoint-aware per-item sizing here).
        .item_basis_main_px(Px(100.0))
        .refine_viewport_layout(LayoutRefinement::default().h_px(Px(196.0)))
        .refine_track_layout(LayoutRefinement::default().h_px(Px(200.0)))
        .track_start_neg_margin(Space::N1)
        .item_padding_start(Space::N1)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-orientation-vertical")
        .into_element(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Carousel demo cards (Fret builder API; not Embla)."),
        vec![
            DocSection::new("Demo", demo)
                .description("A carousel with 5 items and previous/next buttons.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-demo"),
            DocSection::new("Basic", basic)
                .description("Default slide width (basis-full).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-basic")
                .code(
                    "rust",
                    r#"let items = (1..=5).map(|idx| slide(cx, idx)).collect::<Vec<_>>();

shadcn::Carousel::new(items)
    .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Sizes", sizes)
                .description("Three active items (`basis-1/3`) to mirror the docs layout.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-sizes")
                .code(
                    "rust",
                    r#"// Upstream: responsive widths (`md:basis-1/2` / `lg:basis-1/3`).
// Here: fixed basis for deterministic native layout.
shadcn::Carousel::new(items)
    .opts(shadcn::CarouselOptions::new().align(shadcn::CarouselAlign::Start))
    .item_basis_main_px(Px(133.328)) // approx `basis-1/3` in docs/web goldens
    .refine_track_layout(LayoutRefinement::default().w_px(Px(400.0)))
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Spacing", spacing)
                .description(
                    "Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`).",
                )
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-spacing")
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .item_basis_main_px(Px(129.328))
    .refine_track_layout(LayoutRefinement::default().w_px(Px(388.0)))
    .track_start_neg_margin(Space::N1)
    .item_padding_start(Space::N1)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("API", api)
                .description("A carousel with a slide counter (shadcn `setApi`-style outcome).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-api")
                .code(
                    "rust",
                    r#"let api = cx.app.models_mut().insert(shadcn::CarouselApiSnapshot::default());
let api_now = cx.watch_model(&api).copied().unwrap_or_default();

// Upstream `carousel-api` does not wrap each card in `p-1`.
let carousel = shadcn::Carousel::new(items_without_p1)
    .api_snapshot_model(api.clone())
    .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx);

let counter = ui::text(
    cx,
    format!(
        \"Slide {} of {}\",
        api_now.selected_index + 1,
        api_now.snap_count
    ),
)
 .text_sm()
 .into_element(cx);"#,
                ),
            DocSection::new("Plugin (Autoplay)", plugin)
                .description("Autoplay: 2000ms delay; hover pauses; interaction stops.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-plugin")
                .code(
                    "rust",
                    r#"use std::time::Duration;

let items = (1..=5).map(|idx| slide(cx, idx)).collect::<Vec<_>>();

shadcn::Carousel::new(items)
    .autoplay(shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000)))
    .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Expandable", expandable)
                .description("Content-driven height changes (used by the motion pilot suite).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-expandable"),
            DocSection::new("Orientation (Vertical)", orientation_vertical)
                .description("A vertical carousel (orientation=\"vertical\").")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-orientation-vertical")
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .orientation(shadcn::CarouselOrientation::Vertical)
    .opts(shadcn::CarouselOptions::new().align(shadcn::CarouselAlign::Start))
    .refine_viewport_layout(LayoutRefinement::default().h_px(Px(196.0)))
    .refine_track_layout(LayoutRefinement::default().h_px(Px(200.0)))
    .track_start_neg_margin(Space::N1)
    .item_padding_start(Space::N1)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(760.0)),
        ],
    );

    vec![body.test_id("ui-gallery-carousel-component")]
    })
}
