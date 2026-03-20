pub const SOURCE: &str = include_str!("small_with_media.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Show Dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-small-media-trigger"),
            )),
            shadcn::AlertDialogPart::content(
                shadcn::AlertDialogContent::build(|cx, out| {
                    let icon = shadcn::raw::icon::icon_with(
                        cx,
                        fret_icons::IconId::new_static("lucide.bluetooth"),
                        Some(Px(32.0)),
                        None,
                    );
                    let media = shadcn::AlertDialogMedia::new(icon).into_element(cx);

                    out.push_ui(
                        cx,
                        shadcn::AlertDialogHeader::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogTitle::new("Allow accessory to connect?"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogDescription::new(
                                    "Do you want to allow the USB accessory to connect to this device?",
                                ),
                            );
                        })
                        .media(media),
                    );
                    out.push_ui(
                        cx,
                        shadcn::AlertDialogFooter::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogCancel::from_scope("Don't allow")
                                    .test_id("ui-gallery-alert-dialog-small-media-cancel"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogAction::from_scope("Allow")
                                    .test_id("ui-gallery-alert-dialog-small-media-action"),
                            );
                        }),
                    );
                })
                .size(shadcn::AlertDialogContentSize::Sm)
                .test_id("ui-gallery-alert-dialog-small-media-content"),
            ),
        ])
        .into_element(cx)
}
// endregion: example
