pub const SOURCE: &str = include_str!("gallery.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_icons::ids;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let checkbox_root = material3::Checkbox::uncontrolled(cx, false);
    let material3_checkbox = checkbox_root.checked_model();
    let switch_root = material3::Switch::uncontrolled(cx, false);
    let material3_switch = switch_root.selected_model();
    let radio_group_root = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);
    let material3_radio_value = radio_group_root.value_model();
    let tabs_root = material3::Tabs::uncontrolled(cx, "overview");
    let material3_tabs_value = tabs_root.value_model();
    let list_root = material3::List::uncontrolled(cx, "alpha");
    let material3_list_value = list_root.value_model();
    let navigation_bar_root = material3::NavigationBar::uncontrolled(cx, "search");
    let material3_navigation_bar_value = navigation_bar_root.value_model();
    let text_field_root = material3::TextField::uncontrolled(cx);
    let material3_text_field_value = text_field_root.value_model();
    let text_field_disabled_root = material3::Switch::uncontrolled(cx, false);
    let material3_text_field_disabled = text_field_disabled_root.selected_model();
    let text_field_error_root = material3::Switch::uncontrolled(cx, false);
    let material3_text_field_error = text_field_error_root.selected_model();

    let disabled = cx
        .get_model_copied(&material3_text_field_disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error = cx
        .get_model_copied(&material3_text_field_error, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(cx.text("Material 3 Gallery: compact outcomes-first surface."));

    out.push(cx.text("— Buttons —"));
    out.push(
        ui::h_row(|cx| {
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
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx),
    );

    out.push(cx.text("— Icon Buttons —"));
    out.push(
        ui::h_row(|cx| {
            let (hover_icon, hover_container) = cx.with_theme(|theme| {
                (
                    ColorRef::Color(theme.color_token("md.sys.color.on-tertiary-container")),
                    ColorRef::Color(theme.color_token("md.sys.color.tertiary-container")),
                )
            });
            let hover_style = material3::IconButtonStyle::default()
                .container_background(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_container)),
                )
                .icon_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                );

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
                    .a11y_label("Override")
                    .style(hover_style)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx),
    );

    out.push(cx.text("— FAB —"));
    out.push(
        ui::h_row(|cx| {
            vec![
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search")
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search (small)")
                    .size(material3::FabSize::Small)
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search (large)")
                    .size(material3::FabSize::Large)
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search (primary)")
                    .variant(material3::FabVariant::Primary)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx),
    );

    out.push(cx.text("— Selection —"));
    out.push(
        ui::h_row(|cx| {
            let (hover_container, hover_icon, hover_outline) = cx.with_theme(|theme| {
                (
                    ColorRef::Color(theme.color_token("md.sys.color.tertiary-container")),
                    ColorRef::Color(theme.color_token("md.sys.color.on-tertiary-container")),
                    ColorRef::Color(theme.color_token("md.sys.color.tertiary")),
                )
            });
            let hover_style = material3::CheckboxStyle::default()
                .container_background(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_container)),
                )
                .icon_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                )
                .outline_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_outline)),
                );

            vec![
                checkbox_root
                    .clone()
                    .a11y_label("Checkbox")
                    .into_element(cx),
                material3::Checkbox::new(material3_checkbox.clone())
                    .a11y_label("Checkbox Override")
                    .style(hover_style)
                    .into_element(cx),
                switch_root.clone().a11y_label("Switch").into_element(cx),
                material3::Switch::new(material3_switch.clone())
                    .a11y_label("Switch Override")
                    .style({
                        let (hover_track, hover_handle) = cx.with_theme(|theme| {
                            (
                                ColorRef::Color(
                                    theme.color_token("md.sys.color.tertiary-container"),
                                ),
                                ColorRef::Color(
                                    theme.color_token("md.sys.color.on-tertiary-container"),
                                ),
                            )
                        });
                        material3::SwitchStyle::default()
                            .track_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_track)),
                            )
                            .handle_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_handle)),
                            )
                    })
                    .into_element(cx),
                ui::v_stack(|cx| {
                    let items = vec![
                        material3::RadioGroupItem::new("Alpha").a11y_label("Radio Alpha"),
                        material3::RadioGroupItem::new("Beta").a11y_label("Radio Beta"),
                        material3::RadioGroupItem::new("Charlie")
                            .a11y_label("Radio Charlie")
                            .disabled(true),
                    ];

                    let hover_color = cx.with_theme(|theme| {
                        ColorRef::Color(theme.color_token("md.sys.color.tertiary"))
                    });
                    let hover_style = material3::RadioStyle::default()
                        .icon_color(fret_ui_kit::WidgetStateProperty::new(None).when(
                            fret_ui_kit::WidgetStates::HOVERED,
                            Some(hover_color.clone()),
                        ))
                        .state_layer_color(
                            fret_ui_kit::WidgetStateProperty::new(None)
                                .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_color)),
                        );

                    vec![
                        cx.text("Radio Group"),
                        radio_group_root
                            .clone()
                            .a11y_label("Radio Group")
                            .items(items.clone())
                            .into_element(cx),
                        cx.text("Radio Group Override"),
                        material3::RadioGroup::new(material3_radio_value.clone())
                            .a11y_label("Radio Group Override")
                            .style(hover_style)
                            .items(items)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N1)
                .items_start()
                .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_center()
        .into_element(cx),
    );

    out.push(cx.text("— Text Field —"));
    out.push(
        ui::h_row(|cx| {
            vec![
                text_field_disabled_root
                    .clone()
                    .a11y_label("Disable Text Field")
                    .into_element(cx),
                cx.text("Disabled"),
                text_field_error_root
                    .clone()
                    .a11y_label("Text Field Error")
                    .into_element(cx),
                cx.text("Error"),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx),
    );
    out.push(
        ui::v_stack(|cx| {
            let hover =
                cx.with_theme(|theme| ColorRef::Color(theme.color_token("md.sys.color.tertiary")));
            let hover_style = material3::TextFieldStyle::default()
                .outline_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover.clone())),
                )
                .label_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover)),
                );

            vec![
                cx.text("Text Field"),
                material3::TextField::new(material3_text_field_value.clone())
                    .label("Label")
                    .placeholder("Placeholder")
                    .disabled(disabled)
                    .error(error)
                    .into_element(cx),
                cx.text("Text Field Override"),
                material3::TextField::new(material3_text_field_value)
                    .label("Label")
                    .placeholder("Placeholder")
                    .style(hover_style)
                    .disabled(disabled)
                    .error(error)
                    .into_element(cx),
            ]
        })
        .gap(Space::N1)
        .items_start()
        .into_element(cx),
    );

    out.push(cx.text("— Tabs —"));
    out.push(
        ui::v_stack(|cx| {
            let items = vec![
                material3::TabItem::new("overview", "Overview"),
                material3::TabItem::new("security", "Security"),
                material3::TabItem::new("settings", "Settings"),
            ];

            let hover_color =
                cx.with_theme(|theme| ColorRef::Color(theme.color_token("md.sys.color.tertiary")));
            let hover_style = material3::TabsStyle::default()
                .label_color(fret_ui_kit::WidgetStateProperty::new(None).when(
                    fret_ui_kit::WidgetStates::HOVERED,
                    Some(hover_color.clone()),
                ))
                .state_layer_color(fret_ui_kit::WidgetStateProperty::new(None).when(
                    fret_ui_kit::WidgetStates::HOVERED,
                    Some(hover_color.clone()),
                ))
                .active_indicator_color(fret_ui_kit::WidgetStateProperty::new(Some(hover_color)));

            vec![
                cx.text("Tabs"),
                material3::Tabs::new(material3_tabs_value.clone())
                    .a11y_label("Tabs")
                    .items(items.clone())
                    .into_element(cx),
                cx.text("Tabs Override"),
                material3::Tabs::new(material3_tabs_value)
                    .a11y_label("Tabs Override")
                    .style(hover_style)
                    .items(items)
                    .into_element(cx),
            ]
        })
        .gap(Space::N1)
        .items_start()
        .into_element(cx),
    );

    out.push(cx.text("— Navigation Bar —"));
    out.push(
        material3::NavigationBar::new(material3_navigation_bar_value)
            .a11y_label("Navigation bar")
            .items(vec![
                material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH),
                material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS),
                material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL),
            ])
            .into_element(cx),
    );

    out.push(cx.text("— List —"));
    out.push(
        material3::List::new(material3_list_value)
            .a11y_label("List")
            .items(vec![
                material3::ListItem::new("alpha", "Alpha").leading_icon(ids::ui::SEARCH),
                material3::ListItem::new("beta", "Beta").leading_icon(ids::ui::SETTINGS),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true),
            ])
            .into_element(cx),
    );

    ui::v_flex(move |_cx| out)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N3)
        .items_start()
        .into_element(cx)
        .into()
}

// endregion: example
