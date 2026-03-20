use super::super::super::super::super::*;
use fret::UiChild;
use fret::UiCx;

use super::{OverlayModels, widgets};

// Typed row helpers: the diagnostics layout now composes typed widget helpers directly, while the
// final landing stays explicit at the cached preview seam.
fn row(_cx: *mut UiCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<> {
    ui::h_flex(move |_cx| children)
        .gap_px(gap)
        .justify_start()
        .items_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
}

fn row_end(_cx: *mut UiCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<> {
    ui::h_flex(move |_cx| children)
        .gap_px(gap)
        .justify_end()
        .items_center()
        .no_wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub(super) fn compose_body(cx: &mut UiCx<'_>, models: OverlayModels) -> impl UiChild + use<> {
    ui::v_flex(move |cx| {
        let gap = cx.with_theme(|theme| fret_ui_kit::MetricRef::space(Space::N2).resolve(theme));

        ui::children![cx;
            row_end(cx, gap, ui::children![cx; widgets::underlay(cx)]),
            row(
                cx,
                gap,
                ui::children![
                    cx;
                    widgets::dropdown(cx, &models),
                    widgets::context_menu(cx, &models),
                    widgets::overlay_reset(cx, &models),
                ],
            ),
            row_end(
                cx,
                gap,
                ui::children![cx; widgets::context_menu_edge(cx, &models)],
            ),
            row(
                cx,
                gap,
                ui::children![
                    cx;
                    widgets::tooltip(cx),
                    widgets::hover_card(cx),
                    widgets::popover(cx, &models),
                    widgets::dialog(cx, &models),
                    widgets::dialog_glass(cx, &models),
                ],
            ),
            row(
                cx,
                gap,
                ui::children![cx; widgets::alert_dialog(cx, &models), widgets::sheet(cx, &models)],
            ),
            widgets::portal_geometry(cx, &models),
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_full())
}
