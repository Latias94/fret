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

    shadcn::Drawer::new(open)
        .compose()
        .trigger(shadcn::DrawerTrigger::build(
            shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
        ))
        .content_with(move |cx| {
            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::DrawerDescription::new("This action cannot be undone.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DrawerFooter::new([
                    shadcn::Button::new("Submit").into_element(cx),
                    shadcn::DrawerClose::from_scope().build(
                        cx,
                        shadcn::Button::new("Cancel").variant(shadcn::ButtonVariant::Outline),
                    ),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
