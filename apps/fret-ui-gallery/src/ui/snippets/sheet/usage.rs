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

    let trigger = shadcn::SheetTrigger::new(
        shadcn::Button::new("Open")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
    );

    shadcn::Sheet::new(open)
        .side(shadcn::SheetSide::Right)
        .compose()
        .trigger(trigger)
        .portal(shadcn::SheetPortal::new())
        .overlay(shadcn::SheetOverlay::new())
        .content_with(move |cx| {
            shadcn::SheetContent::new([
                shadcn::SheetHeader::new([
                    shadcn::SheetTitle::new("Edit profile").into_element(cx),
                    shadcn::SheetDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::SheetFooter::new([
                    shadcn::Button::new("Save changes").into_element(cx),
                    shadcn::SheetClose::from_scope().into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
