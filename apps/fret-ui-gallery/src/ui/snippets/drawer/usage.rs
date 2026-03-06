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

    let trigger = shadcn::DrawerTrigger::new(
        shadcn::Button::new("Open")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
    );

    let content = shadcn::DrawerContent::new([
        shadcn::DrawerHeader::new([
            shadcn::DrawerTitle::new("Move Goal").into_element(cx),
            shadcn::DrawerDescription::new("Set your daily activity goal.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::DrawerFooter::new([
            shadcn::Button::new("Submit").into_element(cx),
            shadcn::DrawerClose::from_scope().into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Drawer::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::DrawerPortal::new())
        .overlay(shadcn::DrawerOverlay::new())
        .content(content)
        .into_element(cx)
}
// endregion: example
