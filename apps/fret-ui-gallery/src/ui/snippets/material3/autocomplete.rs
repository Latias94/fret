pub const SOURCE: &str = include_str!("autocomplete.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::action::OnActivate;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let outlined_autocomplete = material3::Autocomplete::uncontrolled(cx);
    let value = outlined_autocomplete.query_model();
    let dialog = material3::Dialog::uncontrolled(cx);
    let dialog_open = dialog.open_model();
    let disabled_toggle = material3::Switch::uncontrolled(cx, false);
    let disabled = disabled_toggle.selected_model();
    let error_toggle = material3::Switch::uncontrolled(cx, false);
    let error = error_toggle.selected_model();
    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error_now = cx
        .get_model_copied(&error, Invalidation::Layout)
        .unwrap_or(false);
    let selected_value = cx.local_model_keyed("selected_value", || None::<Arc<str>>);
    let exposed_dropdown = material3::ExposedDropdown::new_controllable(
        cx,
        None,
        Some(Arc::<str>::from("beta")),
        None,
        String::new(),
    );
    let exposed_selected_value = exposed_dropdown.selected_value_model();
    let exposed_query = exposed_dropdown.query_model();

    let query_now = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_default();
    let selected_now = cx
        .get_model_cloned(&selected_value, Invalidation::Layout)
        .unwrap_or(None);
    let selected_label = selected_now.as_deref().unwrap_or("<none>");

    let exposed_selected_now = cx
        .get_model_cloned(&exposed_selected_value, Invalidation::Layout)
        .unwrap_or(None);
    let exposed_selected_label = exposed_selected_now.as_deref().unwrap_or("<none>");

    let toggles = ui::h_row(move |cx| {
        vec![
            cx.text("disabled"),
            disabled_toggle
                .clone()
                .a11y_label("Disable autocomplete")
                .test_id("ui-gallery-material3-autocomplete-disabled")
                .into_element(cx),
            cx.text("error"),
            error_toggle
                .clone()
                .a11y_label("Toggle autocomplete error state")
                .test_id("ui-gallery-material3-autocomplete-error")
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx);

    let items: Arc<[material3::AutocompleteItem]> = Arc::from(vec![
        material3::AutocompleteItem::new("alpha", "Alpha"),
        material3::AutocompleteItem::new("beta", "Beta"),
        material3::AutocompleteItem::new("gamma", "Gamma"),
        material3::AutocompleteItem::new("delta", "Delta"),
        material3::AutocompleteItem::new("epsilon", "Epsilon"),
        material3::AutocompleteItem::new("zeta", "Zeta"),
    ]);

    let supporting = if error_now {
        "Error: required"
    } else {
        "Supporting text"
    };

    let outlined = outlined_autocomplete
        .selected_value(selected_value.clone())
        .variant(material3::AutocompleteVariant::Outlined)
        .label("Search")
        .placeholder("Type to filter")
        .supporting_text(supporting)
        .leading_icon(fret_icons::ids::ui::SEARCH)
        .items(items.clone())
        .disabled(disabled_now)
        .error(error_now)
        .a11y_label("autocomplete outlined")
        .test_id("ui-gallery-material3-autocomplete")
        .into_element(cx);

    let outlined_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Outlined"),
                    shadcn::card_description(
                        "Combobox-style: focus stays on the input; the active option is exposed via active-descendant.",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![outlined]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let filled = material3::Autocomplete::new(value.clone())
        .selected_value(selected_value.clone())
        .variant(material3::AutocompleteVariant::Filled)
        .label("Search (filled)")
        .placeholder("Type to filter")
        .supporting_text(supporting)
        .leading_icon(fret_icons::ids::ui::SEARCH)
        .items(items.clone())
        .disabled(disabled_now)
        .error(error_now)
        .a11y_label("autocomplete filled")
        .test_id("ui-gallery-material3-autocomplete-filled")
        .into_element(cx);

    let filled_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Filled"),
                    shadcn::card_description(
                        "Filled container + active indicator outcomes (token-driven).",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![filled]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let exposed = exposed_dropdown
        .variant(material3::AutocompleteVariant::Outlined)
        .label("Searchable select")
        .placeholder("Type to filter")
        .supporting_text(
            "Policy: when the input blurs, the query reverts to the committed selection.",
        )
        .leading_icon(fret_icons::ids::ui::SEARCH)
        .items(items.clone())
        .disabled(disabled_now)
        .error(error_now)
        .a11y_label("exposed dropdown")
        .test_id("ui-gallery-material3-exposed-dropdown")
        .into_element(cx);

    let exposed_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Exposed dropdown (composition)"),
                    shadcn::card_description(
                        "Compose-style: a committed selection model drives the closed display, while the query stays editable while focused.",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![exposed]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let open_action: OnActivate = {
        let dialog_open = dialog_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&dialog_open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_action: OnActivate = {
        let dialog_open = dialog_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&dialog_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let dialog = dialog
        .headline("Autocomplete (Dialog probe)")
        .supporting_text("Overlay should anchor correctly inside a modal dialog without clipping.")
        .actions(vec![material3::DialogAction::new("Close").on_activate(close_action)])
        .test_id("ui-gallery-material3-autocomplete-dialog")
        .into_element(
            cx,
            move |cx| {
                ui::v_flex(move |cx| {
                        vec![
                            material3::Button::new("Open dialog probe")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_action.clone())
                                .test_id("ui-gallery-material3-autocomplete-dialog-open")
                                .into_element(cx),
                            cx.text(
                                "Tip: focus the autocomplete and press ArrowDown; keep typing while the menu is open.",
                            ),
                        ]
                    })
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4).into_element(cx)
            },
            {
                let items = items.clone();
                let value = value.clone();
                let selected_value = selected_value.clone();
                move |cx| {
                    let spacer = cx.container(
                        ContainerProps {
                            layout: {
                                let mut l = LayoutStyle::default();
                                l.size.width = Length::Fill;
                                l.size.height = Length::Px(Px(360.0));
                                l
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    );

                    vec![ui::v_flex(move |cx| {
                            vec![
                                material3::Autocomplete::new(value.clone())
                                    .selected_value(selected_value.clone())
                                    .variant(material3::AutocompleteVariant::Outlined)
                                    .label("Dialog autocomplete")
                                    .placeholder("Type to filter")
                                    .supporting_text("Bottom-edge clamping probe: open near the dialog bottom.")
                                    .items(items.clone())
                                    .a11y_label("autocomplete dialog")
                                    .test_id("ui-gallery-material3-autocomplete-dialog-field")
                                    .into_element(cx),
                                spacer,
                                material3::Autocomplete::new(value.clone())
                                    .selected_value(selected_value.clone())
                                    .variant(material3::AutocompleteVariant::Outlined)
                                    .label("Dialog autocomplete (bottom)")
                                    .placeholder("Type to filter")
                                    .supporting_text("Open menu near the dialog bottom edge.")
                                    .items(items.clone())
                                    .a11y_label("autocomplete dialog bottom")
                                    .test_id("ui-gallery-material3-autocomplete-dialog-field-bottom")
                                    .into_element(cx),
                            ]
                        })
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4).into_element(cx)]
                }
            },
        );

    ui::v_flex(move |cx| {
        vec![
            cx.text(
                "Material 3 Autocomplete: editable combobox input with a listbox popover menu.",
            ),
            toggles,
            cx.text(Arc::from(format!(
                "Query: \"{}\" | Selected value: {}",
                query_now, selected_label
            ))),
            cx.text(Arc::from(format!(
                "Exposed dropdown committed value: {}",
                exposed_selected_label
            ))),
            exposed_card,
            outlined_card,
            filled_card,
            dialog,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

// endregion: example
