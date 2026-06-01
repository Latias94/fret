use std::{collections::HashMap, sync::Arc};

use fret_core::{
    Color, DockGraph, DockNode, DockNodeId, Edges, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

use super::super::hit_test::tab_scroll_for_node;
use super::super::layout::split_tab_bar;
use super::super::tab_bar_geometry::TabBarGeometry;
use super::super::tab_overflow::tab_strip_rect_with_overflow_button;

#[derive(Debug, Clone)]
pub(in crate::dock) struct TabChromePaintInput {
    pub(in crate::dock) rect: Rect,
    pub(in crate::dock) tabs_len: usize,
    pub(in crate::dock) active: usize,
    pub(in crate::dock) tab_widths: Option<Arc<[Px]>>,
    pub(in crate::dock) scroll: Px,
    pub(in crate::dock) hovered_tab: Option<usize>,
}

pub(in crate::dock) fn tab_chrome_paint_inputs(
    graph: &DockGraph,
    layout: &HashMap<DockNodeId, Rect>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    hovered_tab: Option<(DockNodeId, usize)>,
) -> Vec<TabChromePaintInput> {
    let mut inputs = Vec::new();
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        inputs.push(TabChromePaintInput {
            rect,
            tabs_len: tabs.len(),
            active: *active,
            tab_widths: tab_widths
                .get(&node)
                .filter(|w| w.len() == tabs.len())
                .cloned(),
            scroll: tab_scroll_for_node(tab_scroll, node),
            hovered_tab: hovered_tab
                .and_then(|(hovered_node, index)| (hovered_node == node).then_some(index)),
        });
    }
    inputs
}

fn paint_tab_chrome_input(
    theme: fret_ui::ThemeSnapshot,
    input: &TabChromePaintInput,
    scene: &mut Scene,
) {
    let panel_bg = theme.color_token("card");
    let surface_bg = theme.color_token("background");
    let hover_bg = theme.color_token("accent");
    let primary = theme.color_token("primary");
    let border = theme.color_token("border");
    let radius_sm = theme.metric_token("metric.radius.sm");

    let (tab_bar, _content) = split_tab_bar(input.rect);

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(0),
        rect: input.rect,
        background: fret_core::Paint::Solid(panel_bg).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(1),
        rect: tab_bar,
        background: fret_core::Paint::Solid(surface_bg).into(),
        border: Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        },
        border_paint: fret_core::Paint::Solid(border).into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let tab_geom_candidate = input
        .tab_widths
        .clone()
        .map(|w| TabBarGeometry::variable(strip_candidate, w))
        .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, input.tabs_len));
    let overflow = tab_geom_candidate.max_scroll().0 > 0.0;
    let tab_geom = if overflow {
        tab_geom_candidate
    } else {
        input
            .tab_widths
            .clone()
            .map(|w| TabBarGeometry::variable(tab_bar, w))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, input.tabs_len))
    };
    let tab_strip = if overflow { strip_candidate } else { tab_bar };

    scene.push(SceneOp::PushClipRect { rect: tab_strip });
    for i in 0..input.tabs_len {
        let tab_rect = tab_geom.tab_rect(i, input.scroll);
        if tab_rect.origin.x.0 + tab_rect.size.width.0 < tab_strip.origin.x.0
            || tab_rect.origin.x.0 > tab_strip.origin.x.0 + tab_strip.size.width.0
        {
            continue;
        }

        let is_active = i == input.active;
        let is_hovered = input.hovered_tab == Some(i);
        let (bg, tab_border, corner_radii) = if is_active {
            (
                panel_bg,
                Edges {
                    top: Px(1.0),
                    right: Px(1.0),
                    bottom: Px(0.0),
                    left: Px(1.0),
                },
                fret_core::Corners {
                    top_left: radius_sm,
                    top_right: radius_sm,
                    bottom_left: Px(0.0),
                    bottom_right: Px(0.0),
                },
            )
        } else if is_hovered {
            (
                hover_bg,
                Edges::all(Px(0.0)),
                fret_core::Corners {
                    top_left: radius_sm,
                    top_right: radius_sm,
                    bottom_left: Px(0.0),
                    bottom_right: Px(0.0),
                },
            )
        } else {
            (
                Color { a: 0.0, ..panel_bg },
                Edges::all(Px(0.0)),
                fret_core::Corners::all(Px(0.0)),
            )
        };

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(2),
            rect: tab_rect,
            background: fret_core::Paint::Solid(bg).into(),
            border: tab_border,
            border_paint: fret_core::Paint::Solid(border).into(),
            corner_radii,
        });

        if is_active {
            let underline_h = Px(2.0);
            let underline = Rect {
                origin: Point::new(
                    tab_rect.origin.x,
                    Px(tab_rect.origin.y.0 + tab_rect.size.height.0 - underline_h.0),
                ),
                size: Size::new(tab_rect.size.width, underline_h),
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(3),
                rect: underline,
                background: fret_core::Paint::Solid(primary).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }
    scene.push(SceneOp::PopClip);
}

pub(in crate::dock) fn paint_tab_chrome_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[TabChromePaintInput],
    scene: &mut Scene,
) {
    for input in inputs {
        paint_tab_chrome_input(theme.clone(), input, scene);
    }
}
