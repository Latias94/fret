pub const SOURCE: &str = include_str!("small_with_media.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let open_for_trigger = open.clone();
    let open_for_children = open.clone();

    shadcn::AlertDialog::new(open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Show Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .test_id("ui-gallery-alert-dialog-small-media-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::AlertDialogContent::build(move |cx, children| {
                let icon = fret_ui_shadcn::icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.bluetooth"),
                    Some(Px(32.0)),
                    None,
                );

                children.push(
                    shadcn::AlertDialogHeader::new(vec![
                        shadcn::AlertDialogTitle::new("Allow accessory to connect?")
                            .into_element(cx),
                        shadcn::AlertDialogDescription::new(
                            "Do you want to allow the USB accessory to connect to this device?",
                        )
                        .into_element(cx),
                    ])
                    .media(shadcn::AlertDialogMedia::new(icon).into_element(cx))
                    .into_element(cx),
                );
                children.push(
                    shadcn::AlertDialogFooter::new(vec![
                        shadcn::AlertDialogCancel::new("Don't allow", open_for_children.clone())
                            .test_id("ui-gallery-alert-dialog-small-media-cancel")
                            .into_element(cx),
                        shadcn::AlertDialogAction::new("Allow", open_for_children.clone())
                            .test_id("ui-gallery-alert-dialog-small-media-action")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                );
            })
            .size(shadcn::AlertDialogContentSize::Sm)
            .into_element(cx)
            .test_id("ui-gallery-alert-dialog-small-media-content")
        },
    )
}
// endregion: example
