pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let open = match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let open_for_trigger = open.clone();

    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("No Close Button")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-no-close-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([shadcn::DialogHeader::new([
                shadcn::DialogTitle::new("No Close Button").into_element(cx),
                shadcn::DialogDescription::new(
                    "This dialog omits explicit close controls and relies on Escape or overlay dismissal.",
                )
                .into_element(cx),
            ])
            .into_element(cx)])
            .show_close_button(false)
            .into_element(cx)
            .test_id("ui-gallery-dialog-no-close-content")
        },
    )
}
// endregion: example
