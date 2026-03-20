pub const SOURCE: &str = include_str!("media.rs");

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
                shadcn::Button::new("Share Project")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-media-trigger"),
            )),
            shadcn::AlertDialogPart::content(
                shadcn::AlertDialogContent::build(|cx, out| {
                    let icon = shadcn::raw::icon::icon_with(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-fading-plus"),
                        Some(Px(32.0)),
                        None,
                    );
                    let media = shadcn::AlertDialogMedia::new(icon).into_element(cx);

                    out.push_ui(
                        cx,
                        shadcn::AlertDialogHeader::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogTitle::new("Share this project?"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogDescription::new(
                                    "Anyone with the link will be able to view and edit this project.",
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
                                shadcn::AlertDialogCancel::from_scope("Cancel")
                                    .test_id("ui-gallery-alert-dialog-media-cancel"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogAction::from_scope("Share")
                                    .test_id("ui-gallery-alert-dialog-media-action"),
                            );
                        }),
                    );
                })
                .test_id("ui-gallery-alert-dialog-media-content"),
            ),
        ])
        .into_element(cx)
}
// endregion: example
