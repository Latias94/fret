pub const SOURCE: &str = include_str!("small_with_media.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Px, TextOverflow, TextWrap};
use fret_ui_shadcn::{facade as shadcn, prelude::ui};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Show Dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-small-media-trigger"),
            )),
            shadcn::AlertDialogPart::content_with(|cx| {
                let icon = shadcn::raw::icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.bluetooth"),
                    Some(Px(32.0)),
                    None,
                );
                let media = shadcn::AlertDialogMedia::new(icon).into_element(cx);

                shadcn::AlertDialogContent::new([])
                .size(shadcn::AlertDialogContentSize::Sm)
                .test_id("ui-gallery-alert-dialog-small-media-content")
                .with_children(cx, |cx| {
                    vec![
                        shadcn::AlertDialogHeader::new([])
                            .media(media)
                            .with_children(cx, |cx| {
                                vec![
                                    shadcn::AlertDialogTitle::new_children([ui::text(
                                        "Allow accessory to connect?",
                                    )
                                    .wrap(TextWrap::Word)
                                    .overflow(TextOverflow::Clip)
                                    .test_id("ui-gallery-alert-dialog-small-media-title")
                                    .into_element(cx)])
                                    .into_element(cx),
                                    shadcn::AlertDialogDescription::new_children([ui::text(
                                        "Do you want to allow the USB accessory to connect to this device?",
                                    )
                                    .wrap(TextWrap::Word)
                                    .overflow(TextOverflow::Clip)
                                    .test_id("ui-gallery-alert-dialog-small-media-description")
                                    .into_element(cx)])
                                    .into_element(cx),
                                ]
                            }),
                        shadcn::AlertDialogFooter::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::AlertDialogCancel::from_scope("Don't allow")
                                    .test_id("ui-gallery-alert-dialog-small-media-cancel")
                                    .into_element(cx),
                                shadcn::AlertDialogAction::from_scope("Allow")
                                    .test_id("ui-gallery-alert-dialog-small-media-action")
                                    .into_element(cx),
                            ]
                        }),
                    ]
                })
            }),
        ])
        .into_element(cx)
}
// endregion: example
