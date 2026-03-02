pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    email: Option<Model<String>>,
}

fn email_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.email {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.email = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let email = email_model(cx);
    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(max_w),
        |cx| {
            vec![
                shadcn::Label::new("Your email address").into_element(cx),
                shadcn::Input::new(email)
                    .placeholder("you@example.com")
                    .a11y_label("Email")
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-label-demo")
}
// endregion: example
