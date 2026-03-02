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

    let icon_row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new()
                    .icon(fret_icons::ids::ui::SETTINGS)
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-spinner-extras-custom-icon");

    let input = shadcn::InputGroup::new(input_value)
        .a11y_label("Send a message")
        .trailing([shadcn::Spinner::new().into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    let textarea = shadcn::InputGroup::new(textarea_value)
        .textarea()
        .a11y_label("Send a message textarea")
        .block_end([stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new().into_element(cx),
                    shadcn::typography::muted(cx, "Validating..."),
                    shadcn::InputGroupButton::new("")
                        .a11y_label("Send")
                        .size(shadcn::InputGroupButtonSize::IconSm)
                        .icon(fret_icons::IconId::new_static("lucide.arrow-up"))
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-extras-textarea-actions")])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Extras are Fret-specific demos and regression gates (not part of upstream shadcn SpinnerDemo).",
                ),
                icon_row,
                input,
                textarea,
            ]
        },
    )
    .test_id("ui-gallery-spinner-extras")
}

// endregion: example

