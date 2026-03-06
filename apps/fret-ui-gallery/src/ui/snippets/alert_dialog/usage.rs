pub const SOURCE: &str = include_str!("usage.rs");

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

    let trigger = shadcn::AlertDialogTrigger::new(
        shadcn::Button::new("Open")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
    );

    let content = {
        let header = shadcn::AlertDialogHeader::new(vec![
            shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
            shadcn::AlertDialogDescription::new(
                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
            )
            .into_element(cx),
        ])
        .into_element(cx);
        let footer = shadcn::AlertDialogFooter::new(vec![
            shadcn::AlertDialogCancel::from_scope("Cancel").into_element(cx),
            shadcn::AlertDialogAction::from_scope("Continue").into_element(cx),
        ])
        .into_element(cx);

        shadcn::AlertDialogContent::new(vec![header, footer]).into_element(cx)
    };

    shadcn::AlertDialog::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::AlertDialogPortal::new())
        .overlay(shadcn::AlertDialogOverlay::new())
        .content(content)
        .into_element(cx)
}
// endregion: example
