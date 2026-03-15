pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::headless::form_state::FormState;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let form_state = cx.local_model_keyed("form_state", FormState::default);
    let username = cx.local_model_keyed("username", String::new);

    let username_field = shadcn::FormField::new(
        form_state,
        "username",
        [shadcn::Input::new(username)
            .placeholder("shadcn")
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)],
    )
    .label("Username")
    .description("This is your public display name.")
    .into_element(cx);

    shadcn::Form::new([
        username_field,
        shadcn::Button::new("Submit")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
    .into_element(cx)
    .test_id("ui-gallery-form-usage")
}
// endregion: example
