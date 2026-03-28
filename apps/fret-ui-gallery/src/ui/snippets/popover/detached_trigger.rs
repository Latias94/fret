pub const SOURCE: &str = include_str!("detached_trigger.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn hidden_internal_trigger(cx: &mut UiCx<'_>) -> AnyElement {
    ui::v_flex(|_cx| Vec::<AnyElement>::new())
        .layout(
            LayoutRefinement::default()
                .absolute()
                .w_px(Px(1.0))
                .h_px(Px(1.0)),
        )
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("detached_open", || false);

    let detached_trigger = shadcn::PopoverTrigger::new(
        shadcn::Button::new("Open From Toolbar")
            .variant(shadcn::ButtonVariant::Outline)
            .toggle_model(open.clone())
            .test_id("ui-gallery-popover-detached-trigger-button")
            .into_element(cx),
    )
    .auto_toggle(false)
    .into_element(cx);

    let detached_id = detached_trigger.id;
    let popover = shadcn::Popover::from_open(open.clone())
        .trigger_element(detached_id)
        .anchor_element(detached_id)
        .into_element_with(
            cx,
            hidden_internal_trigger,
            move |cx| {
                shadcn::PopoverContent::build(cx, |cx| {
                    ui::children![
                        cx;
                        shadcn::PopoverHeader::new(ui::children![
                            cx;
                            shadcn::PopoverTitle::new("Toolbar Actions"),
                            shadcn::PopoverDescription::new(
                                "The trigger lives outside the popover root, but focus and placement still follow that external button.",
                            )
                        ]),
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .toggle_model(open.clone())
                            .test_id("ui-gallery-popover-detached-trigger-close")
                    ]
                })
                .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                .test_id("ui-gallery-popover-detached-trigger-panel")
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-popover-detached-trigger-popover");

    ui::v_flex(|_cx| vec![detached_trigger, popover])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-popover-detached-trigger")
}
// endregion: example
