pub const SOURCE: &str = include_str!("outside_press.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("outside_press_open", || false);
    let probe_activations = cx.local_model_keyed("outside_press_probe_activations", || 0u32);
    let probe_count = cx.watch_model(&probe_activations).copied().unwrap_or(0);

    ui::v_stack(|cx| {
        let open_for_drawer = open.clone();
        let probe_activations_for_button = probe_activations.clone();
        vec![
            shadcn::Drawer::new(open_for_drawer)
                .children([
                    shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                        shadcn::Button::new("Open modal drawer")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-drawer-outside-press-trigger"),
                    )),
                    shadcn::DrawerPart::content_with(move |cx| {
                        shadcn::DrawerContent::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::DrawerHeader::build(|cx, out| {
                                    out.push_ui(
                                        cx,
                                        shadcn::DrawerTitle::new("Dismiss on outside press"),
                                    );
                                    out.push_ui(
                                        cx,
                                        shadcn::DrawerDescription::new(
                                            "Click the underlay probe outside the panel. The drawer should close and restore focus to the trigger.",
                                        ),
                                    );
                                }),
                            );
                            out.push_ui(
                                cx,
                                ui::text(
                                    "The probe below exists only to make modal outside-press routing deterministic in the gallery and diag scripts.",
                                )
                                .text_sm(),
                            );
                            out.push_ui(
                                cx,
                                shadcn::DrawerFooter::build(|cx, out| {
                                    let close = shadcn::DrawerClose::from_scope().build(
                                        cx,
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("ui-gallery-drawer-outside-press-close"),
                                    );
                                    out.push(close);
                                }),
                            );
                        })
                        .into_element(cx)
                        .test_id("ui-gallery-drawer-outside-press-content")
                    }),
                ])
                .into_element(cx),
            shadcn::Button::new("Underlay focus probe")
                .variant(shadcn::ButtonVariant::Outline)
                .on_activate(cx.actions().listen(move |host, action_cx| {
                    let _ = host
                        .models_mut()
                        .update(&probe_activations_for_button, |value| *value += 1);
                    host.request_redraw(action_cx.window);
                }))
                .test_id("ui-gallery-drawer-outside-press-underlay-probe")
                .into_element(cx),
            ui::text(format!("Underlay activations: {probe_count}"))
                .text_sm()
                .into_element(cx)
                .test_id("ui-gallery-drawer-outside-press-underlay-status"),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
