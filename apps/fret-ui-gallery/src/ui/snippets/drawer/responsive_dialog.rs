pub const SOURCE: &str = include_str!("responsive_dialog.rs");

// region: example
use fret_core::{Px, TextAlign};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn profile_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    model: Model<String>,
    input_test_id: Option<&'static str>,
) -> AnyElement {
    let input =
        shadcn::Input::new(model).refine_layout(LayoutRefinement::default().w_full().min_w_0());
    let input = match input_test_id {
        Some(test_id) => input.test_id(test_id),
        None => input,
    };

    shadcn::Field::new([
        shadcn::FieldLabel::new(label).into_element(cx),
        input.into_element(cx),
    ])
    .into_element(cx)
}

#[derive(Clone, Copy)]
struct ProfileFormTestIds {
    form: &'static str,
    email: &'static str,
    username: &'static str,
    save: &'static str,
}

fn profile_form<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    email: Model<String>,
    username: Model<String>,
    test_ids: Option<ProfileFormTestIds>,
) -> AnyElement {
    let form = ui::v_stack(|cx| {
        vec![
            profile_field(cx, "Email", email, test_ids.map(|ids| ids.email)),
            profile_field(cx, "Username", username, test_ids.map(|ids| ids.username)),
            match test_ids {
                Some(ids) => shadcn::Button::new("Save changes")
                    .test_id(ids.save)
                    .into_element(cx),
                None => shadcn::Button::new("Save changes").into_element(cx),
            },
        ]
    })
    .gap(Space::N6)
    .items_stretch()
    .layout(LayoutRefinement::default().w_full().min_w_0());

    match test_ids {
        Some(ids) => form.into_element(cx).test_id(ids.form),
        None => form.into_element(cx),
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let desktop_open = cx.local_model_keyed("desktop_open", || false);
    let mobile_open = cx.local_model_keyed("mobile_open", || false);
    let email = cx.local_model_keyed("email", || String::from("shadcn@example.com"));
    let username = cx.local_model_keyed("username", || String::from("@shadcn"));

    let desktop_open_trigger = desktop_open.clone();
    let mobile_open_trigger = mobile_open.clone();
    let desktop_email = email.clone();
    let desktop_username = username.clone();
    let mobile_email = email.clone();
    let mobile_username = username.clone();

    let desktop_dialog = shadcn::Dialog::new(desktop_open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Edit Profile (Desktop)")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(desktop_open_trigger.clone())
                .test_id("ui-gallery-drawer-responsive-desktop-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Edit profile").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                profile_form(cx, desktop_email.clone(), desktop_username.clone(), None),
            ])
            .refine_layout(LayoutRefinement::default().max_w(Px(425.0)))
            .into_element(cx)
            .test_id("ui-gallery-drawer-responsive-desktop-content")
        },
    );

    let mobile_drawer = shadcn::Drawer::new(mobile_open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Edit Profile (Mobile)")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(mobile_open_trigger.clone())
                .test_id("ui-gallery-drawer-responsive-mobile-trigger")
                .into_element(cx)
        },
        move |cx| {
            let form = profile_form(
                cx,
                mobile_email.clone(),
                mobile_username.clone(),
                Some(ProfileFormTestIds {
                    form: "ui-gallery-drawer-responsive-mobile-form",
                    email: "ui-gallery-drawer-responsive-mobile-email",
                    username: "ui-gallery-drawer-responsive-mobile-username",
                    save: "ui-gallery-drawer-responsive-mobile-save",
                }),
            );
            let form = ui::v_stack(move |_cx| vec![form])
                .px_4()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new("Edit profile").into_element(cx),
                    shadcn::DrawerDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .into_element(cx),
                ])
                .text_align(TextAlign::Start)
                .into_element(cx),
                form,
                shadcn::DrawerFooter::new([shadcn::DrawerClose::from_scope().build(
                    cx,
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-drawer-responsive-mobile-cancel"),
                )])
                .refine_style(ChromeRefinement::default().pt(Space::N2))
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-drawer-responsive-mobile-content")
        },
    );

    ui::h_flex(move |_cx| [desktop_dialog, mobile_drawer])
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}
// endregion: example
