use super::super::super::*;

pub(in crate::ui) fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let fixed_tabs = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs")
        .test_id("ui-gallery-material3-tabs")
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled"),
        ])
        .into_element(cx);

    let override_style = material3::TabsStyle::default()
        .label_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ))
        .state_layer_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ))
        .active_indicator_color(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ));
    let fixed_tabs_overridden = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs (overridden)")
        .test_id("ui-gallery-material3-tabs-overridden")
        .style(override_style)
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview-overridden"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings-overridden"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled-overridden"),
        ])
        .into_element(cx);

    let scrollable_tabs = material3::Tabs::new(value)
        .a11y_label("Material 3 Tabs (scrollable)")
        .test_id("ui-gallery-material3-tabs-scrollable")
        .scrollable(true)
        .items(vec![
            material3::TabItem::new("overview", "Overview"),
            material3::TabItem::new("settings", "Settings"),
            material3::TabItem::new("typography", "Typography"),
            material3::TabItem::new("very_long_label", "Very Long Label For Layout Probe"),
            material3::TabItem::new("tokens", "Tokens"),
            material3::TabItem::new("motion", "Motion"),
            material3::TabItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    vec![
        cx.text("Material 3 Tabs: roving focus + state layer + bounded ripple."),
        fixed_tabs,
        cx.text(
            "Override preview: hover label/state-layer + active-indicator color via TabsStyle.",
        ),
        fixed_tabs_overridden,
        cx.text("Scrollable/variable width preview (measurement-driven indicator)."),
        scrollable_tabs,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_navigation_bar(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let bar = material3::NavigationBar::new(value)
        .a11y_label("Material 3 Navigation Bar")
        .test_id("ui-gallery-material3-navigation-bar")
        .items(vec![
            material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                .badge_dot()
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-nav-search"),
            material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-nav-settings"),
            material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                .badge_text("9")
                .a11y_label("Destination More")
                .test_id("ui-gallery-material3-nav-more"),
        ])
        .into_element(cx);

    vec![
        cx.text("Material 3 Navigation Bar: roving focus + state layer + bounded ripple."),
        bar,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_navigation_rail(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let rail = material3::NavigationRail::new(value)
        .a11y_label("Material 3 Navigation Rail")
        .test_id("ui-gallery-material3-navigation-rail")
        .items(vec![
            material3::NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                .badge_dot()
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-rail-search"),
            material3::NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-rail-settings"),
            material3::NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                .badge_text("99+")
                .a11y_label("Destination Play")
                .test_id("ui-gallery-material3-rail-play"),
            material3::NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                .disabled(true)
                .a11y_label("Destination Disabled")
                .test_id("ui-gallery-material3-rail-disabled"),
        ])
        .into_element(cx);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [rail],
    );

    vec![
        cx.text("Material 3 Navigation Rail: roving focus + state layer + bounded ripple."),
        container,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let drawer = material3::NavigationDrawer::new(value)
        .a11y_label("Material 3 Navigation Drawer")
        .test_id("ui-gallery-material3-navigation-drawer")
        .items(vec![
            material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-drawer-search"),
            material3::NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                .badge_label("2")
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-drawer-settings"),
            material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                .badge_label("99+")
                .a11y_label("Destination Play")
                .test_id("ui-gallery-material3-drawer-play"),
            material3::NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                .disabled(true)
                .a11y_label("Destination Disabled")
                .test_id("ui-gallery-material3-drawer-disabled"),
        ])
        .into_element(cx);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(280.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [drawer],
    );

    vec![
        cx.text("Material 3 Navigation Drawer: roving focus + state layer + bounded ripple."),
        container,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_modal_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let open_drawer: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let modal = material3::ModalNavigationDrawer::new(open.clone())
        .test_id("ui-gallery-material3-modal-navigation-drawer")
        .into_element(
            cx,
            move |cx| {
                material3::NavigationDrawer::new(value)
                    .variant(material3::NavigationDrawerVariant::Modal)
                    .a11y_label("Material 3 Modal Navigation Drawer")
                    .test_id("ui-gallery-material3-modal-navigation-drawer-panel")
                    .items(vec![
                        material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("ui-gallery-material3-modal-drawer-search"),
                        material3::NavigationDrawerItem::new(
                            "settings",
                            "Settings",
                            ids::ui::SETTINGS,
                        )
                        .badge_label("2")
                        .a11y_label("Destination Settings")
                        .test_id("ui-gallery-material3-modal-drawer-settings"),
                        material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .a11y_label("Destination Play")
                            .test_id("ui-gallery-material3-modal-drawer-play"),
                        material3::NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("ui-gallery-material3-modal-drawer-disabled"),
                    ])
                    .into_element(cx)
            },
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open drawer")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_drawer.clone())
                                .test_id("ui-gallery-material3-modal-drawer-open")
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id("ui-gallery-material3-modal-drawer-underlay-probe")
                                .into_element(cx),
                            cx.text(
                                "Tip: click the scrim or press Esc to close; Tab/Shift+Tab should stay inside the drawer while open.",
                            ),
                        ]
                    },
                )
            },
        );

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [modal],
    );

    vec![
        cx.text("Material 3 Modal Navigation Drawer: modal scrim + focus trap/restore + token-driven motion."),
        container,
        cx.text(format!(
            "open={} value={}",
            is_open as u8,
            current.as_ref()
        )),
    ]
}
