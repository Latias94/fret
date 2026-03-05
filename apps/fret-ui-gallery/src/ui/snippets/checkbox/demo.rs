pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    model: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let model = cx.with_state(Models::default, |st| st.model.clone());
    let model = model.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.model = Some(model.clone()));
        model
    });

    ui::h_flex(|cx| {
        vec![
            shadcn::Checkbox::new(model)
                // Required for label click -> focus/toggle forwarding.
                .control_id("ui-gallery-checkbox-demo-toggle")
                .a11y_label("Accept terms")
                .test_id("ui-gallery-checkbox-demo-toggle")
                .into_element(cx),
            shadcn::FieldLabel::new("Accept terms and conditions")
                .for_control("ui-gallery-checkbox-demo-toggle")
                .test_id("ui-gallery-checkbox-demo-label")
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N3)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-checkbox-demo")
}
// endregion: example
