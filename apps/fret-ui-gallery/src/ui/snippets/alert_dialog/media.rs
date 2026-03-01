// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = open_model(cx);
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
            let icon = shadcn::icon::icon_with(
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

