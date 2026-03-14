use super::super::super::super::super::*;
use fret::UiChild;
use fret::UiCx;

use super::widgets::OverlayWidgets;

// Typed row helpers: the diagnostics layout still consumes landed overlay/widget roots from
// `OverlayWidgets`, but callers now keep the final landing explicit at the cached preview seam.
fn row(_cx: &mut UiCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<> {
    ui::h_flex(move |_cx| children)
        .gap_px(gap)
        .justify_start()
        .items_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
}

fn row_end(_cx: &mut UiCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<> {
    ui::h_flex(move |_cx| children)
        .gap_px(gap)
        .justify_end()
        .items_center()
        .no_wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub(super) fn compose_body(cx: &mut UiCx<'_>, widgets: OverlayWidgets) -> impl UiChild + use<> {
    ui::v_flex(move |cx| {
        let gap = cx.with_theme(|theme| fret_ui_kit::MetricRef::space(Space::N2).resolve(theme));

        ui::children![cx;
            row_end(cx, gap, vec![widgets.underlay]),
            row(
                cx,
                gap,
                vec![
                    widgets.dropdown,
                    widgets.context_menu,
                    widgets.overlay_reset,
                ],
            ),
            row_end(cx, gap, vec![widgets.context_menu_edge]),
            row(
                cx,
                gap,
                vec![
                    widgets.tooltip,
                    widgets.hover_card,
                    widgets.popover,
                    widgets.dialog,
                    widgets.dialog_glass,
                ],
            ),
            row(cx, gap, vec![widgets.alert_dialog, widgets.sheet]),
            widgets.portal_geometry,
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_full())
}
