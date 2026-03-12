pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let open_for_trigger = open.clone();
    let open_for_children = open.clone();

    shadcn::AlertDialog::new(open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Open")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .test_id("ui-gallery-alert-dialog-basic-trigger")
                .into_element(cx)
        },
        move |cx| {
            let header = shadcn::AlertDialogHeader::new(vec![
                shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                shadcn::AlertDialogDescription::new(
                    "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                )
                .into_element(cx),
            ])
            .into_element(cx);
            let footer = shadcn::AlertDialogFooter::new(vec![
                shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                    .test_id("ui-gallery-alert-dialog-basic-cancel")
                    .into_element(cx),
                shadcn::AlertDialogAction::new("Continue", open_for_children.clone())
                    .test_id("ui-gallery-alert-dialog-basic-action")
                    .into_element(cx),
            ])
            .into_element(cx);

            shadcn::AlertDialogContent::new(vec![header, footer])
                .into_element(cx)
                .test_id("ui-gallery-alert-dialog-basic-content")
        },
    )
}
// endregion: example
