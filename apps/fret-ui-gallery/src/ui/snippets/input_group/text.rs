pub const SOURCE: &str = include_str!("text.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let amount = cx.local_model_keyed("amount", String::new);
    let website = cx.local_model_keyed("website", String::new);
    let username = cx.local_model_keyed("username", String::new);
    let message = cx.local_model_keyed("message", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let currency_group = shadcn::InputGroup::new(amount)
        .a11y_label("Amount")
        .placeholder("0.00")
        .leading([shadcn::InputGroupText::new("$").into_element(cx)])
        .trailing([shadcn::InputGroupText::new("USD").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let website_group = shadcn::InputGroup::new(website)
        .a11y_label("Website")
        .placeholder("example.com")
        .control_test_id("ui-gallery-input-group-text-control")
        .leading([shadcn::InputGroupText::new("https://")
            .into_element(cx)
            .test_id("ui-gallery-input-group-text-leading")])
        .trailing([shadcn::InputGroupText::new(".com")
            .into_element(cx)
            .test_id("ui-gallery-input-group-text-trailing")])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let username_group = shadcn::InputGroup::new(username)
        .a11y_label("Username")
        .placeholder("Enter your username")
        .trailing([shadcn::InputGroupText::new("@company.com").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let textarea_group = shadcn::InputGroup::new(message)
        .textarea()
        .a11y_label("Message")
        .placeholder("Enter your message")
        .block_end([shadcn::InputGroupText::new("120 characters left")
            .size(shadcn::InputGroupTextSize::Xs)
            .into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    ui::v_stack(move |_cx| {
        vec![
            currency_group,
            website_group,
            username_group,
            textarea_group,
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(max_w)
    .into_element(cx)
    .test_id("ui-gallery-input-group-text")
}
// endregion: example
