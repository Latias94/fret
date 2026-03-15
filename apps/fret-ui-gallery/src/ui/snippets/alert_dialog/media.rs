pub const SOURCE: &str = include_str!("media.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let open_for_trigger = open.clone();
    let open_for_children = open.clone();

    shadcn::AlertDialog::new(open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Share Project")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .test_id("ui-gallery-alert-dialog-media-trigger")
                .into_element(cx)
        },
        move |cx| {
            let icon = shadcn::raw::icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.circle-plus"),
                Some(Px(32.0)),
                None,
            );

            let header = shadcn::AlertDialogHeader::new(vec![
                shadcn::AlertDialogTitle::new("Share this project?").into_element(cx),
                shadcn::AlertDialogDescription::new(
                    "Anyone with the link will be able to view and edit this project.",
                )
                .into_element(cx),
            ])
            .media(shadcn::AlertDialogMedia::new(icon).into_element(cx))
            .into_element(cx);

            let footer = shadcn::AlertDialogFooter::new(vec![
                shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                    .test_id("ui-gallery-alert-dialog-media-cancel")
                    .into_element(cx),
                shadcn::AlertDialogAction::new("Share", open_for_children.clone())
                    .test_id("ui-gallery-alert-dialog-media-action")
                    .into_element(cx),
            ])
            .into_element(cx);

            shadcn::AlertDialogContent::new(vec![header, footer])
                .into_element(cx)
                .test_id("ui-gallery-alert-dialog-media-content")
        },
    )
}
// endregion: example
