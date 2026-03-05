pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.with_state(Models::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    };

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    ui::v_stack(move |cx| {
        vec![
            shadcn::InputGroup::new(value.clone())
                .a11y_label("Search")
                .leading([shadcn::InputGroupText::new("icon").into_element(cx)])
                .trailing([shadcn::InputGroupButton::new("Go")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx)])
                .trailing_has_button(true)
                .test_id("ui-gallery-input-group-demo")
                .into_element(cx),
            shadcn::InputGroup::new(value.clone())
                .textarea()
                .a11y_label("Message")
                .block_end([
                    shadcn::InputGroupText::new("Ctrl+Enter to send").into_element(cx),
                    shadcn::InputGroupButton::new("Send")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::InputGroupButtonSize::Sm)
                        .into_element(cx),
                ])
                .block_end_border_top(true)
                .textarea_min_height(Px(90.0))
                .test_id("ui-gallery-input-group-demo-textarea")
                .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(max_w_xs)
    .into_element(cx)
}
// endregion: example
