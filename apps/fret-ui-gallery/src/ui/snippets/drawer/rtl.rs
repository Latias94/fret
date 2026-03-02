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
    let open_for_close = open.clone();

    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        move |cx| {
            shadcn::Drawer::new(open.clone()).into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Open RTL Drawer")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open_for_trigger.clone())
                        .test_id("ui-gallery-drawer-rtl-trigger")
                        .into_element(cx)
                },
                move |cx| {
                    shadcn::DrawerContent::new([
                        shadcn::DrawerHeader::new([
                            shadcn::DrawerTitle::new("RTL Drawer").into_element(cx),
                            shadcn::DrawerDescription::new(
                                "Drawer layout should follow right-to-left direction context.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_close.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-drawer-rtl-content")
                },
            )
        },
    )
}
// endregion: example
