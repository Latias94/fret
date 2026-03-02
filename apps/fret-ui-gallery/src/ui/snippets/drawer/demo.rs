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
    let trigger_open = open.clone();
    let close_open = open.clone();

    shadcn::Drawer::new(open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Open Drawer")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(trigger_open.clone())
                .test_id("ui-gallery-drawer-demo-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new("Move Goal").into_element(cx),
                    shadcn::DrawerDescription::new("Set your daily activity goal.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DrawerFooter::new([
                    shadcn::Button::new("Submit").into_element(cx),
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(close_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-drawer-demo-content")
        },
    )
}
// endregion: example
