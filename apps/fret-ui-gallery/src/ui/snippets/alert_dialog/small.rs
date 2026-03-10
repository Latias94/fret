pub const SOURCE: &str = include_str!("small.rs");

// region: example
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
            shadcn::Button::new("Show Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .test_id("ui-gallery-alert-dialog-small-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::AlertDialogContent::build(move |cx, children| {
                children.push(
                    shadcn::AlertDialogHeader::new(vec![
                        shadcn::AlertDialogTitle::new("Allow accessory to connect?")
                            .into_element(cx),
                        shadcn::AlertDialogDescription::new(
                            "Do you want to allow the USB accessory to connect to this device?",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                );
                children.push(
                    shadcn::AlertDialogFooter::new(vec![
                        shadcn::AlertDialogCancel::new("Don't allow", open_for_children.clone())
                            .test_id("ui-gallery-alert-dialog-small-cancel")
                            .into_element(cx),
                        shadcn::AlertDialogAction::new("Allow", open_for_children.clone())
                            .test_id("ui-gallery-alert-dialog-small-action")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                );
            })
            .size(shadcn::AlertDialogContentSize::Sm)
            .into_element(cx)
            .test_id("ui-gallery-alert-dialog-small-content")
        },
    )
}
// endregion: example
