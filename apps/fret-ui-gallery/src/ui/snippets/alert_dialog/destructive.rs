pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Delete Chat")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .test_id("ui-gallery-alert-dialog-destructive-trigger"),
            )),
            shadcn::AlertDialogPart::content(shadcn::AlertDialogContent::build(|cx, out| {
                let icon = shadcn::raw::icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.trash-2"),
                    Some(Px(32.0)),
                    None,
                );

                out.push(
                    shadcn::AlertDialogHeader::new([
                        shadcn::AlertDialogTitle::new("Delete chat?").into_element(cx),
                        shadcn::AlertDialogDescription::new(
                            "This will permanently delete this chat conversation. View Settings to delete any memories saved during this chat.",
                        )
                        .into_element(cx),
                    ])
                    .media(shadcn::AlertDialogMedia::new(icon).into_element(cx))
                    .into_element(cx),
                );
                out.push(
                    shadcn::AlertDialogFooter::new([
                        shadcn::AlertDialogCancel::from_scope("Cancel")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .test_id("ui-gallery-alert-dialog-destructive-cancel")
                            .into_element(cx),
                        shadcn::AlertDialogAction::from_scope("Delete")
                            .variant(shadcn::ButtonVariant::Destructive)
                            .test_id("ui-gallery-alert-dialog-destructive-action")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                );
            })
            .size(shadcn::AlertDialogContentSize::Sm)
            .test_id("ui-gallery-alert-dialog-destructive-content")),
        ])
        .into_element(cx)
}
// endregion: example
