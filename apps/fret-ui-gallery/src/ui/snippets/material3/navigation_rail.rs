pub const SOURCE: &str = include_str!("navigation_rail.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::ids;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let rail = material3::NavigationRail::uncontrolled(cx, "search");
    let value = rail.value_model();
    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let rail = rail
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

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(Px(360.0));
    let container = cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [rail],
    );

    ui::v_flex(|cx| {
        vec![
            cx.text("Material 3 Navigation Rail: roving focus + state layer + bounded ripple."),
            container,
            cx.text(format!("value={}", current.as_ref())),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
    .into()
}

// endregion: example
