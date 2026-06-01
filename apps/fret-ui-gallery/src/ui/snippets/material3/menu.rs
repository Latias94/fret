pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_icons::ids;
use fret_ui::action::OnActivate;
use fret_ui_kit::{ColorRef, WidgetStateProperty};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[rustfmt::skip]
pub fn render(
    cx: &mut AppComponentCx<'_>,
    last_action: Model<Arc<str>>,
) -> impl UiChild + use<> {
    let dropdown = material3::DropdownMenu::uncontrolled(cx)
        .a11y_label("Material 3 Menu")
        .test_id("ui-gallery-material3-menu");
    let open = dropdown.open_model();
    let override_open = cx.local_model_keyed("override_open", || false);
    let search_menu_selected = cx.local_model_keyed("search_menu_selected", || Arc::<str>::from("alpha"));
    let show_toolbar = cx.local_model_keyed("show_toolbar", || true);
    let density = cx.local_model_keyed("density", || {
        Some(Arc::<str>::from("comfortable"))
    });

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

    let search_menu_dropdown = material3::DropdownMenu::uncontrolled(cx)
        .a11y_label("Material 3 Search actions menu")
        .test_id("ui-gallery-material3-menu-search-actions");
    let search_menu_open = search_menu_dropdown.open_model();
    let toggle_open_search_menu: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        let search_menu_open = search_menu_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            let _ = host.models_mut().update(&search_menu_open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        })
    };

    let last_action_for_entries = last_action.clone();
    let show_toolbar_for_entries = show_toolbar.clone();
    let density_for_entries = density.clone();
    let dropdown = dropdown.into_element(
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
                material3::MenuEntry::Label(
                    material3::MenuLabel::new("Edit")
                        .test_id("ui-gallery-material3-menu-label-edit"),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Cut")
                        .leading_icon(ids::ui::SLASH)
                        .shortcut("Ctrl+X")
                        .test_id("ui-gallery-material3-menu-item-cut")
                        .on_select(on_select(
                            "material3.menu.cut",
                            last_action_for_entries.clone(),
                        )),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Copy")
                        .leading_icon(ids::ui::COPY)
                        .shortcut("Ctrl+C")
                        .test_id("ui-gallery-material3-menu-item-copy")
                        .on_select(on_select(
                            "material3.menu.copy",
                            last_action_for_entries.clone(),
                        )),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Paste")
                        .supporting_text("Clipboard is empty")
                        .shortcut("Ctrl+V")
                        .test_id("ui-gallery-material3-menu-item-paste")
                        .disabled(true),
                ),
                material3::MenuEntry::Separator,
                material3::MenuEntry::Label(
                    material3::MenuLabel::new("View")
                        .test_id("ui-gallery-material3-menu-label-view"),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::checkbox(show_toolbar_for_entries.clone(), "Show toolbar")
                        .supporting_text("Keep editor tools visible")
                        .shortcut("Ctrl+B")
                        .test_id("ui-gallery-material3-menu-item-toolbar")
                        .on_select(on_select(
                            "material3.menu.toolbar",
                            last_action_for_entries.clone(),
                        )),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::radio(
                        density_for_entries.clone(),
                        "comfortable",
                        "Comfortable density",
                    )
                    .test_id("ui-gallery-material3-menu-item-density-comfortable")
                    .on_select(on_select(
                        "material3.menu.density.comfortable",
                        last_action_for_entries.clone(),
                    )),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::radio(density_for_entries, "compact", "Compact density")
                        .test_id("ui-gallery-material3-menu-item-density-compact")
                        .on_select(on_select(
                            "material3.menu.density.compact",
                            last_action_for_entries.clone(),
                        )),
                ),
                material3::MenuEntry::Separator,
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Settings")
                        .trailing_icon(ids::ui::CHEVRON_RIGHT)
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
            theme.color_token("md.sys.color.secondary-container"),
        ))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))))
        .item_icon_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))))
        .item_supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))))
        .item_trailing_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))))
        .section_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))))
        .item_state_layer_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
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
                    material3::MenuEntry::Label(
                        material3::MenuLabel::new("Override")
                            .test_id("ui-gallery-material3-menu-label-override"),
                    ),
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

    let search_menu_suggestions = material3::List::new(search_menu_selected)
        .a11y_label("Search suggestions")
        .test_id("ui-gallery-material3-menu-search-suggestions")
        .items(vec![
            material3::ListItem::new("alpha", "Alpha")
                .leading_icon(ids::ui::SEARCH)
                .test_id("ui-gallery-material3-menu-search-option-alpha"),
            material3::ListItem::new("beta", "Beta")
                .leading_icon(ids::ui::SEARCH)
                .test_id("ui-gallery-material3-menu-search-option-beta"),
            material3::ListItem::new("gamma", "Gamma")
                .leading_icon(ids::ui::SEARCH)
                .test_id("ui-gallery-material3-menu-search-option-gamma"),
        ])
        .into_element(cx);

    let search_menu_view = material3::SearchView::uncontrolled(cx)
        .leading_icon(ids::ui::SEARCH)
        .trailing_icon(ids::ui::CLOSE)
        .placeholder("Search actions")
        .a11y_label("Search actions")
        .test_id("ui-gallery-material3-menu-search")
        .overlay_test_id("ui-gallery-material3-menu-search-panel")
        .into_element(cx, |_cx| vec![search_menu_suggestions]);

    let last_action_for_search_menu_entries = last_action.clone();
    let search_menu_dropdown = search_menu_dropdown.into_element(
        cx,
        move |cx| {
            material3::Button::new("Actions")
                .variant(material3::ButtonVariant::Outlined)
                .on_activate(toggle_open_search_menu.clone())
                .test_id("ui-gallery-material3-menu-search-actions-trigger")
                .into_element(cx)
        },
        move |_cx| {
            vec![
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Filter Alpha")
                        .test_id("ui-gallery-material3-menu-search-actions-alpha")
                        .on_select(on_select(
                            "material3.menu.search.alpha",
                            last_action_for_search_menu_entries.clone(),
                        )),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Clear search")
                        .test_id("ui-gallery-material3-menu-search-actions-clear")
                        .on_select(on_select(
                            "material3.menu.search.clear",
                            last_action_for_search_menu_entries.clone(),
                        )),
                ),
            ]
        },
    );

    let search_menu_view = ui::v_stack(move |_cx| vec![search_menu_view])
        .layout(LayoutRefinement::default().w_px(Px(420.0)).min_w_0())
        .into_element(cx);
    let search_menu_row = ui::h_row(move |_cx| vec![search_menu_view, search_menu_dropdown])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N3)
        .items_center()
        .into_element(cx);

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let card_default = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Default"),
                    shadcn::card_description(
                        "Default root owns its open state via `DropdownMenu::uncontrolled(cx)`.",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![dropdown]),
        ]
    })
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let card_override = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Override"),
                    shadcn::card_description(
                        "ADR 0220: MenuStyle overrides (container + item colors).",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![dropdown_override]),
        ]
    })
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let card_search_menu = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Search + Menu"),
                    shadcn::card_description(
                        "Sibling non-modal overlays: the SearchView closes when the actions menu opens.",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![search_menu_row]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    ui::v_flex(|cx| {
            vec![
                cx.text("Tip: Arrow keys / Home / End navigate; type to jump by prefix; Esc/outside press closes."),
                ui::h_row(move |_cx| vec![card_default, card_override]).gap(Space::N4).items_center().into_element(cx),
                card_search_menu,
                cx.text(format!("last action: {last}")),
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
}

// endregion: example
