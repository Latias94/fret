pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_ui::action::OnActivate;
use fret_ui_kit::{ColorRef, WidgetStateProperty};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[rustfmt::skip]
pub fn render(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> impl UiChild + use<> {
    let dropdown = material3::DropdownMenu::uncontrolled(cx)
        .a11y_label("Material 3 Menu")
        .test_id("ui-gallery-material3-menu");
    let open = dropdown.open_model();
    let override_open = cx.local_model_keyed("override_open", || false);

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
            theme.color_token("md.sys.color.secondary-container"),
        ))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
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

    ui::v_flex(|cx| {
            vec![
                cx.text("Tip: Arrow keys / Home / End navigate; type to jump by prefix; Esc/outside press closes."),
                ui::h_row(move |_cx| vec![card_default, card_override]).gap(Space::N4).items_center().into_element(cx),
                cx.text(format!("last action: {last}")),
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
    .into()
}

// endregion: example
