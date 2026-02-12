use super::super::super::super::super::*;

use super::widgets::OverlayWidgets;

fn row(cx: &mut ElementContext<'_, App>, gap: Px, children: Vec<AnyElement>) -> AnyElement {
    let layout = cx.with_theme(|theme| {
        decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0())
    });
    cx.flex(
        fret_ui::element::FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Center,
            wrap: true,
        },
        |_cx| children,
    )
}

fn row_end(cx: &mut ElementContext<'_, App>, gap: Px, children: Vec<AnyElement>) -> AnyElement {
    let layout = cx.with_theme(|theme| {
        decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0())
    });
    cx.flex(
        fret_ui::element::FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: fret_ui::element::MainAlign::End,
            align: fret_ui::element::CrossAlign::Center,
            wrap: false,
        },
        |_cx| children,
    )
}

pub(super) fn compose_body(
    cx: &mut ElementContext<'_, App>,
    widgets: OverlayWidgets,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
        |cx| {
            let gap =
                cx.with_theme(|theme| fret_ui_kit::MetricRef::space(Space::N2).resolve(theme));

            vec![
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
                        widgets.underlay,
                        widgets.dialog,
                    ],
                ),
                row(cx, gap, vec![widgets.alert_dialog, widgets.sheet]),
                widgets.portal_geometry,
            ]
        },
    )
}
