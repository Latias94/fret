pub const SOURCE: &str = include_str!("navigation_drawer.rs");

// region: example
use std::sync::Arc;

use fret_core::Px;
use fret_icons::ids;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<Arc<str>>) -> AnyElement {
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

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(Px(280.0));
    let container = cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [drawer],
    );

    ui::v_flex(|cx| {
        vec![
            cx.text("Material 3 Navigation Drawer: roving focus + state layer + bounded ripple."),
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
