pub const SOURCE: &str = include_str!("snackbar.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::{Color, Edges, Px};
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui_kit::{ColorRef, ToastStore, WidgetStateProperty};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";

pub fn render(cx: &mut AppComponentCx<'_>, last_action: Model<Arc<str>>) -> impl UiChild + use<> {
    let store = cx.local_model_keyed("store", ToastStore::default);

    let host_style = material3::SnackbarStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(Color {
            r: 0.12,
            g: 0.15,
            b: 0.18,
            a: 1.0,
        }))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(Color {
            r: 0.96,
            g: 0.92,
            b: 0.82,
            a: 1.0,
        }))))
        .action_label_color(WidgetStateProperty::new(Some(ColorRef::Color(Color {
            r: 0.67,
            g: 0.86,
            b: 1.0,
            a: 1.0,
        }))))
        .close_icon_color(WidgetStateProperty::new(Some(ColorRef::Color(Color {
            r: 0.96,
            g: 0.92,
            b: 0.82,
            a: 0.88,
        }))))
        .container_corner_radius(WidgetStateProperty::new(Some(Px(10.0))))
        .container_padding(WidgetStateProperty::new(Some(Edges {
            left: Px(18.0),
            right: Px(14.0),
            top: Px(10.0),
            bottom: Px(10.0),
        })));

    let host_layer = material3::SnackbarHost::new(store.clone())
        .max_snackbars(1)
        .style(host_style)
        .into_element(cx);

    let show_short: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Saved")
                    .action_id("Undo", CommandId::new(CMD_TOAST_ACTION)),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_two_line: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Update available")
                    .supporting_text("Restart the app to apply the latest changes.")
                    .action_id("Restart", CommandId::new(CMD_TOAST_ACTION))
                    .duration(material3::SnackbarDuration::Long),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_indefinite: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Connection lost")
                    .supporting_text("Trying to reconnect...")
                    .duration(material3::SnackbarDuration::Indefinite),
            );
            host.request_redraw(acx.window);
        })
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = ui::h_flex(|cx| {
        vec![
            material3::Button::new("Show (short)")
                .variant(material3::ButtonVariant::Outlined)
                .on_activate(show_short.clone())
                .test_id("ui-gallery-material3-snackbar-show-short")
                .into_element(cx),
            material3::Button::new("Show (two-line)")
                .variant(material3::ButtonVariant::Outlined)
                .on_activate(show_two_line.clone())
                .test_id("ui-gallery-material3-snackbar-show-two-line")
                .into_element(cx),
            material3::Button::new("Show (indefinite)")
                .variant(material3::ButtonVariant::Outlined)
                .on_activate(show_indefinite.clone())
                .test_id("ui-gallery-material3-snackbar-show-indefinite")
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Snackbar"),
                    shadcn::card_description(
                        "Snackbar MVP: Material token-driven toast-layer skin (md.comp.snackbar.*).",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                vec![
                    host_layer,
                    buttons,
                    cx.text(format!("last action: {last}")),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

// endregion: example
