pub const SOURCE: &str = include_str!("custom_close_button.rs");

// region: example
use fret::children::UiElementSinkExt;
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
            let input = shadcn::Input::new(link_model.clone())
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx);
            shadcn::DialogContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::DialogHeader::build(|cx, out| {
                        out.push_ui(cx, shadcn::DialogTitle::new("Share link"));
                        out.push_ui(
                            cx,
                            shadcn::DialogDescription::new(
                                "Replace the close affordance with a custom footer action.",
                            ),
                        );
                    }),
                );
                out.push(input);
                out.push_ui(
                    cx,
                    shadcn::DialogFooter::build(|cx, out| {
                        let close = shadcn::DialogClose::from_scope().build(
                            cx,
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-dialog-custom-close-footer"),
                        );
                        out.push(close);
                    }),
                );
            })
            .show_close_button(false)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
            .into_element(cx)
            .test_id("ui-gallery-dialog-custom-close-content")
        },
    )
}
// endregion: example
