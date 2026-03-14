pub const SOURCE: &str = include_str!("state_matrix.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::{Edges, Px};
use fret_icons::ids;
use fret_ui::action::OnActivate;
use fret_ui::element::{ContainerProps, Length, TextProps};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn render_chips(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let filter_selected = cx.local_model_keyed("filter_selected", || true);
    let filter_unselected = cx.local_model_keyed("filter_unselected", || false);
    let input_selected = cx.local_model_keyed("input_selected", || true);
    let input_unselected = cx.local_model_keyed("input_unselected", || false);

    let last_action_for_activate = last_action.clone();
    let activate: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&last_action_for_activate, |v| {
            *v = Arc::<str>::from("material3.assist_chip.activated");
        });
    });

    let (hover_container, hover_label) = cx.with_theme(|theme| {
        (
            theme.color_token("md.sys.color.tertiary-container"),
            theme.color_token("md.sys.color.on-tertiary-container"),
        )
    });
    let accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);

    let override_style = material3::AssistChipStyle::default()
        .label_color(WidgetStateProperty::new(Some(ColorRef::Color(accent))))
        .state_layer_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        )
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        )
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ));

    let hover_style = material3::AssistChipStyle::default()
        .label_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_label))),
        )
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ));

    let filter_override_style = material3::FilterChipStyle::default()
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(hover_container)),
        ))
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        );

    let activate_row1 = activate.clone();
    let activate_row2 = activate.clone();
    let activate_row3 = activate.clone();
    let activate_row4 = activate.clone();

    let last_action_for_input_selected = last_action.clone();
    let activate_input_selected_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_selected, |v| {
                *v = Arc::<str>::from("material3.input_chip.primary.activated");
            });
    });

    let last_action_for_input_unselected = last_action.clone();
    let activate_input_unselected_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_unselected, |v| {
                *v = Arc::<str>::from("material3.input_chip.primary.activated");
            });
    });

    let last_action_for_input_unselected_trailing = last_action.clone();
    let activate_input_unselected_trailing: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_unselected_trailing, |v| {
                *v = Arc::<str>::from("material3.input_chip.trailing_icon.activated");
            });
    });

    let last_action_for_filter_primary = last_action.clone();
    let activate_filter_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_filter_primary, |v| {
                *v = Arc::<str>::from("material3.filter_chip.primary.activated");
            });
    });

    let last_action_for_filter_trailing = last_action.clone();
    let activate_filter_trailing: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_filter_trailing, |v| {
                *v = Arc::<str>::from("material3.filter_chip.trailing_icon.activated");
            });
    });

    let activate_filter_primary_row = activate_filter_primary.clone();
    let activate_filter_trailing_row = activate_filter_trailing.clone();
    let activate_filter_primary_for_set = activate_filter_primary.clone();
    let activate_filter_trailing_for_set = activate_filter_trailing.clone();

    let activate_input_unselected_primary_row = activate_input_unselected_primary.clone();
    let activate_input_unselected_trailing_row = activate_input_unselected_trailing.clone();
    let activate_input_unselected_primary_for_set = activate_input_unselected_primary.clone();
    let activate_input_unselected_trailing_for_set = activate_input_unselected_trailing.clone();

    let filter_selected_row = filter_selected.clone();
    let filter_unselected_row = filter_unselected.clone();
    let filter_selected_row2 = filter_selected.clone();
    let filter_unselected_row2 = filter_unselected.clone();
    let filter_selected_for_set = filter_selected.clone();

    let input_selected_row = input_selected.clone();
    let input_unselected_row = input_unselected.clone();
    let input_unselected_row2 = input_unselected.clone();
    let input_unselected_for_set = input_unselected.clone();

    let override_style_row1 = override_style.clone();
    let override_style_row2 = override_style.clone();
    let hover_style_row1 = hover_style.clone();
    let hover_style_row2 = hover_style.clone();
    let filter_override_style_row = filter_override_style.clone();

    vec![
        cx.text("Material 3 Chips: assist + suggestion + filter + input + chip-set roving focus."),
        ui::h_row(move |cx| {
                vec![
                    material3::AssistChip::new("Flat")
                        .on_activate(activate_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat")
                        .into_element(cx),
                    material3::AssistChip::new("Override")
                        .on_activate(activate_row1.clone())
                        .style(override_style_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat-override")
                        .into_element(cx),
                    material3::AssistChip::new("Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-chip-flat-disabled")
                        .into_element(cx),
                    material3::AssistChip::new("Hover Override")
                        .on_activate(activate_row1.clone())
                        .style(hover_style_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat-hover-override")
                        .into_element(cx),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
        ui::h_row(move |cx| {
                vec![
                    material3::AssistChip::new("Elevated")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated")
                        .into_element(cx),
                    material3::AssistChip::new("Override")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .style(override_style_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated-override")
                        .into_element(cx),
                    material3::AssistChip::new("Disabled")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SLASH)
                        .disabled(true)
                        .test_id("ui-gallery-material3-chip-elevated-disabled")
                        .into_element(cx),
                    material3::AssistChip::new("Hover Override")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .style(hover_style_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated-hover-override")
                        .into_element(cx),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
        ui::h_row(move |cx| {
                vec![
                    material3::SuggestionChip::new("Suggestion")
                        .on_activate(activate_row3.clone())
                        .test_id("ui-gallery-material3-suggestion-chip-flat")
                        .into_element(cx),
                    material3::SuggestionChip::new("Suggestion (icon)")
                        .leading_icon(ids::ui::SEARCH)
                        .variant(material3::SuggestionChipVariant::Elevated)
                        .on_activate(activate_row3.clone())
                        .test_id("ui-gallery-material3-suggestion-chip-elevated")
                        .into_element(cx),
                    material3::SuggestionChip::new("Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-suggestion-chip-disabled")
                        .into_element(cx),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
        ui::h_row(move |cx| {
                vec![
                    material3::FilterChip::new(filter_selected_row.clone(), "Filter")
                        .trailing_icon(ids::ui::CLOSE)
                        .on_activate(activate_filter_primary_row.clone())
                        .on_trailing_icon_activate(activate_filter_trailing_row.clone())
                        .test_id("ui-gallery-material3-filter-chip-selected")
                        .into_element(cx),
                    material3::FilterChip::new(filter_unselected_row.clone(), "Filter")
                        .on_activate(activate_filter_primary_row.clone())
                        .test_id("ui-gallery-material3-filter-chip-unselected")
                        .into_element(cx),
                    material3::FilterChip::new(filter_selected_row2.clone(), "Override")
                        .variant(material3::FilterChipVariant::Elevated)
                        .style(filter_override_style_row.clone())
                        .on_activate(activate_filter_primary_row.clone())
                        .test_id("ui-gallery-material3-filter-chip-override")
                        .into_element(cx),
                    material3::FilterChip::new(filter_unselected_row2.clone(), "Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-filter-chip-disabled")
                        .into_element(cx),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
        ui::h_row(move |cx| {
                vec![
                    material3::InputChip::new(input_selected_row.clone(), "Input")
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_input_selected_primary.clone())
                        .test_id("ui-gallery-material3-input-chip-selected")
                        .into_element(cx),
                    material3::InputChip::new(input_unselected_row.clone(), "Input")
                        .trailing_icon(ids::ui::CLOSE)
                        .on_activate(activate_input_unselected_primary_row.clone())
                        .on_trailing_icon_activate(activate_input_unselected_trailing_row.clone())
                        .test_id("ui-gallery-material3-input-chip-unselected")
                        .into_element(cx),
                    material3::InputChip::new(input_unselected_row2.clone(), "Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-input-chip-disabled")
                        .into_element(cx),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
        cx.text(
            "Material 3 ChipSet: ArrowLeft/Right + Home/End roving focus. Multi-action chips use ArrowLeft/Right to move focus between primary/trailing actions.",
        ),
        material3::ChipSet::new(vec![
            material3::ChipSetItem::from(
                material3::AssistChip::new("Assist")
                    .leading_icon(ids::ui::SETTINGS)
                    .on_activate(activate_row4.clone())
                    .test_id("ui-gallery-material3-chip-set-assist"),
            ),
            material3::ChipSetItem::from(
                material3::SuggestionChip::new("Suggestion")
                    .leading_icon(ids::ui::SEARCH)
                    .on_activate(activate_row4.clone())
                    .test_id("ui-gallery-material3-chip-set-suggestion"),
            ),
            material3::ChipSetItem::from(
                material3::FilterChip::new(filter_selected_for_set, "Filter")
                    .trailing_icon(ids::ui::CLOSE)
                    .on_activate(activate_filter_primary_for_set.clone())
                    .on_trailing_icon_activate(activate_filter_trailing_for_set.clone())
                    .test_id("ui-gallery-material3-chip-set-filter"),
            ),
            material3::ChipSetItem::from(
                material3::InputChip::new(input_unselected_for_set, "Input")
                    .trailing_icon(ids::ui::CLOSE)
                    .on_activate(activate_input_unselected_primary_for_set.clone())
                    .on_trailing_icon_activate(activate_input_unselected_trailing_for_set.clone())
                    .test_id("ui-gallery-material3-chip-set-input"),
            ),
        ])
        .a11y_label("chip set")
        .test_id("ui-gallery-material3-chip-set")
        .into_element(cx),
    ]
}

fn render_cards(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let activate: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&last_action, |v| {
            *v = Arc::<str>::from("material3.card.activated");
        });
    });

    let (body_style, body_color, hover_container, hover_outline) = cx.with_theme(|theme| {
        let body_style = theme
            .text_style_by_key("md.sys.typescale.body-medium")
            .unwrap_or_else(|| fret_core::TextStyle::default());
        let body_color = theme.color_token("md.sys.color.on-surface");
        let hover_container = theme.color_token("md.sys.color.tertiary-container");
        let hover_outline = theme.color_token("md.sys.color.tertiary");
        (body_style, body_color, hover_container, hover_outline)
    });

    let override_style = material3::CardStyle::default()
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ))
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_outline))),
        );

    let card_content_row1 = {
        let body_style = body_style.clone();
        let body_color = body_color;
        move |cx: &mut UiCx<'_>, label: &'static str| {
            let mut container = ContainerProps::default();
            container.layout.size.width = Length::Px(Px(280.0));
            container.layout.size.height = Length::Px(Px(72.0));
            container.padding = Edges::all(Px(12.0)).into();

            let mut text = TextProps::new(Arc::<str>::from(label));
            text.style = Some(body_style.clone());
            text.color = Some(body_color);
            cx.container(container, move |cx| vec![cx.text_props(text)])
        }
    };

    let card_content_row2 = {
        let body_style = body_style.clone();
        let body_color = body_color;
        move |cx: &mut UiCx<'_>, label: &'static str| {
            let mut container = ContainerProps::default();
            container.layout.size.width = Length::Px(Px(280.0));
            container.layout.size.height = Length::Px(Px(72.0));
            container.padding = Edges::all(Px(12.0)).into();

            let mut text = TextProps::new(Arc::<str>::from(label));
            text.style = Some(body_style.clone());
            text.color = Some(body_color);
            cx.container(container, move |cx| vec![cx.text_props(text)])
        }
    };

    let activate_row1 = activate.clone();
    let activate_row2 = activate.clone();
    let override_style_row1 = override_style.clone();
    let override_style_row2 = override_style.clone();

    vec![
        cx.text(
            "Material 3 Card: token-driven surface + outline + ink (interactive only when on_activate is set).",
        ),
        ui::h_row(move |cx| {
                vec![
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .test_id("ui-gallery-material3-card-filled")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Filled")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .style(override_style_row1.clone())
                        .test_id("ui-gallery-material3-card-filled-override")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Override")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .disabled(true)
                        .test_id("ui-gallery-material3-card-filled-disabled")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Disabled")]),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
        ui::h_row(move |cx| {
                vec![
                    material3::Card::new()
                        .variant(material3::CardVariant::Elevated)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-card-elevated")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Elevated")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Outlined)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-card-outlined")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Outlined")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Outlined)
                        .on_activate(activate_row2.clone())
                        .style(override_style_row2.clone())
                        .test_id("ui-gallery-material3-card-outlined-override")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Outline override")]),
                ]
            }).gap(Space::N2).items_center().into_element(cx),
    ]
}

fn render_fab(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    fn on_activate(id: &'static str, last_action: Model<Arc<str>>) -> OnActivate {
        Arc::new(move |host, _acx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from(id);
            });
        })
    }

    let row = {
        let last_action = last_action.clone();
        move |cx: &mut UiCx<'_>,
              variant: material3::FabVariant,
              label: &'static str| {
            let last_action = last_action.clone();
            ui::h_row(move |cx| {
                vec![
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(variant)
                        .a11y_label(label)
                        .on_activate(on_activate("material3.fab.activated", last_action.clone()))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(variant)
                        .a11y_label("Small")
                        .size(material3::FabSize::Small)
                        .on_activate(on_activate(
                            "material3.fab.small.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(variant)
                        .a11y_label("Large")
                        .size(material3::FabSize::Large)
                        .on_activate(on_activate(
                            "material3.fab.large.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(variant)
                        .a11y_label("Lowered")
                        .lowered(true)
                        .on_activate(on_activate(
                            "material3.fab.lowered.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(variant)
                        .a11y_label("Disabled")
                        .disabled(true)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx)
        }
    };

    let extended = {
        let last_action = last_action.clone();
        ui::h_row(move |cx| {
            vec![
                material3::Fab::new(ids::ui::SEARCH)
                    .variant(material3::FabVariant::Surface)
                    .label("Create")
                    .on_activate(on_activate(
                        "material3.extended_fab.activated",
                        last_action.clone(),
                    ))
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .variant(material3::FabVariant::Primary)
                    .label("Create")
                    .on_activate(on_activate(
                        "material3.extended_fab.primary.activated",
                        last_action.clone(),
                    ))
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .variant(material3::FabVariant::Surface)
                    .label("Reroute")
                    .icon(None)
                    .on_activate(on_activate(
                        "material3.extended_fab.no_icon.activated",
                        last_action.clone(),
                    ))
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx)
    };

    vec![
        cx.text(
            "Material 3 FAB: token-driven variants + focus ring + state layer + bounded ripple.",
        ),
        row(cx, material3::FabVariant::Surface, "Surface"),
        row(cx, material3::FabVariant::Primary, "Primary"),
        row(cx, material3::FabVariant::Secondary, "Secondary"),
        row(cx, material3::FabVariant::Tertiary, "Tertiary"),
        extended,
    ]
}

fn render_search_view(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let selected = cx.local_model_keyed("selected", || Arc::<str>::from("alpha"));

    let suggestions = material3::List::new(selected)
        .a11y_label("Suggestions")
        .test_id("ui-gallery-material3-search-view-suggestions")
        .items(vec![
            material3::ListItem::new("alpha", "Alpha")
                .leading_icon(ids::ui::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-alpha"),
            material3::ListItem::new("bravo", "Bravo")
                .leading_icon(ids::ui::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-bravo"),
            material3::ListItem::new("charlie", "Charlie")
                .leading_icon(ids::ui::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-charlie"),
        ])
        .into_element(cx);

    let view = material3::SearchView::uncontrolled(cx)
        .leading_icon(ids::ui::SEARCH)
        .trailing_icon(ids::ui::CLOSE)
        .placeholder("Search")
        .a11y_label("Search")
        .test_id("ui-gallery-material3-search-view")
        .overlay_test_id("ui-gallery-material3-search-view-panel")
        .into_element(cx, |_cx| vec![suggestions]);

    vec![view]
}

fn render_menu(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown_root = material3::DropdownMenu::uncontrolled(cx).a11y_label("Material 3 Menu");
    let open = dropdown_root.open_model();

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
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        })
    };

    let dropdown = dropdown_root.into_element(
        cx,
        move |cx| {
            material3::Button::new("Open menu")
                .variant(material3::ButtonVariant::Outlined)
                .on_activate(toggle_open.clone())
                .into_element(cx)
        },
        move |_cx| {
            vec![
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Cut")
                        .on_select(on_select("material3.menu.cut", last_action.clone())),
                ),
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Copy")
                        .on_select(on_select("material3.menu.copy", last_action.clone())),
                ),
                material3::MenuEntry::Item(material3::MenuItem::new("Paste").disabled(true)),
                material3::MenuEntry::Separator,
                material3::MenuEntry::Item(
                    material3::MenuItem::new("Settings")
                        .on_select(on_select("material3.menu.settings", last_action)),
                ),
            ]
        },
    );

    vec![dropdown]
}

pub fn render(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> impl UiChild + use<> {
    let checkbox_root = material3::Checkbox::uncontrolled(cx, false);
    let material3_checkbox = checkbox_root.checked_model();
    let switch_root = material3::Switch::uncontrolled(cx, false);
    let material3_switch = switch_root.selected_model();
    let radio_group_root = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);
    let material3_radio_value = radio_group_root.value_model();
    let tabs_root = material3::Tabs::uncontrolled(cx, "overview");
    let material3_tabs_value = tabs_root.value_model();
    let navigation_bar_root = material3::NavigationBar::uncontrolled(cx, "search");
    let material3_navigation_bar_value = navigation_bar_root.value_model();
    let text_field_root = material3::TextField::uncontrolled(cx);
    let material3_text_field_value = text_field_root.value_model();
    let text_field_disabled_root = material3::Switch::uncontrolled(cx, false);
    let material3_text_field_disabled = text_field_disabled_root.selected_model();
    let text_field_error_root = material3::Switch::uncontrolled(cx, false);
    let material3_text_field_error = text_field_error_root.selected_model();

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons_row = ui::h_row(|cx| {
        vec![
            material3::Button::new("Filled")
                .variant(material3::ButtonVariant::Filled)
                .into_element(cx),
            material3::Button::new("Tonal")
                .variant(material3::ButtonVariant::Tonal)
                .into_element(cx),
            material3::Button::new("Outlined")
                .variant(material3::ButtonVariant::Outlined)
                .into_element(cx),
            material3::Button::new("Text")
                .variant(material3::ButtonVariant::Text)
                .into_element(cx),
            material3::Button::new("Disabled")
                .variant(material3::ButtonVariant::Filled)
                .disabled(true)
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let icon_buttons = ui::h_row(|cx| {
        vec![
            material3::IconButton::new(ids::ui::SEARCH)
                .a11y_label("Search")
                .into_element(cx),
            material3::IconButton::new(ids::ui::SETTINGS)
                .a11y_label("Settings")
                .into_element(cx),
            material3::IconButton::new(ids::ui::CLOSE)
                .a11y_label("Close")
                .into_element(cx),
            material3::IconButton::new(ids::ui::SEARCH)
                .a11y_label("Disabled")
                .disabled(true)
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let radio_group = radio_group_root
        .clone()
        .a11y_label("Material 3 RadioGroup")
        .orientation(material3::RadioGroupOrientation::Horizontal)
        .gap(fret_core::Px(8.0))
        .items(vec![
            material3::RadioGroupItem::new("Alpha").a11y_label("Radio Alpha"),
            material3::RadioGroupItem::new("Beta").a11y_label("Radio Beta"),
            material3::RadioGroupItem::new("Charlie")
                .a11y_label("Radio Charlie (disabled)")
                .disabled(true),
        ])
        .into_element(cx);

    let text_field = {
        let disabled = cx
            .get_model_copied(&material3_text_field_disabled, Invalidation::Layout)
            .unwrap_or(false);
        let error = cx
            .get_model_copied(&material3_text_field_error, Invalidation::Layout)
            .unwrap_or(false);

        let toggles = ui::h_row(move |cx| {
            vec![
                text_field_disabled_root
                    .clone()
                    .a11y_label("Text field disabled")
                    .into_element(cx),
                cx.text(format!("disabled={disabled}")),
                text_field_error_root
                    .clone()
                    .a11y_label("Text field error")
                    .into_element(cx),
                cx.text(format!("error={error}")),
            ]
        })
        .gap(Space::N3)
        .items_center()
        .into_element(cx);

        let field = material3::TextField::new(material3_text_field_value)
            .variant(material3::TextFieldVariant::Outlined)
            .label("Name")
            .placeholder("Type here")
            .supporting_text("State matrix field")
            .leading_icon(ids::ui::SEARCH)
            .disabled(disabled)
            .error(error)
            .test_id("ui-gallery-material3-state-matrix-text-field")
            .into_element(cx);

        shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Text field"),
                        shadcn::card_description("Outlined text field bound to models."),
                    ]
                }),
                shadcn::card_content(|_cx| vec![toggles, field]),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
    };

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(cx.text(
        "Material 3 State Matrix: exercise hover/focus/press/disabled/selected states across multiple components.",
    ));
    out.push(cx.text(
        "Tip: use keyboard Tab/Arrow/Home/End on Tabs/Radio/Menu; use Esc/outside press to close Menu.",
    ));

    out.push(cx.text("— Buttons —"));
    out.push(buttons_row);

    out.push(cx.text("— Chips —"));
    out.extend(render_chips(cx, last_action.clone()));

    out.push(cx.text("— Cards —"));
    out.extend(render_cards(cx, last_action.clone()));

    out.push(cx.text("— Icon Buttons —"));
    out.push(icon_buttons);

    out.push(cx.text("— FAB —"));
    out.extend(render_fab(cx, last_action.clone()));

    out.push(cx.text("— Checkbox —"));
    out.push(checkbox_root.a11y_label("Checkbox").into_element(cx));

    out.push(cx.text("— Switch —"));
    out.push(switch_root.a11y_label("Switch").into_element(cx));

    out.push(cx.text("— Radio —"));
    out.push(radio_group);

    out.push(cx.text("— Text Field —"));
    out.push(text_field);

    out.push(cx.text("— Search View —"));
    out.extend(render_search_view(cx));

    out.push(cx.text("— Tabs —"));
    out.push(
        material3::Tabs::new(material3_tabs_value)
            .a11y_label("Material 3 Tabs")
            .items(vec![
                material3::TabItem::new("overview", "Overview"),
                material3::TabItem::new("settings", "Settings"),
                material3::TabItem::new("disabled", "Disabled").disabled(true),
            ])
            .test_id("ui-gallery-material3-state-matrix-tabs")
            .into_element(cx),
    );

    out.push(cx.text("— Navigation Bar —"));
    out.push(
        material3::NavigationBar::new(material3_navigation_bar_value)
            .a11y_label("Material 3 NavigationBar")
            .items(vec![
                material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH),
                material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS),
                material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL),
            ])
            .test_id("ui-gallery-material3-state-matrix-navigation-bar")
            .into_element(cx),
    );

    out.push(cx.text("— Menu —"));
    out.extend(render_menu(cx, last_action.clone()));

    out.push(cx.text(format!("last action: {last}")));

    ui::v_flex(move |_cx| out)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N3)
        .items_start()
        .into_element(cx)
        .into()
}

// endregion: example
