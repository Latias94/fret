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

    shadcn::AlertDialog::new(open).into_element_parts(
        cx,
        move |cx| {
            let trigger = shadcn::Button::new("Show Dialog (Parts)")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .test_id("ui-gallery-alert-dialog-parts-trigger")
                .into_element(cx);
            shadcn::AlertDialogTrigger::new(trigger)
        },
        shadcn::AlertDialogPortal::new(),
        shadcn::AlertDialogOverlay::new(),
        move |cx| {
            let header = shadcn::AlertDialogHeader::new(vec![
                shadcn::AlertDialogTitle::new("Part-based AlertDialog").into_element(cx),
                shadcn::AlertDialogDescription::new(
                    "Thin adapters for shadcn-style authoring (Trigger/Portal/Overlay).",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let footer = shadcn::AlertDialogFooter::new(vec![
                shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                    .test_id("ui-gallery-alert-dialog-parts-cancel")
                    .into_element(cx),
                shadcn::AlertDialogAction::new("Continue", open_for_children.clone())
                    .test_id("ui-gallery-alert-dialog-parts-action")
                    .into_element(cx),
            ])
            .into_element(cx);

            shadcn::AlertDialogContent::new(vec![header, footer])
                .into_element(cx)
                .test_id("ui-gallery-alert-dialog-parts-content")
        },
    )
}
// endregion: example

