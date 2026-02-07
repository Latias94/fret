use super::super::*;

pub(super) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct AlertDialogModels {
        basic_open: Option<Model<bool>>,
        small_open: Option<Model<bool>>,
        media_open: Option<Model<bool>>,
        small_media_open: Option<Model<bool>>,
        destructive_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
    }

    let (basic_open, small_open, media_open, small_media_open, destructive_open, rtl_open) = cx
        .with_state(AlertDialogModels::default, |state| {
            (
                state.basic_open.clone(),
                state.small_open.clone(),
                state.media_open.clone(),
                state.small_media_open.clone(),
                state.destructive_open.clone(),
                state.rtl_open.clone(),
            )
        });

    let (basic_open, small_open, media_open, small_media_open, destructive_open, rtl_open) = match (
        basic_open,
        small_open,
        media_open,
        small_media_open,
        destructive_open,
        rtl_open,
    ) {
        (
            Some(basic_open),
            Some(small_open),
            Some(media_open),
            Some(small_media_open),
            Some(destructive_open),
            Some(rtl_open),
        ) => (
            basic_open,
            small_open,
            media_open,
            small_media_open,
            destructive_open,
            rtl_open,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let small_open = cx.app.models_mut().insert(false);
            let media_open = cx.app.models_mut().insert(false);
            let small_media_open = cx.app.models_mut().insert(false);
            let destructive_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);
            cx.with_state(AlertDialogModels::default, |state| {
                state.basic_open = Some(basic_open.clone());
                state.small_open = Some(small_open.clone());
                state.media_open = Some(media_open.clone());
                state.small_media_open = Some(small_media_open.clone());
                state.destructive_open = Some(destructive_open.clone());
                state.rtl_open = Some(rtl_open.clone());
            });
            (
                basic_open,
                small_open,
                media_open,
                small_media_open,
                destructive_open,
                rtl_open,
            )
        }
    };

    let theme = Theme::global(&*cx.app).clone();

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let media_chip =
        |cx: &mut ElementContext<'_, App>, icon_name: &'static str, destructive: bool| {
            let (bg, border, fg) = if destructive {
                (
                    ColorRef::Color(theme.color_required("destructive")),
                    ColorRef::Color(theme.color_required("destructive")),
                    ColorRef::Color(theme.color_required("destructive-foreground")),
                )
            } else {
                (
                    ColorRef::Color(theme.color_required("muted")),
                    ColorRef::Color(theme.color_required("border")),
                    ColorRef::Color(theme.color_required("foreground")),
                )
            };

            let icon = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .items_center()
                    .justify_center(),
                move |cx| {
                    vec![shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static(icon_name),
                    )]
                },
            );

            cx.container(
                decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .border_1()
                        .rounded(Radius::Full)
                        .bg(bg)
                        .border_color(border)
                        .text_color(fg)
                        .p(Space::N2),
                    LayoutRefinement::default().w_px(Px(40.0)).h_px(Px(40.0)),
                ),
                move |_cx| [icon],
            )
        };

    let build_dialog = |cx: &mut ElementContext<'_, App>,
                        test_id_prefix: &'static str,
                        open_model: Model<bool>,
                        trigger_label: &'static str,
                        trigger_variant: shadcn::ButtonVariant,
                        title: &'static str,
                        description: &'static str,
                        cancel_label: &'static str,
                        action_label: &'static str,
                        action_variant: shadcn::ButtonVariant,
                        max_w: Px,
                        media_icon: Option<&'static str>,
                        media_destructive: bool| {
        let open_for_trigger = open_model.clone();
        let open_for_children = open_model.clone();
        shadcn::AlertDialog::new(open_model).into_element(
            cx,
            move |cx| {
                shadcn::Button::new(trigger_label)
                    .variant(trigger_variant)
                    .test_id(format!("{test_id_prefix}-trigger"))
                    .toggle_model(open_for_trigger.clone())
                    .into_element(cx)
            },
            move |cx| {
                let mut header_children = Vec::new();
                if let Some(icon_name) = media_icon {
                    header_children.push(media_chip(cx, icon_name, media_destructive));
                }
                header_children.push(shadcn::AlertDialogTitle::new(title).into_element(cx));
                header_children
                    .push(shadcn::AlertDialogDescription::new(description).into_element(cx));

                let header = shadcn::AlertDialogHeader::new(header_children).into_element(cx);
                let footer = shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new(cancel_label, open_for_children.clone())
                        .test_id(format!("{test_id_prefix}-cancel"))
                        .into_element(cx),
                    shadcn::AlertDialogAction::new(action_label, open_for_children.clone())
                        .variant(action_variant)
                        .test_id(format!("{test_id_prefix}-action"))
                        .into_element(cx),
                ])
                .into_element(cx);

                shadcn::AlertDialogContent::new(vec![header, footer])
                    .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default().test_id(format!("{test_id_prefix}-content")),
                    )
            },
        )
    };

    let demo_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-demo",
        open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Are you absolutely sure?",
        "This action cannot be undone. This will permanently delete your account from our servers.",
        "Cancel",
        "Continue",
        shadcn::ButtonVariant::Default,
        Px(520.0),
        None,
        false,
    );
    let demo = section_card(cx, "Demo", demo_content);

    let basic_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-basic",
        basic_open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Are you absolutely sure?",
        "This action cannot be undone. This will permanently delete your account from our servers.",
        "Cancel",
        "Continue",
        shadcn::ButtonVariant::Default,
        Px(520.0),
        None,
        false,
    );
    let basic = section_card(cx, "Basic", basic_content);

    let small_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-small",
        small_open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Allow accessory to connect?",
        "Do you want to allow the USB accessory to connect to this device?",
        "Don't allow",
        "Allow",
        shadcn::ButtonVariant::Default,
        Px(400.0),
        None,
        false,
    );
    let small = section_card(cx, "Small", small_content);

    let media_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-media",
        media_open,
        "Share Project",
        shadcn::ButtonVariant::Outline,
        "Share this project?",
        "Anyone with the link will be able to view and edit this project.",
        "Cancel",
        "Share",
        shadcn::ButtonVariant::Default,
        Px(520.0),
        Some("lucide.circle-plus"),
        false,
    );
    let media = section_card(cx, "Media", media_content);

    let small_with_media_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-small-media",
        small_media_open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Allow accessory to connect?",
        "Do you want to allow the USB accessory to connect to this device?",
        "Don't allow",
        "Allow",
        shadcn::ButtonVariant::Default,
        Px(400.0),
        Some("lucide.bluetooth"),
        false,
    );
    let small_with_media = section_card(cx, "Small with Media", small_with_media_content);

    let destructive_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-destructive",
        destructive_open,
        "Delete Chat",
        shadcn::ButtonVariant::Destructive,
        "Delete chat?",
        "This will permanently delete this chat conversation. Review settings if you need to clear related memories.",
        "Cancel",
        "Delete",
        shadcn::ButtonVariant::Destructive,
        Px(400.0),
        Some("lucide.trash-2"),
        true,
    );
    let destructive = section_card(cx, "Destructive", destructive_content);

    let rtl_dialog = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            build_dialog(
                cx,
                "ui-gallery-alert-dialog-rtl",
                rtl_open,
                "????? ??????",
                shadcn::ButtonVariant::Outline,
                "?? ??? ????? ???????",
                "?? ???? ??????? ?? ??? ???????. ????? ??? ??? ??? ?????? ??????? ?? ???????.",
                "?????",
                "??????",
                shadcn::ButtonVariant::Default,
                Px(520.0),
                None,
                false,
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_dialog);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Alert Dialog docs order and keeps each state in a separate section for quick lookup.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                basic,
                small,
                media,
                small_with_media,
                destructive,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).attach_semantics(
        SemanticsDecoration::default().test_id("ui-gallery-alert-dialog-component"),
    );

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Demo / Basic",
                    r#"AlertDialog::new(open).into_element(
    cx,
    |cx| Button::new("Show Dialog").toggle_model(open.clone()).into_element(cx),
    |cx| AlertDialogContent::new([header, footer]).into_element(cx),
)"#,
                ),
                code_block(
                    cx,
                    "Small + Media",
                    r#"AlertDialogContent::new([...])
    .refine_layout(LayoutRefinement::default().max_w(Px(400.0)))
// current Fret API has no size or AlertDialogMedia prop; use layout + inline media chip"#,
                ),
                code_block(
                    cx,
                    "Destructive + RTL",
                    r#"AlertDialogAction::new("Delete", open).variant(ButtonVariant::Destructive)
with_direction_provider(LayoutDirection::Rtl, |cx| ...)"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Alert Dialog is modal by default and should be reserved for destructive or irreversible decisions.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current API does not expose `size` or `AlertDialogMedia`; this page approximates those examples via `refine_layout(max_w)` and an inline media chip.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use `AlertDialogCancel` + `AlertDialogAction` with the same open model to guarantee close behavior stays predictable.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep dialog copy concise and explicit, and ensure destructive actions have clear labels and visual hierarchy.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-alert-dialog",
        component_panel,
        code_panel,
        notes_panel,
    )
}
