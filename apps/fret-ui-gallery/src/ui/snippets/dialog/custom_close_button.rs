pub const SOURCE: &str = include_str!("custom_close_button.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let share_link = cx.local_model_keyed("share_link", || {
        String::from("https://ui.shadcn.com/docs/installation")
    });

    let link_model = share_link.clone();

    shadcn::Dialog::new(open.clone())
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("Share")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-custom-close-trigger"),
            )),
            shadcn::DialogPart::content_with(move |cx| {
                let input = shadcn::Input::new(link_model.clone())
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx);
                shadcn::DialogContent::new([])
                    .show_close_button(false)
                    .refine_layout(LayoutRefinement::default().max_w(Px(448.0)))
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogTitle::new("Share link").into_element(cx),
                                    shadcn::DialogDescription::new(
                                        "Anyone who has this link will be able to view this.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            input,
                            shadcn::DialogFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogClose::from_scope().build(
                                        cx,
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .test_id("ui-gallery-dialog-custom-close-footer"),
                                    ),
                                ]
                            }),
                        ]
                    })
                    .test_id("ui-gallery-dialog-custom-close-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example
