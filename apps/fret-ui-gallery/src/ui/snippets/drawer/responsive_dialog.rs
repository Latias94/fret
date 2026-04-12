pub const SOURCE: &str = include_str!("responsive_dialog.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Px, TextAlign};
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn profile_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    model: Model<String>,
    input_test_id: Option<&'static str>,
) -> impl IntoUiElement<H> + use<H> {
    let input =
        shadcn::Input::new(model).refine_layout(LayoutRefinement::default().w_full().min_w_0());
    let input = match input_test_id {
        Some(test_id) => input.test_id(test_id),
        None => input,
    };

    shadcn::Field::new(ui::children![cx; shadcn::FieldLabel::new(label), input]).into_element(cx)
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
) -> impl IntoUiElement<H> + use<H> {
    let form = ui::v_stack(|cx| {
        ui::children![
            cx;
            profile_field(cx, "Email", email, test_ids.map(|ids| ids.email)),
            profile_field(cx, "Username", username, test_ids.map(|ids| ids.username)),
            match test_ids {
                Some(ids) => shadcn::Button::new("Save changes").test_id(ids.save),
                None => shadcn::Button::new("Save changes"),
            }
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
            shadcn::Button::new("Edit Profile")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(desktop_open_trigger.clone())
                .test_id("ui-gallery-drawer-responsive-desktop-trigger")
                .into_element(cx)
        },
        move |cx| {
            let form = profile_form(
                cx,
                desktop_email.clone(),
                desktop_username.clone(),
                Some(ProfileFormTestIds {
                    form: "ui-gallery-drawer-responsive-desktop-form",
                    email: "ui-gallery-drawer-responsive-desktop-email",
                    username: "ui-gallery-drawer-responsive-desktop-username",
                    save: "ui-gallery-drawer-responsive-desktop-save",
                }),
            )
            .into_element(cx);
            shadcn::DialogContent::new([])
                .refine_layout(LayoutRefinement::default().max_w(Px(425.0)))
                .with_children(cx, |cx| {
                    vec![
                        shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::DialogTitle::new("Edit profile")
                                    .into_element(cx)
                                    .test_id("ui-gallery-drawer-responsive-desktop-title"),
                                shadcn::DialogDescription::new(
                                    "Make changes to your profile here. Click save when you're done.",
                                )
                                .into_element(cx)
                                .test_id("ui-gallery-drawer-responsive-desktop-description"),
                            ]
                        }),
                        form,
                    ]
                })
                .test_id("ui-gallery-drawer-responsive-desktop-content")
        },
    );

    let mobile_drawer = shadcn::Drawer::new(mobile_open)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Edit Profile")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(mobile_open_trigger.clone())
                    .test_id("ui-gallery-drawer-responsive-mobile-trigger"),
            )),
            shadcn::DrawerPart::content_with(move |cx| {
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
                )
                .into_element(cx);
                let form = ui::v_stack(move |_cx| vec![form])
                    .px_4()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                shadcn::DrawerContent::new([])
                    .children(|cx| {
                        ui::children![
                            cx;
                            shadcn::DrawerHeader::new([])
                                .text_align(TextAlign::Start)
                                .children(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::DrawerTitle::new("Edit profile")
                                            .into_element(cx)
                                            .test_id("ui-gallery-drawer-responsive-mobile-title"),
                                        shadcn::DrawerDescription::new(
                                            "Make changes to your profile here. Click save when you're done.",
                                        )
                                        .into_element(cx)
                                        .test_id("ui-gallery-drawer-responsive-mobile-description")
                                    ]
                                }),
                            form,
                            shadcn::DrawerFooter::new([])
                                .refine_style(ChromeRefinement::default().pt(Space::N2))
                                .children(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::DrawerClose::from_scope().child(
                                            shadcn::Button::new("Cancel")
                                                .variant(shadcn::ButtonVariant::Outline)
                                                .test_id("ui-gallery-drawer-responsive-mobile-cancel"),
                                        )
                                    ]
                                })
                        ]
                    })
                    .test_id("ui-gallery-drawer-responsive-mobile-content")
                    .into_element(cx)
            }),
        ])
        .into_element(cx);

    ui::h_flex(move |_cx| [desktop_dialog, mobile_drawer])
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}
// endregion: example
