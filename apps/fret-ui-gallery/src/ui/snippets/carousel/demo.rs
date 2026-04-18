pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign, SemanticsDecoration};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    // Match shadcn/ui v4 docs example widths (`max-w-xs`) deterministically in native builds.
    let max_w_xs = Px(320.0);

    // Demo: include a descendant pressable so diag scripts can gate pointer propagation
    // (drag-from-descendant should not activate; click should).
    let demo_inner_clicked = cx.local_model_keyed("demo_inner_clicked", || false);
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
                let _ = host.models_mut().update(&demo_inner_clicked, |v| *v = !*v);
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    // Demo DnD handle (MVP): show a handle-only DnD activation that does not compete with the
    // carousel swipe gesture.
    const DEMO_DND_KIND: fret_runtime::DragKindId = fret_runtime::DragKindId(101);
    let demo_dnd_pointer =
        cx.local_model_keyed("demo_dnd_pointer", || None::<fret_core::PointerId>);
    let demo_dnd_dragging = cx.local_model_keyed("demo_dnd_dragging", || false);
    let demo_dnd_dragging_now = cx.watch_model(&demo_dnd_dragging).copied().unwrap_or(false);
    let demo_dnd_frame_id = cx.frame_id;
    let demo_dnd_scope = fret_ui_kit::dnd::DndScopeId(cx.root_id().0);
    let demo_dnd_service = fret_ui_kit::dnd::dnd_service_model(cx);

    let demo_dnd_long_press_pointer =
        cx.local_model_keyed(
            "demo_dnd_long_press_pointer",
            || None::<fret_core::PointerId>,
        );

    let demo_slide = |cx: &mut AppComponentCx<'_>, idx: usize, visual: SlideVisual| {
        let theme = Theme::global(&*cx.app).clone();

        let number = ui::text(format!("{idx}"))
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
            let build_demo_dnd_handlers =
                |pointer_model: fret_runtime::Model<Option<fret_core::PointerId>>,
                 activation_constraint: fret_ui_kit::dnd::ActivationConstraint| {
                    let on_update_dragging = demo_dnd_dragging.clone();
                    let on_update = Arc::new(
                        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              update: &fret_ui_kit::dnd::DndUpdate| {
                            if matches!(
                                update.sensor,
                                fret_ui_kit::dnd::SensorOutput::DragStart { .. }
                                    | fret_ui_kit::dnd::SensorOutput::DragMove { .. }
                            ) {
                                let _ =
                                    host.models_mut().update(&on_update_dragging, |v| *v = true);
                                host.request_redraw(action_cx.window);
                            }
                        },
                    );

                    let forwarders = fret_ui_kit::dnd::DndPointerForwarders::new(
                        demo_dnd_service.clone(),
                        frame_id,
                        fret_ui_kit::dnd::DndPointerForwardersConfig::for_kind(DEMO_DND_KIND)
                            .scope(scope)
                            .activation_constraint(activation_constraint)
                            .collision_strategy(fret_ui_kit::dnd::CollisionStrategy::ClosestCenter)
                            .on_update(on_update),
                    );

                    let down_forwarder = forwarders.on_pointer_down();
                    let move_forwarder = forwarders.on_pointer_move();
                    let up_forwarder = forwarders.on_pointer_up();
                    let cancel_forwarder = forwarders.on_pointer_cancel();

                    let on_down_pointer = pointer_model.clone();
                    let on_down_dragging = demo_dnd_dragging.clone();
                    let on_down: fret_ui::action::OnPointerDown = Arc::new(
                        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              down: fret_ui::action::PointerDownCx| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            let window = action_cx.window;
                            let _ = host.models_mut().update(&on_down_pointer, |v| {
                                *v = Some(down.pointer_id);
                            });
                            let _ = host.models_mut().update(&on_down_dragging, |v| *v = false);
                            let handled = down_forwarder(host, action_cx, down);
                            host.request_redraw(window);
                            handled
                        },
                    );

                    let on_move_pointer = pointer_model.clone();
                    let on_move: fret_ui::action::OnPointerMove = Arc::new(
                        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              mv: fret_ui::action::PointerMoveCx| {
                            let tracked = host
                                .models_mut()
                                .read(&on_move_pointer, |v| *v)
                                .ok()
                                .flatten()
                                .is_some_and(|id| id == mv.pointer_id);
                            if !tracked {
                                return false;
                            }

                            move_forwarder(host, action_cx, mv)
                        },
                    );

                    let on_up_pointer = pointer_model.clone();
                    let on_up_dragging = demo_dnd_dragging.clone();
                    let on_up: fret_ui::action::OnPointerUp = Arc::new(
                        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              up: fret_ui::action::PointerUpCx| {
                            let tracked = host
                                .models_mut()
                                .read(&on_up_pointer, |v| *v)
                                .ok()
                                .flatten()
                                .is_some_and(|id| id == up.pointer_id);
                            if !tracked {
                                return false;
                            }

                            let window = action_cx.window;
                            let handled = up_forwarder(host, action_cx, up);
                            let _ = host.models_mut().update(&on_up_pointer, |v| *v = None);
                            let _ = host.models_mut().update(&on_up_dragging, |v| *v = false);
                            host.request_redraw(window);
                            handled
                        },
                    );

                    let on_cancel_pointer = pointer_model.clone();
                    let on_cancel_dragging = demo_dnd_dragging.clone();
                    let on_cancel: fret_ui::action::OnPointerCancel = Arc::new(
                        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              cancel: fret_ui::action::PointerCancelCx| {
                            let tracked = host
                                .models_mut()
                                .read(&on_cancel_pointer, |v| *v)
                                .ok()
                                .flatten()
                                .is_some_and(|id| id == cancel.pointer_id);
                            if !tracked {
                                return false;
                            }

                            let window = action_cx.window;
                            let handled = cancel_forwarder(host, action_cx, cancel);
                            let _ = host.models_mut().update(&on_cancel_pointer, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&on_cancel_dragging, |v| *v = false);
                            host.request_redraw(window);
                            handled
                        },
                    );

                    (on_down, on_move, on_up, on_cancel)
                };

            let (on_down, on_move, on_up, on_cancel) = build_demo_dnd_handlers(
                demo_dnd_pointer.clone(),
                fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
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
            let (on_long_press_down, on_long_press_move, on_long_press_up, on_long_press_cancel) =
                build_demo_dnd_handlers(
                    demo_dnd_long_press_pointer.clone(),
                    fret_ui_kit::dnd::ActivationConstraint::DelayAndDistance { ticks: 12, px: 6.0 },
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
            LayoutRefinement::default()
                .relative()
                .w_full()
                .aspect_ratio(1.0),
        );
        let content = cx.container(props, move |_cx| layered);

        let card = shadcn::card(
            |cx| ui::children![cx; shadcn::card_content(|cx| ui::children![cx; content])],
        )
        .into_element(cx);
        ui::container(move |_cx| vec![card])
            .w_full()
            .p_1()
            .into_element(cx)
    };

    let demo_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(demo_slide(cx, idx, demo_visual)))
        .collect::<Vec<_>>();

    shadcn::Carousel::new(items)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-demo")
        .into_element(cx)
}
// endregion: example
