pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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
            shadcn::Button::new("Delete Chat")
                .variant(shadcn::ButtonVariant::Destructive)
                .toggle_model(open_for_trigger.clone())
                .test_id("ui-gallery-alert-dialog-destructive-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::AlertDialogContent::build(move |cx, children| {
                let icon = fret_ui_shadcn::icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.trash-2"),
                    Some(Px(32.0)),
                    None,
                );

                children.push(
                    shadcn::AlertDialogHeader::new(vec![
                        shadcn::AlertDialogTitle::new("Delete chat?").into_element(cx),
                        shadcn::AlertDialogDescription::new(
                            "This will permanently delete this chat conversation. Review settings if you need to clear related memories.",
                        )
                        .into_element(cx),
                    ])
                    .media(shadcn::AlertDialogMedia::new(icon).into_element(cx))
                    .into_element(cx),
                );
                children.push(
                    shadcn::AlertDialogFooter::new(vec![
                        shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                            .test_id("ui-gallery-alert-dialog-destructive-cancel")
                            .into_element(cx),
                        shadcn::AlertDialogAction::new("Delete", open_for_children.clone())
                            .variant(shadcn::ButtonVariant::Destructive)
                            .test_id("ui-gallery-alert-dialog-destructive-action")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                );
            })
                .size(shadcn::AlertDialogContentSize::Sm)
                .into_element(cx)
                .test_id("ui-gallery-alert-dialog-destructive-content")
        },
    )
}
// endregion: example
