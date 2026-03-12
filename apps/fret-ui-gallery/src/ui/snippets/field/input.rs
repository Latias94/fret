pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let username = cx.local_model_keyed("username", String::new);
    let password = cx.local_model_keyed("password", String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let username_id = "username";
    let password_id = "password";

    shadcn::FieldSet::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::FieldGroup::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::FieldLabel::new("Username").for_control(username_id),
                        );
                        out.push_ui(
                            cx,
                            shadcn::Input::new(username)
                                .control_id(username_id)
                                .placeholder("Max Leiter")
                                .test_id("ui-gallery-field-input-username"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::FieldDescription::new(
                                "Choose a unique username for your account.",
                            )
                            .for_control(username_id),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::FieldLabel::new("Password").for_control(password_id),
                        );
                        out.push_ui(
                            cx,
                            shadcn::FieldDescription::new("Must be at least 8 characters long.")
                                .for_control(password_id),
                        );
                        out.push_ui(
                            cx,
                            shadcn::Input::new(password)
                                .control_id(password_id)
                                .password()
                                .placeholder("••••••••")
                                .test_id("ui-gallery-field-input-password"),
                        );
                    }),
                );
            }),
        );
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-input")
}
// endregion: example
