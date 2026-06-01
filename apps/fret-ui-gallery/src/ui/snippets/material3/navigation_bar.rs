pub const SOURCE: &str = include_str!("navigation_bar.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_icons::ids;
use fret_ui::element::AnyElement;
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

fn route_panel(cx: &mut AppComponentCx<'_>, route: &str) -> AnyElement {
    let (route_id, title, body) = match route {
        "settings" => (
            "settings",
            "Settings",
            "Account, privacy, and notification preferences.",
        ),
        "more" => (
            "more",
            "More",
            "Secondary destinations and overflow actions.",
        ),
        _ => (
            "search",
            "Search",
            "Recent queries, saved filters, and discovery shortcuts.",
        ),
    };

    ui::v_flex(move |cx| {
        vec![
            cx.text(title).test_id(format!(
                "ui-gallery-material3-navigation-bar-route-panel-{route_id}"
            )),
            cx.text(body),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .test_id("ui-gallery-material3-navigation-bar-route-panel")
    .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let bar = material3::NavigationBar::uncontrolled(cx, "search");
    let value = bar.value_model();
    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let bar = bar
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

    ui::v_flex(|cx| {
        vec![
            cx.text("Material 3 Navigation Bar: roving focus + state layer + bounded ripple."),
            bar,
            route_panel(cx, current.as_ref()),
            cx.text(format!("value={}", current.as_ref())),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

// endregion: example
