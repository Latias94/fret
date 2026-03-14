pub const SOURCE: &str = include_str!("modal_navigation_drawer.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::ids;
use fret_ui::action::OnActivate;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
                                "Tip: click the scrim or press Esc to close; Tab/Shift+Tab should stay inside the drawer while open.",
                            ),
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
                    "Material 3 Modal Navigation Drawer: modal scrim + focus trap/restore + token-driven motion.",
                ),
                container,
                cx.text(format!("open={} value={}", is_open as u8, current.as_ref())),
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
    .into()
}

// endregion: example
