pub const SOURCE: &str = include_str!("media.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Share Project")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-media-trigger"),
            )),
            shadcn::AlertDialogPart::content_with(|cx| {
                let icon = shadcn::raw::icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.circle-fading-plus"),
                    Some(Px(32.0)),
                    None,
                );
                let media = shadcn::AlertDialogMedia::new(icon).into_element(cx);

                shadcn::AlertDialogContent::new([])
                .test_id("ui-gallery-alert-dialog-media-content")
                .with_children(cx, |cx| {
                    vec![
                        shadcn::AlertDialogHeader::new([])
                            .media(media)
                            .with_children(cx, |cx| {
                                vec![
                                    shadcn::AlertDialogTitle::new("Share this project?")
                                        .into_element(cx),
                                    shadcn::AlertDialogDescription::new(
                                        "Anyone with the link will be able to view and edit this project.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                        shadcn::AlertDialogFooter::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::AlertDialogCancel::from_scope("Cancel")
                                    .test_id("ui-gallery-alert-dialog-media-cancel")
                                    .into_element(cx),
                                shadcn::AlertDialogAction::from_scope("Share")
                                    .test_id("ui-gallery-alert-dialog-media-action")
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
