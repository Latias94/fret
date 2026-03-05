pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    input_value: Option<Model<String>>,
    textarea_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (input_value, textarea_value) = cx.with_state(Models::default, |st| {
        (st.input_value.clone(), st.textarea_value.clone())
    });

    let (input_value, textarea_value) = match (input_value, textarea_value) {
        (Some(input_value), Some(textarea_value)) => (input_value, textarea_value),
        _ => {
            let input_value = cx.app.models_mut().insert(String::new());
            let textarea_value = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| {
                st.input_value = Some(input_value.clone());
                st.textarea_value = Some(textarea_value.clone());
            });
            (input_value, textarea_value)
        }
    };

    let input = shadcn::InputGroup::new(input_value)
        .placeholder("Send a message...")
        .disabled(true)
        .a11y_label("Send a message")
        .trailing([shadcn::Spinner::new().into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    let validating = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::typography::muted(cx, "Validating..."),
            ]
        },
    );

    let send = shadcn::InputGroupButton::new("")
        .a11y_label("Send")
        .size(shadcn::InputGroupButtonSize::IconSm)
        .icon(fret_icons::IconId::new_static("lucide.arrow-up"))
        .into_element(cx);

    let actions = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .items_center()
            .justify_between(),
        |_cx| vec![validating, send],
    )
    .test_id("ui-gallery-spinner-extras-textarea-actions");

    let textarea = shadcn::InputGroup::new(textarea_value)
        .textarea()
        .placeholder("Send a message...")
        .disabled(true)
        .a11y_label("Send a message textarea")
        .block_end([actions])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full().max_w(Px(448.0))),
        |_cx| vec![input, textarea],
    )
    .test_id("ui-gallery-spinner-input-group")
}

// endregion: example
