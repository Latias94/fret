// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.with_state(Models::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let trigger_open = open.clone();

    shadcn::Sheet::new(open.clone())
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new([shadcn::SheetHeader::new([
                    shadcn::SheetTitle::new("No Close Button").into_element(cx),
                    shadcn::SheetDescription::new(
                        "This example intentionally omits footer actions. Use outside press or Escape to close.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx)])
                .show_close_button(false)
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-sheet-no-close-button")
}
// endregion: example
