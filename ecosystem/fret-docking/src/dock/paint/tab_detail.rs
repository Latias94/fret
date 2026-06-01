use std::{collections::HashMap, sync::Arc};

use fret_core::{
    Edges, PanelKey, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

use super::super::consts::{DOCK_TAB_CLOSE_GAP, DOCK_TAB_CLOSE_SIZE};
use super::super::hit_test::{tab_close_rect, tab_scroll_for_node};
use super::super::tab_bar_geometry::TabBarGeometry;
use super::super::tab_overflow::{
    TabOverflowMenuState, overflow_menu_close_rect, overflow_menu_max_scroll,
    overflow_menu_row_count, overflow_menu_row_height, overflow_menu_row_rect,
    tab_overflow_button_rect, tab_overflow_menu_rect, tab_strip_rect_with_overflow_button,
};
use super::super::types::PreparedTabTitle;
use super::super::layout::split_tab_bar;
use fret_core::{DockGraph, DockNode, DockNodeId};

fn tab_title_clip_rect(
    theme: fret_ui::ThemeSnapshot,
    tab_rect: Rect,
    close_glyph_present: bool,
) -> Rect {
    let pad_x = theme.metric_token("metric.padding.md").0.max(0.0);
    let pad_sm = theme.metric_token("metric.padding.sm").0.max(0.0);
    let reserve = if close_glyph_present {
        DOCK_TAB_CLOSE_SIZE.0 + DOCK_TAB_CLOSE_GAP.0 + pad_sm
    } else {
        0.0
    };

    let max_pad = (tab_rect.size.width.0 - reserve - 1.0).max(0.0);
    let pad_x = pad_x.clamp(0.0, max_pad);

    Rect {
        origin: Point::new(Px(tab_rect.origin.x.0 + pad_x), tab_rect.origin.y),
        size: Size::new(
            Px((tab_rect.size.width.0 - pad_x - reserve).max(1.0)),
            tab_rect.size.height,
        ),
    }
}

#[derive(Debug, Clone)]
pub(in crate::dock) struct TabDetailPaintInput {
    pub(in crate::dock) rect: Rect,
    pub(in crate::dock) tabs: Arc<[PanelKey]>,
    pub(in crate::dock) active: usize,
    pub(in crate::dock) tab_widths: Option<Arc<[Px]>>,
    pub(in crate::dock) scroll: Px,
    pub(in crate::dock) hovered_tab: Option<usize>,
    pub(in crate::dock) hovered_tab_close: bool,
    pub(in crate::dock) hovered_tab_overflow_button: bool,
    pub(in crate::dock) pressed_tab_close: Option<usize>,
    pub(in crate::dock) tab_overflow_menu: Option<TabOverflowMenuState>,
}

pub(in crate::dock) fn tab_detail_paint_inputs(
    graph: &DockGraph,
    layout: &HashMap<DockNodeId, Rect>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    hovered_tab_overflow_button: Option<DockNodeId>,
    pressed_tab_close: Option<(DockNodeId, usize)>,
    tab_overflow_menu: Option<TabOverflowMenuState>,
) -> Vec<TabDetailPaintInput> {
    let mut inputs = Vec::new();
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        inputs.push(TabDetailPaintInput {
            rect,
            tabs: Arc::from(tabs.clone()),
            active: *active,
            tab_widths: tab_widths
                .get(&node)
                .filter(|w| w.len() == tabs.len())
                .cloned(),
            scroll: tab_scroll_for_node(tab_scroll, node),
            hovered_tab: hovered_tab
                .and_then(|(hovered_node, index)| (hovered_node == node).then_some(index)),
            hovered_tab_close,
            hovered_tab_overflow_button: hovered_tab_overflow_button == Some(node),
            pressed_tab_close: pressed_tab_close
                .and_then(|(pressed_node, index)| (pressed_node == node).then_some(index)),
            tab_overflow_menu: tab_overflow_menu
                .as_ref()
                .filter(|menu| menu.tabs == node)
                .cloned(),
        });
    }
    inputs
}

#[allow(clippy::too_many_arguments)]
fn paint_tab_detail_input(
    theme: fret_ui::ThemeSnapshot,
    input: &TabDetailPaintInput,
    tab_titles: &HashMap<PanelKey, PreparedTabTitle>,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_overflow_glyph: Option<PreparedTabTitle>,
    tab_close_svg: Option<fret_core::SvgId>,
    tab_overflow_svg: Option<fret_core::SvgId>,
    scene: &mut Scene,
) {
    let hover_bg = theme.color_token("accent");
    let fg = theme.color_token("foreground");
    let fg_muted = theme.color_token("muted-foreground");
    let radius_sm = theme.metric_token("metric.radius.sm");

    let (tab_bar, _content) = split_tab_bar(input.rect);
    let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let tab_geom_candidate = input
        .tab_widths
        .clone()
        .map(|w| TabBarGeometry::variable(strip_candidate, w))
        .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, input.tabs.len()));
    let overflow = tab_geom_candidate.max_scroll().0 > 0.0;
    let tab_geom = if overflow {
        tab_geom_candidate
    } else {
        input
            .tab_widths
            .clone()
            .map(|w| TabBarGeometry::variable(tab_bar, w))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, input.tabs.len()))
    };
    let tab_strip = if overflow { strip_candidate } else { tab_bar };

    scene.push(SceneOp::PushClipRect { rect: tab_strip });

    for (i, panel) in input.tabs.iter().enumerate() {
        let tab_rect = tab_geom.tab_rect(i, input.scroll);
        if tab_rect.origin.x.0 + tab_rect.size.width.0 < tab_strip.origin.x.0
            || tab_rect.origin.x.0 > tab_strip.origin.x.0 + tab_strip.size.width.0
        {
            continue;
        }

        let is_active = i == input.active;
        let is_hovered = input.hovered_tab == Some(i);

        if let Some(title) = tab_titles.get(panel) {
            let clip = tab_title_clip_rect(theme.clone(), tab_rect, tab_close_glyph.is_some());
            let text_x = clip.origin.x;
            let inner_y = tab_rect.origin.y.0
                + ((tab_rect.size.height.0 - title.metrics.size.height.0) * 0.5);
            let text_y = Px(inner_y + title.metrics.baseline.0);
            let text_color = if is_active || is_hovered {
                fg
            } else {
                fg_muted
            };

            scene.push(SceneOp::PushClipRect { rect: clip });
            scene.push(SceneOp::Text {
                order: fret_core::DrawOrder(4),
                origin: Point::new(text_x, text_y),
                text: title.blob,
                paint: (text_color).into(),
                outline: None,
                shadow: None,
            });
            scene.push(SceneOp::PopClip);
        }

        if (is_active || is_hovered) && (tab_close_svg.is_some() || tab_close_glyph.is_some()) {
            let close_rect = tab_close_rect(theme.clone(), tab_rect);
            let close_hovered = is_hovered && input.hovered_tab_close;
            let close_pressed = input.pressed_tab_close == Some(i);

            if close_pressed || close_hovered {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(5),
                    rect: close_rect,
                    background: fret_core::Paint::Solid(hover_bg).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),
                    corner_radii: fret_core::Corners::all(radius_sm),
                });
            }

            let color = if close_pressed || close_hovered {
                fg
            } else {
                fg_muted
            };
            if let Some(svg) = tab_close_svg {
                let pad = Px(2.0);
                let rect = Rect {
                    origin: Point::new(
                        Px(close_rect.origin.x.0 + pad.0),
                        Px(close_rect.origin.y.0 + pad.0),
                    ),
                    size: Size::new(
                        Px((close_rect.size.width.0 - pad.0 * 2.0).max(1.0)),
                        Px((close_rect.size.height.0 - pad.0 * 2.0).max(1.0)),
                    ),
                };
                scene.push(SceneOp::SvgMaskIcon {
                    order: fret_core::DrawOrder(6),
                    rect,
                    svg,
                    fit: fret_core::SvgFit::Contain,
                    color,
                    opacity: 1.0,
                });
            } else if let Some(glyph) = tab_close_glyph {
                let text_x = Px(close_rect.origin.x.0
                    + (close_rect.size.width.0 - glyph.metrics.size.width.0) * 0.5);
                let inner_y = close_rect.origin.y.0
                    + ((close_rect.size.height.0 - glyph.metrics.size.height.0) * 0.5);
                let text_y = Px(inner_y + glyph.metrics.baseline.0);
                scene.push(SceneOp::Text {
                    order: fret_core::DrawOrder(6),
                    origin: Point::new(text_x, text_y),
                    text: glyph.blob,
                    paint: (color).into(),
                    outline: None,
                    shadow: None,
                });
            }
        }
    }

    scene.push(SceneOp::PopClip);

    if overflow {
        let button_rect = tab_overflow_button_rect(theme.clone(), tab_bar);
        let hovered = input.hovered_tab_overflow_button;
        let open = input.tab_overflow_menu.is_some();
        if hovered || open {
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10),
                rect: button_rect,
                background: fret_core::Paint::Solid(hover_bg).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(radius_sm),
            });
        }
        let color = if hovered || open { fg } else { fg_muted };
        if let Some(svg) = tab_overflow_svg {
            let pad = Px(4.0);
            let rect = Rect {
                origin: Point::new(
                    Px(button_rect.origin.x.0 + pad.0),
                    Px(button_rect.origin.y.0 + pad.0),
                ),
                size: Size::new(
                    Px((button_rect.size.width.0 - pad.0 * 2.0).max(1.0)),
                    Px((button_rect.size.height.0 - pad.0 * 2.0).max(1.0)),
                ),
            };
            scene.push(SceneOp::SvgMaskIcon {
                order: fret_core::DrawOrder(11),
                rect,
                svg,
                fit: fret_core::SvgFit::Contain,
                color,
                opacity: 1.0,
            });
        } else if let Some(glyph) = tab_overflow_glyph {
            let text_x = Px(button_rect.origin.x.0
                + (button_rect.size.width.0 - glyph.metrics.size.width.0) * 0.5);
            let inner_y = button_rect.origin.y.0
                + ((button_rect.size.height.0 - glyph.metrics.size.height.0) * 0.5);
            let text_y = Px(inner_y + glyph.metrics.baseline.0);
            scene.push(SceneOp::Text {
                order: fret_core::DrawOrder(11),
                origin: Point::new(text_x, text_y),
                text: glyph.blob,
                paint: (color).into(),
                outline: None,
                shadow: None,
            });
        }
    }

    if let Some(menu) = input.tab_overflow_menu.as_ref() {
        let item_count = menu.items.len();
        if item_count == 0 {
            return;
        }
        let menu_rect = tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
        let max_scroll = overflow_menu_max_scroll(tab_bar, item_count);
        let scroll = Px(menu.scroll.0.clamp(0.0, max_scroll.0));
        let row_h = overflow_menu_row_height(tab_bar).0;
        if row_h <= 0.0 {
            return;
        }

        let bg = theme.color_token("popover");
        let border = theme.color_token("popover.border");
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(100),
            rect: menu_rect,
            background: fret_core::Paint::Solid(bg).into(),
            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(border).into(),
            corner_radii: fret_core::Corners::all(radius_sm),
        });

        scene.push(SceneOp::PushClipRect { rect: menu_rect });
        let first = (scroll.0 / row_h).floor().max(0.0) as usize;
        let visible = overflow_menu_row_count(item_count);
        for row in 0..visible {
            let idx = first + row;
            let Some(&tab_ix) = menu.items.get(idx) else {
                break;
            };
            let Some(panel) = input.tabs.get(tab_ix) else {
                break;
            };
            let row_rect = overflow_menu_row_rect(menu_rect, tab_bar, scroll, idx);
            let close_rect = overflow_menu_close_rect(theme.clone(), row_rect);

            let is_hovered = menu.hovered == Some(idx);
            let is_active = tab_ix == input.active;
            if is_hovered {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(101),
                    rect: row_rect,
                    background: fret_core::Paint::Solid(hover_bg).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            if let Some(title) = tab_titles.get(panel) {
                let pad_x = theme.metric_token("metric.padding.md");
                let text_clip_w = Px((close_rect.origin.x.0 - row_rect.origin.x.0).max(1.0));
                let text_clip = Rect::new(
                    row_rect.origin,
                    Size::new(text_clip_w, row_rect.size.height),
                );
                let text_x = Px(row_rect.origin.x.0 + pad_x.0);
                let inner_y = row_rect.origin.y.0 + ((row_h - title.metrics.size.height.0) * 0.5);
                let text_y = Px(inner_y + title.metrics.baseline.0);
                let text_color = if is_active { fg } else { fg_muted };
                scene.push(SceneOp::PushClipRect { rect: text_clip });
                scene.push(SceneOp::Text {
                    order: fret_core::DrawOrder(102),
                    origin: Point::new(text_x, text_y),
                    text: title.blob,
                    paint: (text_color).into(),
                    outline: None,
                    shadow: None,
                });
                scene.push(SceneOp::PopClip);
            }

            let close_color = if is_hovered { fg } else { fg_muted };
            if let Some(svg) = tab_close_svg {
                let pad = Px(2.0);
                let rect = Rect {
                    origin: Point::new(
                        Px(close_rect.origin.x.0 + pad.0),
                        Px(close_rect.origin.y.0 + pad.0),
                    ),
                    size: Size::new(
                        Px((close_rect.size.width.0 - pad.0 * 2.0).max(1.0)),
                        Px((close_rect.size.height.0 - pad.0 * 2.0).max(1.0)),
                    ),
                };
                scene.push(SceneOp::SvgMaskIcon {
                    order: fret_core::DrawOrder(103),
                    rect,
                    svg,
                    fit: fret_core::SvgFit::Contain,
                    color: close_color,
                    opacity: 1.0,
                });
            } else if let Some(glyph) = tab_close_glyph {
                let text_x = Px(close_rect.origin.x.0
                    + (close_rect.size.width.0 - glyph.metrics.size.width.0) * 0.5);
                let inner_y = close_rect.origin.y.0
                    + ((close_rect.size.height.0 - glyph.metrics.size.height.0) * 0.5);
                let text_y = Px(inner_y + glyph.metrics.baseline.0);
                scene.push(SceneOp::Text {
                    order: fret_core::DrawOrder(103),
                    origin: Point::new(text_x, text_y),
                    text: glyph.blob,
                    paint: (close_color).into(),
                    outline: None,
                    shadow: None,
                });
            }
        }
        scene.push(SceneOp::PopClip);
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn paint_tab_detail_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[TabDetailPaintInput],
    tab_titles: &HashMap<PanelKey, PreparedTabTitle>,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_overflow_glyph: Option<PreparedTabTitle>,
    tab_close_svg: Option<fret_core::SvgId>,
    tab_overflow_svg: Option<fret_core::SvgId>,
    scene: &mut Scene,
) {
    for input in inputs {
        paint_tab_detail_input(
            theme.clone(),
            input,
            tab_titles,
            tab_close_glyph,
            tab_overflow_glyph,
            tab_close_svg,
            tab_overflow_svg,
            scene,
        );
    }
}
