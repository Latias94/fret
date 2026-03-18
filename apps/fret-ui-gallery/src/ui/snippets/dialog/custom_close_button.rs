pub const SOURCE: &str = include_str!("custom_close_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let share_link = cx.local_model_keyed("share_link", || {
        String::from("https://ui.shadcn.com/docs/components/dialog")
    });

    let open_for_trigger = open.clone();
    let link_model = share_link.clone();

    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Share")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-custom-close-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Share link").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Replace the close affordance with a custom footer action.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Input::new(link_model.clone())
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                shadcn::DialogFooter::new([shadcn::DialogClose::from_scope().build(
                    cx,
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .test_id("ui-gallery-dialog-custom-close-footer"),
                )])
                .into_element(cx),
            ])
            .show_close_button(false)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
            .into_element(cx)
            .test_id("ui-gallery-dialog-custom-close-content")
        },
    )
}
// endregion: example
