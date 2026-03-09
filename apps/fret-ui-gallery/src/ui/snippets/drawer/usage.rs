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
        .portal(shadcn::DrawerPortal::new())
        .overlay(shadcn::DrawerOverlay::new())
        .content_with(move |cx| {
            shadcn::DrawerContent::build(|cx, out| {
                out.push(
                    shadcn::DrawerHeader::build(|cx, out| {
                        out.push(shadcn::DrawerTitle::new("Move Goal").into_element(cx));
                        out.push(
                            shadcn::DrawerDescription::new("Set your daily activity goal.")
                                .into_element(cx),
                        );
                    })
                    .into_element(cx),
                );
                out.push(
                    shadcn::DrawerFooter::build(|cx, out| {
                        out.push(shadcn::Button::new("Submit").into_element(cx));
                        out.push(shadcn::DrawerClose::from_scope().into_element(cx));
                    })
                    .into_element(cx),
                );
            })
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
