pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let open_for_trigger = open.clone();
    let open_for_children = open.clone();

    shadcn::AlertDialog::new(open).build(
        cx,
        shadcn::Button::new("Show Dialog")
            .variant(shadcn::ButtonVariant::Outline)
            .toggle_model(open_for_trigger.clone())
            .test_id("ui-gallery-alert-dialog-demo-trigger"),
        shadcn::AlertDialogContent::build(|cx, out| {
                out.push(shadcn::AlertDialogHeader::build(|cx, out| {
                        out.push(
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                        );
                        out.push(
                            shadcn::AlertDialogDescription::new(
                                "This action cannot be undone. This will permanently delete your account from our servers.",
                            )
                            .into_element(cx),
                        );
                    })
                    .into_element(cx));
                out.push(shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push(
                            shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                                .test_id("ui-gallery-alert-dialog-demo-cancel")
                                .into_element(cx),
                        );
                        out.push(
                            shadcn::AlertDialogAction::new("Continue", open_for_children.clone())
                                .test_id("ui-gallery-alert-dialog-demo-action")
                                .into_element(cx),
                        );
                    })
                    .into_element(cx));
        })
        .test_id("ui-gallery-alert-dialog-demo-content"),
    )
}
// endregion: example
