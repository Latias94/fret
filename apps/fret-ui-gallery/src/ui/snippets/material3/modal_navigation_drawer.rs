pub const SOURCE: &str = include_str!("modal_navigation_drawer.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons::ids;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

fn route_panel(cx: &mut AppComponentCx<'_>, route: &str) -> AnyElement {
    let (route_id, title, body) = match route {
        "settings" => (
            "settings",
            "Settings",
            "Modal drawer-selected settings content with close-after-selection policy.",
        ),
        "play" => (
            "play",
            "Play",
            "Playback queue, saved media, and current session details.",
        ),
        _ => (
            "search",
            "Search",
            "Modal drawer-selected search content and discovery shortcuts.",
        ),
    };

    ui::v_flex(move |cx| {
        vec![
            cx.text(title).test_id(format!(
                "ui-gallery-material3-modal-navigation-drawer-route-panel-{route_id}"
            )),
            cx.text(body),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .test_id("ui-gallery-material3-modal-navigation-drawer-route-panel")
    .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let modal = material3::ModalNavigationDrawer::uncontrolled(cx);
    let open = modal.open_model();
    let value = cx.local_model_keyed("value", || Arc::<str>::from("search"));
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
    let close_on_select: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let current_for_content = current.clone();

    let modal = modal
        .test_id("ui-gallery-material3-modal-navigation-drawer")
        .into_element(
            cx,
            move |cx| {
                material3::NavigationDrawer::new(value.clone())
                    .variant(material3::NavigationDrawerVariant::Modal)
                    .a11y_label("Material 3 Modal Navigation Drawer")
                    .test_id("ui-gallery-material3-modal-navigation-drawer-panel")
                    .items(vec![
                        material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .on_select(close_on_select.clone())
                            .a11y_label("Destination Search")
                            .test_id("ui-gallery-material3-modal-drawer-search"),
                        material3::NavigationDrawerItem::new(
                            "settings",
                            "Settings",
                            ids::ui::SETTINGS,
                        )
                        .badge_label("2")
                        .on_select(close_on_select.clone())
                        .a11y_label("Destination Settings")
                        .test_id("ui-gallery-material3-modal-drawer-settings"),
                        material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .on_select(close_on_select.clone())
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
                let current = current_for_content.clone();
                ui::v_flex(move |cx| {
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
                                "Tip: select a destination to close the drawer; Tab/Shift+Tab should stay inside while open.",
                            ),
                            route_panel(cx, current.as_ref()),
                        ]
                    })
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4).into_element(cx)
            },
        );

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(Px(360.0));
    let container = cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [modal],
    );

    ui::v_flex(|cx| {
            vec![
                cx.text(
                    "Material 3 Modal Navigation Drawer: modal scrim + focus trap/restore + routed content.",
                ),
                container,
                cx.text(format!("open={} value={}", is_open as u8, current.as_ref())),
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
}

// endregion: example
