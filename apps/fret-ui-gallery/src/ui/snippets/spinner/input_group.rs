pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let input_value = cx.local_model_keyed("input_value", String::new);
    let textarea_value = cx.local_model_keyed("textarea_value", String::new);

    let input = shadcn::InputGroup::new(input_value)
        .placeholder("Send a message...")
        .disabled(true)
        .a11y_label("Send a message")
        .trailing([shadcn::Spinner::new().into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    let validating = ui::h_row(|cx| {
        vec![
            shadcn::Spinner::new().into_element(cx),
            shadcn::raw::typography::muted("Validating...").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let send = shadcn::InputGroupButton::new("")
        .a11y_label("Send")
        .size(shadcn::InputGroupButtonSize::IconSm)
        .icon(fret_icons::IconId::new_static("lucide.arrow-up"))
        .into_element(cx);

    let actions = ui::h_flex(|_cx| vec![validating, send])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N2)
        .items_center()
        .justify_between()
        .into_element(cx)
        .test_id("ui-gallery-spinner-extras-textarea-actions");

    let textarea = shadcn::InputGroup::new(textarea_value)
        .textarea()
        .placeholder("Send a message...")
        .disabled(true)
        .a11y_label("Send a message textarea")
        .block_end([actions])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    ui::v_flex(|_cx| vec![input, textarea])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().max_w(Px(448.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-input-group")
}

// endregion: example
