use super::super::super::*;

pub(in crate::ui) fn preview_material3_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};

    #[derive(Default)]
    struct DialogPageModels {
        override_open: Option<Model<bool>>,
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let override_open = cx.with_state(DialogPageModels::default, |st| st.override_open.clone());
    let override_open = match override_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DialogPageModels::default, |st| {
                st.override_open = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(DialogPageModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(DialogPageModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let override_is_open = cx
        .get_model_copied(&override_open, Invalidation::Layout)
        .unwrap_or(false);

    let open_dialog: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action: OnActivate = {
        let open = open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm")
            });
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let theme = cx.theme().clone();
    let select_items: Arc<[material3::SelectItem]> = (0..20)
        .map(|i| {
            material3::SelectItem::new(
                Arc::<str>::from(format!("item-{i:02}")),
                Arc::<str>::from(format!("Item {i:02}")),
            )
            .test_id(format!("ui-gallery-material3-dialog-select-item-{i:02}"))
        })
        .collect::<Vec<_>>()
        .into();
    let override_style = material3::DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.secondary-container"),
        ))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))));

    let open_dialog_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog_override: OnActivate = {
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action_override: OnActivate = {
        let override_open = override_open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm.override")
            });
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let build_dialog = |cx: &mut ElementContext<'_, App>,
                        open_model: Model<bool>,
                        style: Option<material3::DialogStyle>,
                        id_prefix: &'static str,
                        open_action: OnActivate,
                        close_action: OnActivate,
                        confirm_action: OnActivate|
     -> AnyElement {
        let mut dialog = material3::Dialog::new(open_model.clone())
            .headline("Discard draft?")
            .supporting_text("This action cannot be undone.")
            .actions(vec![
                material3::DialogAction::new("Cancel")
                    .test_id(format!("{id_prefix}-action-cancel"))
                    .on_activate(close_action.clone()),
                material3::DialogAction::new("Discard")
                    .test_id(format!("{id_prefix}-action-discard"))
                    .on_activate(confirm_action.clone()),
            ])
            .test_id(format!("{id_prefix}"));

        if let Some(style) = style {
            dialog = dialog.style(style);
        }

        dialog.into_element(
            cx,
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open dialog")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_action.clone())
                                .test_id(format!("{id_prefix}-open"))
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id(format!("{id_prefix}-underlay-probe"))
                                .into_element(cx),
                            cx.text("Tip: press Esc or click the scrim to close; Tab should stay inside the dialog while open."),
                        ]
                    },
                )
            },
            {
                let selected = selected.clone();
                let select_items = select_items.clone();
                move |cx| {
                    let spacer = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Px(Px(480.0));
                                l
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    );

                    vec![stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4),
                        move |cx| {
                            vec![
                                material3::Select::new(selected.clone())
                                    .a11y_label("Material 3 Select (dialog)")
                                    .placeholder("Pick one")
                                    .items(select_items.clone())
                                    .match_anchor_width(false)
                                    .test_id(format!("{id_prefix}-select"))
                                    .into_element(cx),
                                cx.text(
                                    "Bottom-edge clamping probe: open the Select menu near the window bottom.",
                                ),
                                spacer,
                                material3::Select::new(selected.clone())
                                    .a11y_label("Material 3 Select (dialog, bottom)")
                                    .placeholder("Pick one")
                                    .items(select_items.clone())
                                    .match_anchor_width(false)
                                    .test_id(format!("{id_prefix}-select-bottom"))
                                    .into_element(cx),
                            ]
                        },
                    )]
                }
            },
        )
    };

    let default_dialog = build_dialog(
        cx,
        open.clone(),
        None,
        "ui-gallery-material3-dialog",
        open_dialog.clone(),
        close_dialog.clone(),
        confirm_action.clone(),
    );
    let override_dialog = build_dialog(
        cx,
        override_open.clone(),
        Some(override_style),
        "ui-gallery-material3-dialog-override",
        open_dialog_override.clone(),
        close_dialog_override.clone(),
        confirm_action_override.clone(),
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_container = |cx: &mut ElementContext<'_, App>, dialog: AnyElement| -> AnyElement {
        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Px(Px(360.0));
        cx.container(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| [dialog],
        )
    };

    let containers = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                build_container(cx, default_dialog),
                build_container(cx, override_dialog),
            ]
        },
    );

    vec![
        cx.text(
            "Material 3 Dialog: modal barrier + focus trap/restore + token-shaped dialog actions.",
        ),
        containers,
        cx.text(format!(
            "open={} override_open={} last_action={}",
            is_open as u8,
            override_is_open as u8,
            last.as_ref()
        )),
    ]
}

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};

    #[derive(Default)]
    struct MenuPageModels {
        override_open: Option<Model<bool>>,
    }

    let override_open = cx.with_state(MenuPageModels::default, |st| st.override_open.clone());
    let override_open = match override_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MenuPageModels::default, |st| {
                st.override_open = Some(model.clone())
            });
            model
        }
    };

    fn on_select(id: &'static str, last_action: Model<Arc<str>>) -> OnActivate {
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from(id);
            });
            host.request_redraw(action_cx.window);
        })
    }

    let toggle_open: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let toggle_open_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        })
    };

    let last_action_for_entries = last_action.clone();
    let dropdown = material3::DropdownMenu::new(open.clone())
        .a11y_label("Material 3 Menu")
        .test_id("ui-gallery-material3-menu")
        .into_element(
            cx,
            move |cx| {
                material3::Button::new("Open menu")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(toggle_open.clone())
                    .test_id("ui-gallery-material3-menu-trigger")
                    .into_element(cx)
            },
            move |_cx| {
                vec![
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Cut")
                            .test_id("ui-gallery-material3-menu-item-cut")
                            .on_select(on_select(
                                "material3.menu.cut",
                                last_action_for_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Copy")
                            .test_id("ui-gallery-material3-menu-item-copy")
                            .on_select(on_select(
                                "material3.menu.copy",
                                last_action_for_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Paste")
                            .test_id("ui-gallery-material3-menu-item-paste")
                            .disabled(true),
                    ),
                    material3::MenuEntry::Separator,
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Settings")
                            .test_id("ui-gallery-material3-menu-item-settings")
                            .on_select(on_select(
                                "material3.menu.settings",
                                last_action_for_entries.clone(),
                            )),
                    ),
                ]
            },
        );

    let theme = cx.theme().clone();
    let override_style = material3::MenuStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.secondary-container"),
        ))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))))
        .item_state_layer_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))));

    let last_action_for_override_entries = last_action.clone();
    let dropdown_override = material3::DropdownMenu::new(override_open.clone())
        .a11y_label("Material 3 Menu (override)")
        .test_id("ui-gallery-material3-menu-override")
        .menu_style(override_style)
        .into_element(
            cx,
            move |cx| {
                material3::Button::new("Open menu (override)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(toggle_open_override.clone())
                    .test_id("ui-gallery-material3-menu-trigger-override")
                    .into_element(cx)
            },
            move |_cx| {
                vec![
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Cut")
                            .test_id("ui-gallery-material3-menu-item-cut-override")
                            .on_select(on_select(
                                "material3.menu.cut.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Copy")
                            .test_id("ui-gallery-material3-menu-item-copy-override")
                            .on_select(on_select(
                                "material3.menu.copy.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Paste")
                            .test_id("ui-gallery-material3-menu-item-paste-override")
                            .disabled(true),
                    ),
                    material3::MenuEntry::Separator,
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Settings")
                            .test_id("ui-gallery-material3-menu-item-settings-override")
                            .on_select(on_select(
                                "material3.menu.settings.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                ]
            },
        );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let card_default = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Default").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![dropdown]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let card_override = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Override").into_element(cx),
            shadcn::CardDescription::new(
                "ADR 0220: MenuStyle overrides (container + item colors).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![dropdown_override]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    vec![
        cx.text("Tip: Arrow keys / Home / End navigate; type to jump by prefix; Esc/outside press closes."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |_cx| vec![card_default, card_override],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

pub(in crate::ui) fn preview_material3_list(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_list = |cx: &mut ElementContext<'_, App>, id_prefix: &str| -> AnyElement {
        material3::List::new(value.clone())
            .a11y_label("Material 3 List")
            .test_id(format!("{id_prefix}-list"))
            .items(vec![
                material3::ListItem::new("alpha", "Alpha")
                    .leading_icon(ids::ui::SEARCH)
                    .a11y_label("List item alpha")
                    .test_id(format!("{id_prefix}-list-item-alpha")),
                material3::ListItem::new("beta", "Beta")
                    .leading_icon(ids::ui::SETTINGS)
                    .a11y_label("List item beta")
                    .test_id(format!("{id_prefix}-list-item-beta")),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true)
                    .a11y_label("List item disabled")
                    .test_id(format!("{id_prefix}-list-item-disabled")),
            ])
            .into_element(cx)
    };

    let standard = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Standard").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![build_list(cx, "ui-gallery-material3-standard")])
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let expressive = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Expressive").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            |cx| build_list(cx, "ui-gallery-material3-expressive"),
        )])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let variants = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        move |_cx| [standard, expressive],
    );

    vec![
        cx.text("Material 3 List: roving focus (Up/Down/Home/End) + selection follows focus."),
        cx.text("Compare Standard vs Expressive via subtree override (shape + icon size)."),
        variants,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_snackbar(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_runtime::CommandId;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::ToastStore;

    #[derive(Default)]
    struct State {
        store: Option<Model<ToastStore>>,
    }

    let store = cx.with_state(State::default, |st| st.store.clone());
    let store = store.unwrap_or_else(|| {
        let store = cx.app.models_mut().insert(ToastStore::default());
        cx.with_state(State::default, |st| st.store = Some(store.clone()));
        store
    });

    let host_layer = material3::SnackbarHost::new(store.clone())
        .max_snackbars(1)
        .into_element(cx);

    let show_short: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Saved").action("Undo", CommandId::new(CMD_TOAST_ACTION)),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_two_line: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Update available")
                    .supporting_text("Restart the app to apply the latest changes.")
                    .action("Restart", CommandId::new(CMD_TOAST_ACTION))
                    .duration(material3::SnackbarDuration::Long),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_indefinite: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Connection lost")
                    .supporting_text("Trying to reconnect...")
                    .duration(material3::SnackbarDuration::Indefinite),
            );
            host.request_redraw(acx.window);
        })
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                material3::Button::new("Show (short)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_short.clone())
                    .test_id("ui-gallery-material3-snackbar-show-short")
                    .into_element(cx),
                material3::Button::new("Show (two-line)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_two_line.clone())
                    .test_id("ui-gallery-material3-snackbar-show-two-line")
                    .into_element(cx),
                material3::Button::new("Show (indefinite)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_indefinite.clone())
                    .test_id("ui-gallery-material3-snackbar-show-indefinite")
                    .into_element(cx),
            ]
        },
    );

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Snackbar").into_element(cx),
            shadcn::CardDescription::new(
                "Snackbar MVP: Material token-driven toast-layer skin (md.comp.snackbar.*).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            host_layer,
            buttons,
            cx.text(format!("last action: {last}")),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}

pub(in crate::ui) fn preview_material3_tooltip(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let content = material3::TooltipProvider::new().with_elements(cx, |cx| {
        let outlined = material3::ButtonVariant::Outlined;

        let top = material3::PlainTooltip::new(
            material3::Button::new("Hover (Top)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-top-trigger")
                .into_element(cx),
            "Plain tooltip (top)",
        )
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let right = material3::PlainTooltip::new(
            material3::Button::new("Hover (Right)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-right-trigger")
                .into_element(cx),
            "Plain tooltip (right)",
        )
        .side(material3::TooltipSide::Right)
        .into_element(cx);

        let bottom = material3::PlainTooltip::new(
            material3::Button::new("Hover (Bottom)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-bottom-trigger")
                .into_element(cx),
            "Plain tooltip (bottom)",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        let left = material3::PlainTooltip::new(
            material3::Button::new("Hover (Left)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-left-trigger")
                .into_element(cx),
            "Plain tooltip (left)",
        )
        .side(material3::TooltipSide::Left)
        .into_element(cx);

        let rich = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-trigger")
                .into_element(cx),
            "Rich tooltip supporting text (body medium).",
        )
        .title("Rich tooltip title")
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let rich_no_title = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich / no title)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-no-title-trigger")
                .into_element(cx),
            "Rich tooltip supporting text only.",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [top, right, bottom, left],
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [rich, rich_no_title],
                ),
                cx.text("Note: Tooltip open delay is controlled via Material3 TooltipProvider (delay-group)."),
            ]
    });

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Tooltip").into_element(cx),
            shadcn::CardDescription::new(
                "Tooltip MVP: delay group + hover intent + safe-hover corridor + token-driven styling (plain + rich).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(content).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}
