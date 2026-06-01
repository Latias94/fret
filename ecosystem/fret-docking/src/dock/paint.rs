// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::hit_test::{tab_close_rect, tab_scroll_for_node};
use super::layout::{drop_zone_rect, split_tab_bar};
use super::manager::DockManager;
use super::prelude_core::*;
use super::split_geometry::{self, SplitHandle};
use super::tab_bar_geometry::TabBarGeometry;
use super::tab_bar_geometry::dock_tab_width_for_title;
use super::tab_overflow::{
    TabOverflowMenuState, overflow_menu_close_rect, overflow_menu_max_scroll,
    overflow_menu_row_count, overflow_menu_row_height, overflow_menu_row_rect,
    tab_overflow_button_rect, tab_overflow_menu_rect, tab_strip_rect_with_overflow_button,
};

mod drop_hints;

pub(super) use drop_hints::paint_drop_hints;

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

    // Keep at least a 1px content span so text doesn't disappear entirely under theme metric
    // misconfiguration (e.g. overly large padding).
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

pub(super) struct DockDragGhostPaint {
    pub(super) position: Point,
    pub(super) grab_offset: Point,
    pub(super) title: PreparedTabTitle,
}

#[derive(Debug, Clone)]
pub(super) struct TabChromePaintInput {
    pub(super) rect: Rect,
    pub(super) tabs_len: usize,
    pub(super) active: usize,
    pub(super) tab_widths: Option<Arc<[Px]>>,
    pub(super) scroll: Px,
    pub(super) hovered_tab: Option<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct TabDetailPaintInput {
    pub(super) rect: Rect,
    pub(super) tabs: Arc<[PanelKey]>,
    pub(super) active: usize,
    pub(super) tab_widths: Option<Arc<[Px]>>,
    pub(super) scroll: Px,
    pub(super) hovered_tab: Option<usize>,
    pub(super) hovered_tab_close: bool,
    pub(super) hovered_tab_overflow_button: bool,
    pub(super) pressed_tab_close: Option<usize>,
    pub(super) tab_overflow_menu: Option<TabOverflowMenuState>,
}

pub(super) fn tab_chrome_paint_inputs(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
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

pub(super) fn tab_detail_paint_inputs(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
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

pub(super) fn paint_tab_chrome_input(
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

pub(super) fn paint_tab_chrome_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[TabChromePaintInput],
    scene: &mut Scene,
) {
    for input in inputs {
        paint_tab_chrome_input(theme.clone(), input, scene);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_tab_detail_input(
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
    let pad_md = theme.metric_token("metric.padding.md");
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
                let pad_x = pad_md;
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

            // Close button (always visible) - clicking this in the overflow menu should not activate the tab.
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
pub(super) fn paint_tab_detail_inputs(
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

#[derive(Debug, Clone)]
pub(super) struct ViewportSurfacePaintInput {
    panel: PanelKey,
    panel_color: Color,
    viewport: ViewportPanel,
    layout: super::DockViewportLayout,
}

fn viewport_surface_paint_input(
    dock: &DockManager,
    window: fret_core::AppWindowId,
    panel_key: &PanelKey,
    panel: &DockPanel,
    content: Rect,
) -> Option<ViewportSurfacePaintInput> {
    let viewport = panel.viewport?;
    let layout = dock
        .viewport_layout(window, viewport.target)
        .filter(|layout| layout.content_rect == content)
        .unwrap_or_else(|| {
            let mapping = ViewportMapping {
                content_rect: content,
                target_px_size: viewport.target_px_size,
                fit: viewport.fit,
            };
            super::DockViewportLayout {
                content_rect: content,
                mapping,
                draw_rect: mapping.map().draw_rect,
            }
        });

    Some(ViewportSurfacePaintInput {
        panel: panel_key.clone(),
        panel_color: panel.color,
        viewport,
        layout,
    })
}

pub(super) fn viewport_surface_paint_inputs(
    dock: &DockManager,
    window: fret_core::AppWindowId,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
) -> Vec<ViewportSurfacePaintInput> {
    let mut inputs = Vec::new();
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = dock.graph.node(node_id) else {
            continue;
        };
        let Some(panel_key) = tabs.get(*active) else {
            continue;
        };
        let Some(panel) = dock.panel(panel_key) else {
            continue;
        };
        let (_tab_bar, content) = split_tab_bar(rect);
        if let Some(input) = viewport_surface_paint_input(dock, window, panel_key, panel, content) {
            inputs.push(input);
        }
    }
    inputs
}

pub(super) fn paint_viewport_surface_input(
    theme: fret_ui::ThemeSnapshot,
    window: fret_core::AppWindowId,
    input: &ViewportSurfacePaintInput,
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    let content = input.layout.content_rect;
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(3),
        rect: content,
        background: fret_core::Paint::Solid(input.panel_color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::all(theme.metric_token("metric.radius.sm")),
    });

    scene.push(SceneOp::PushClipRect { rect: content });
    scene.push(SceneOp::ViewportSurface {
        order: fret_core::DrawOrder(4),
        rect: input.layout.draw_rect,
        target: input.viewport.target,
        opacity: 1.0,
    });
    if let Some(hooks) = overlay_hooks {
        hooks.paint_with_layout(
            theme,
            window,
            &input.panel,
            input.viewport,
            input.layout,
            scene,
        );
    }
    scene.push(SceneOp::PopClip);
}

pub(super) fn paint_viewport_surface_inputs(
    theme: fret_ui::ThemeSnapshot,
    window: fret_core::AppWindowId,
    inputs: &[ViewportSurfacePaintInput],
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    for input in inputs {
        paint_viewport_surface_input(theme.clone(), window, input, overlay_hooks, scene);
    }
}

#[derive(Debug, Clone)]
pub(super) struct FloatingChromePaintInput {
    pub(super) outer: Rect,
    pub(super) title_bar: Rect,
    pub(super) close_button: Rect,
    pub(super) title_bar_hovered: bool,
    pub(super) close_hovered: bool,
    pub(super) close_pressed: bool,
}

pub(super) fn paint_floating_chrome_input(
    theme: fret_ui::ThemeSnapshot,
    input: &FloatingChromePaintInput,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_close_svg: Option<fret_core::SvgId>,
    scene: &mut Scene,
) {
    let border = theme.color_token("border");
    let surface = theme.color_token("background");
    let hover_bg = theme.color_token("accent");
    let fg = theme.color_token("foreground");
    let fg_muted = theme.color_token("muted-foreground");
    let radius_md = theme.metric_token("metric.radius.md");
    let radius_sm = theme.metric_token("metric.radius.sm");

    let border_color = Color { a: 0.85, ..border };
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(0),
        rect: input.outer,
        background: fret_core::Paint::Solid(surface).into(),
        border: Edges::all(DOCK_FLOATING_BORDER),
        border_paint: fret_core::Paint::Solid(border_color).into(),
        corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
    });
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(1),
        rect: input.title_bar,
        background: fret_core::Paint::Solid(if input.title_bar_hovered {
            Color {
                a: 0.22,
                ..hover_bg
            }
        } else {
            surface
        })
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    if input.close_hovered || input.close_pressed {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(2),
            rect: input.close_button,
            background: fret_core::Paint::Solid(hover_bg).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
        });
    }

    let color = if input.close_hovered || input.close_pressed {
        fg
    } else {
        fg_muted
    };
    if let Some(svg) = tab_close_svg {
        let pad = Px(1.0);
        let rect = Rect {
            origin: Point::new(
                Px(input.close_button.origin.x.0 + pad.0),
                Px(input.close_button.origin.y.0 + pad.0),
            ),
            size: Size::new(
                Px((input.close_button.size.width.0 - pad.0 * 2.0).max(1.0)),
                Px((input.close_button.size.height.0 - pad.0 * 2.0).max(1.0)),
            ),
        };
        scene.push(SceneOp::SvgMaskIcon {
            order: fret_core::DrawOrder(3),
            rect,
            svg,
            fit: fret_core::SvgFit::Contain,
            color,
            opacity: 1.0,
        });
    } else if let Some(glyph) = tab_close_glyph {
        let text_x = Px(input.close_button.origin.x.0
            + (input.close_button.size.width.0 - glyph.metrics.size.width.0) * 0.5);
        let inner_y = input.close_button.origin.y.0
            + ((input.close_button.size.height.0 - glyph.metrics.size.height.0) * 0.5);
        let text_y = Px(inner_y + glyph.metrics.baseline.0);
        scene.push(SceneOp::Text {
            order: fret_core::DrawOrder(3),
            origin: Point::new(text_x, text_y),
            text: glyph.blob,
            paint: (color).into(),
            outline: None,
            shadow: None,
        });
    }
}

pub(super) fn paint_floating_chrome_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[FloatingChromePaintInput],
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_close_svg: Option<fret_core::SvgId>,
    scene: &mut Scene,
) {
    for input in inputs {
        paint_floating_chrome_input(theme.clone(), input, tab_close_glyph, tab_close_svg, scene);
    }
}

#[derive(Debug, Clone)]
pub(super) struct SplitHandlePaintInput {
    node: DockNodeId,
    axis: fret_core::Axis,
    bounds: Rect,
    children_len: usize,
    fractions: Vec<f32>,
}

pub(super) fn split_handle_paint_inputs(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
) -> Vec<SplitHandlePaintInput> {
    let mut inputs = Vec::new();
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split {
            axis,
            children,
            fractions,
        }) = graph.node(node)
        else {
            continue;
        };
        if children.len() < 2 {
            continue;
        }
        inputs.push(SplitHandlePaintInput {
            node,
            axis: *axis,
            bounds,
            children_len: children.len(),
            fractions: fractions.clone(),
        });
    }
    inputs
}

pub(super) fn paint_split_handle_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[SplitHandlePaintInput],
    active: Option<DockNodeId>,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    scale_factor: f32,
    scene: &mut Scene,
) {
    for input in inputs {
        let computed = split_geometry::compute_layout(
            input.axis,
            input.bounds,
            input.children_len,
            &input.fractions,
            split_handle_gap,
            split_handle_hit_thickness,
            &[],
        );

        let background = if active == Some(input.node) {
            theme.color_token("ring")
        } else {
            theme.color_token("border")
        };

        let handle = SplitHandle {
            axis: input.axis,
            paint_device_px: 1.0,
        };
        for center in computed.handle_centers {
            handle.paint(
                scene,
                // Keep split handle under component focus rings (typically DrawOrder(1)),
                // while still painting above panel backgrounds (DrawOrder(0)).
                fret_core::DrawOrder(0),
                input.bounds,
                center,
                scale_factor,
                background,
            );
        }
    }
}

pub(super) fn paint_drag_payload_ghost(
    theme: fret_ui::ThemeSnapshot,
    ghost: Option<&DockDragGhostPaint>,
    close_glyph_present: bool,
    scene: &mut Scene,
) {
    let Some(ghost) = ghost else {
        return;
    };

    let width = dock_tab_width_for_title(
        theme.clone(),
        ghost.title.metrics.size.width,
        close_glyph_present,
    );
    let rect = Rect::new(
        Point::new(
            Px(ghost.position.x.0 - ghost.grab_offset.x.0),
            Px(ghost.position.y.0 - ghost.grab_offset.y.0),
        ),
        Size::new(width, DOCK_TAB_H),
    );

    let card = theme.color_token("card");
    let border = theme.color_token("border");
    let fg = theme.color_token("foreground");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let clip = tab_title_clip_rect(theme.clone(), rect, close_glyph_present);

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(10_020),
        rect,
        background: fret_core::Paint::Solid(Color { a: 0.94, ..card }).into(),
        border: Edges::all(Px(1.0)),
        border_paint: fret_core::Paint::Solid(Color { a: 0.88, ..border }).into(),
        corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
    });

    let inner_y =
        rect.origin.y.0 + ((rect.size.height.0 - ghost.title.metrics.size.height.0) * 0.5);
    let text_y = Px(inner_y + ghost.title.metrics.baseline.0);
    scene.push(SceneOp::PushClipRect { rect: clip });
    scene.push(SceneOp::Text {
        order: fret_core::DrawOrder(10_021),
        origin: Point::new(clip.origin.x, text_y),
        text: ghost.title.blob,
        paint: (Color { a: 0.96, ..fg }).into(),
        outline: None,
        shadow: None,
    });
    scene.push(SceneOp::PopClip);
}

pub(super) fn paint_basic_drop_overlay(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    drag_source_tabs: Option<DockNodeId>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    let primary = theme.color_token("primary");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let radius_md = theme.metric_token("metric.radius.md");
    let primary_alpha = |alpha: f32| Color {
        a: alpha,
        ..primary
    };

    let overlay_float_bg = theme
        .color_by_key("component.docking.drop_overlay.float.bg")
        .unwrap_or_else(|| primary_alpha(0.10));
    let overlay_float_border = theme
        .color_by_key("component.docking.drop_overlay.float.border")
        .unwrap_or_else(|| primary_alpha(0.85));
    let overlay_empty_bg = theme
        .color_by_key("component.docking.drop_overlay.empty.bg")
        .unwrap_or_else(|| primary_alpha(0.08));
    let overlay_empty_border = theme
        .color_by_key("component.docking.drop_overlay.empty.border")
        .unwrap_or_else(|| primary_alpha(0.75));
    let overlay_center_content_bg = theme
        .color_by_key("component.docking.drop_overlay.center.content.bg")
        .unwrap_or_else(|| primary_alpha(0.12));
    let overlay_center_content_border = theme
        .color_by_key("component.docking.drop_overlay.center.content.border")
        .unwrap_or_else(|| primary_alpha(0.65));
    let overlay_center_tab_bar_bg = theme
        .color_by_key("component.docking.drop_overlay.center.tab_bar.bg")
        .unwrap_or_else(|| primary_alpha(0.14));
    let overlay_center_tab_bar_border = theme
        .color_by_key("component.docking.drop_overlay.center.tab_bar.border")
        .unwrap_or_else(|| primary_alpha(0.45));

    match target {
        DockDropTarget::Float { window: w } => {
            if w != window {
                return;
            }
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: bounds,
                background: fret_core::Paint::Solid(overlay_float_bg).into(),
                border: Edges::all(Px(3.0)),
                border_paint: fret_core::Paint::Solid(overlay_float_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::EmptyDockSpace { window: w } => {
            if w != window {
                return;
            }
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: bounds,
                background: fret_core::Paint::Solid(overlay_empty_bg).into(),
                border: Edges::all(Px(3.0)),
                border_paint: fret_core::Paint::Solid(overlay_empty_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::Dock(target) if target.zone == DropZone::Center => {
            let Some(rect) = layout.get(&target.tabs).copied() else {
                return;
            };
            let same_tabs_reorder = drag_source_tabs.is_some_and(|src| src == target.tabs);
            if same_tabs_reorder {
                return;
            }
            let (tab_bar, content) = split_tab_bar(rect);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(9_985),
                rect: content,
                background: fret_core::Paint::Solid(overlay_center_content_bg).into(),
                border: Edges::all(Px(2.0)),
                border_paint: fret_core::Paint::Solid(overlay_center_content_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
            });
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(9_990),
                rect: tab_bar,
                background: fret_core::Paint::Solid(overlay_center_tab_bar_bg).into(),
                border: Edges::all(Px(1.0)),
                border_paint: fret_core::Paint::Solid(overlay_center_tab_bar_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
            });
        }
        DockDropTarget::Dock(_) => {}
    }
}

#[derive(Debug, Clone)]
pub(super) enum ComplexDropOverlayPaintInput {
    TabInsertMarker { marker: Rect, caps: [Rect; 2] },
    EdgeZone { overlay: Rect },
}

#[allow(clippy::too_many_arguments)]
pub(super) fn complex_drop_overlay_paint_inputs(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
) -> Vec<ComplexDropOverlayPaintInput> {
    let Some(DockDropTarget::Dock(target)) = target else {
        return Vec::new();
    };
    let Some(rect) = layout.get(&target.tabs).copied() else {
        return Vec::new();
    };

    if target.zone == DropZone::Center {
        let Some(i) = target.insert_index else {
            return Vec::new();
        };
        let (tab_bar, _content) = split_tab_bar(rect);
        let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
        let tab_count = match graph.node(target.tabs) {
            Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
            _ => 0,
        };
        let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
        let geom_candidate = tab_widths
            .get(&target.tabs)
            .filter(|w| w.len() == tab_count)
            .map(|w| TabBarGeometry::variable(strip_candidate, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, tab_count));
        let overflow = geom_candidate.max_scroll().0 > 0.0;
        let geom = if overflow {
            geom_candidate
        } else {
            tab_widths
                .get(&target.tabs)
                .filter(|w| w.len() == tab_count)
                .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count))
        };
        let tab_strip = if overflow { strip_candidate } else { tab_bar };
        let x = geom.insert_x(i.min(tab_count), scroll).0;
        let marker = Rect::new(
            Point::new(Px(x - 3.0), Px(tab_strip.origin.y.0 + 3.0)),
            Size::new(Px(6.0), Px((tab_strip.size.height.0 - 6.0).max(0.0))),
        );
        let cap_w = Px(14.0);
        let cap_h = Px(3.0);
        let cap_x = Px(x - cap_w.0 * 0.5);
        let cap_top = Rect::new(Point::new(cap_x, marker.origin.y), Size::new(cap_w, cap_h));
        let cap_bottom = Rect::new(
            Point::new(
                cap_x,
                Px(marker.origin.y.0 + marker.size.height.0 - cap_h.0),
            ),
            Size::new(cap_w, cap_h),
        );
        return vec![ComplexDropOverlayPaintInput::TabInsertMarker {
            marker,
            caps: [cap_top, cap_bottom],
        }];
    }

    let overlay = match graph.edge_dock_decision(window, target.tabs, target.zone) {
        Some(EdgeDockDecision::InsertIntoSplit {
            split,
            anchor_index,
            insert_index,
        }) => {
            let preview = layout.get(&split).copied().and_then(|bounds| {
                let Some(DockNode::Split {
                    axis,
                    children,
                    fractions,
                }) = graph.node(split)
                else {
                    return None;
                };
                if children.is_empty() || children.len() != fractions.len() {
                    return None;
                }

                let mut next = fractions.clone();
                let old = *next.get(anchor_index)?;
                let keep = old * 0.5;
                let take = old * 0.5;

                next[anchor_index] = keep;
                next.insert(insert_index.min(next.len()), take);

                let computed = split_geometry::compute_layout(
                    *axis,
                    bounds,
                    children.len().saturating_add(1),
                    &next,
                    split_handle_gap,
                    split_handle_hit_thickness,
                    &[],
                );
                computed.panel_rects.get(insert_index).copied()
            });
            preview.unwrap_or_else(|| drop_zone_rect(rect, target.zone))
        }
        _ => drop_zone_rect(rect, target.zone),
    };
    vec![ComplexDropOverlayPaintInput::EdgeZone { overlay }]
}

pub(super) fn paint_complex_drop_overlay_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[ComplexDropOverlayPaintInput],
    scene: &mut Scene,
) {
    let primary = theme.color_token("primary");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let primary_alpha = |alpha: f32| Color {
        a: alpha,
        ..primary
    };
    let overlay_zone_bg = theme
        .color_by_key("component.docking.drop_overlay.zone.bg")
        .unwrap_or_else(|| primary_alpha(0.16));
    let overlay_zone_border = theme
        .color_by_key("component.docking.drop_overlay.zone.border")
        .unwrap_or_else(|| primary_alpha(0.85));

    let tab_insert_marker_bg = theme
        .color_by_key("component.docking.tab_insert.marker.bg")
        .unwrap_or_else(|| primary_alpha(0.85));
    let tab_insert_marker_border = theme
        .color_by_key("component.docking.tab_insert.marker.border")
        .unwrap_or_else(|| primary_alpha(1.0));
    let tab_insert_marker_cap_bg = theme
        .color_by_key("component.docking.tab_insert.marker.cap.bg")
        .unwrap_or_else(|| primary_alpha(0.92));

    for input in inputs {
        match input {
            ComplexDropOverlayPaintInput::TabInsertMarker { marker, caps } => {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(10_000),
                    rect: *marker,
                    background: fret_core::Paint::Solid(tab_insert_marker_bg).into(),
                    border: Edges::all(Px(1.0)),
                    border_paint: fret_core::Paint::Solid(tab_insert_marker_border).into(),
                    corner_radii: fret_core::Corners::all(Px(3.0)),
                });

                for &cap in caps {
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(10_001),
                        rect: cap,
                        background: fret_core::Paint::Solid(tab_insert_marker_cap_bg).into(),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),
                        corner_radii: fret_core::Corners::all(Px(2.0)),
                    });
                }
            }
            ComplexDropOverlayPaintInput::EdgeZone { overlay } => {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(10_000),
                    rect: *overlay,
                    background: fret_core::Paint::Solid(overlay_zone_bg).into(),
                    border: Edges::all(Px(2.0)),
                    border_paint: fret_core::Paint::Solid(overlay_zone_border).into(),
                    corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_tab_insert_preview_title(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_count: usize,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    drag_source_tabs: Option<DockNodeId>,
    drag_tab_title: Option<&PreparedTabTitle>,
    close_glyph_present: bool,
    scene: &mut Scene,
) {
    let Some(DockDropTarget::Dock(target)) = target else {
        return;
    };
    if target.zone != DropZone::Center {
        return;
    }
    let Some(title) = drag_tab_title else {
        return;
    };
    if drag_source_tabs.is_some_and(|src| src == target.tabs) {
        return;
    }
    let Some(rect) = layout.get(&target.tabs).copied() else {
        return;
    };

    let primary = theme.color_token("primary");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let primary_alpha = |alpha: f32| Color {
        a: alpha,
        ..primary
    };
    let tab_insert_preview_bg = theme
        .color_by_key("component.docking.tab_insert.preview.bg")
        .unwrap_or_else(|| primary_alpha(0.22));
    let tab_insert_preview_border = theme
        .color_by_key("component.docking.tab_insert.preview.border")
        .unwrap_or_else(|| primary_alpha(0.85));

    let (tab_bar, _content) = split_tab_bar(rect);
    let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
    let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let geom_candidate = tab_widths
        .get(&target.tabs)
        .filter(|w| w.len() == tab_count)
        .map(|w| TabBarGeometry::variable(strip_candidate, w.clone()))
        .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, tab_count));
    let overflow = geom_candidate.max_scroll().0 > 0.0;
    let geom = if overflow {
        geom_candidate
    } else {
        tab_widths
            .get(&target.tabs)
            .filter(|w| w.len() == tab_count)
            .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count))
    };
    let tab_strip = if overflow { strip_candidate } else { tab_bar };

    let insert_index = target.insert_index.unwrap_or(tab_count);
    let mut x = geom.insert_x(insert_index.min(tab_count), scroll).0;
    let mut w =
        dock_tab_width_for_title(theme.clone(), title.metrics.size.width, close_glyph_present).0;

    let min_x = tab_strip.origin.x.0;
    let max_x = tab_strip.origin.x.0 + tab_strip.size.width.0;
    if x < min_x {
        x = min_x;
    }
    if x > max_x {
        x = max_x;
    }
    w = w.max(0.0).min((max_x - x).max(0.0));
    if w <= 6.0 {
        return;
    }

    let preview = Rect::new(
        Point::new(Px(x), tab_strip.origin.y),
        Size::new(Px(w), tab_strip.size.height),
    );
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(9_995),
        rect: preview,
        background: fret_core::Paint::Solid(tab_insert_preview_bg).into(),
        border: Edges::all(Px(2.0)),
        border_paint: fret_core::Paint::Solid(tab_insert_preview_border).into(),
        corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
    });

    let pad_x = theme.metric_token("metric.padding.md");
    let text_x = Px(preview.origin.x.0 + pad_x.0.max(0.0));
    let inner_y =
        preview.origin.y.0 + ((preview.size.height.0 - title.metrics.size.height.0) * 0.5);
    let text_y = Px(inner_y + title.metrics.baseline.0);
    let fg = theme.color_token("foreground");
    scene.push(SceneOp::PushClipRect { rect: preview });
    scene.push(SceneOp::Text {
        order: fret_core::DrawOrder(9_996),
        origin: Point::new(text_x, text_y),
        text: title.blob,
        paint: (Color { a: 0.92, ..fg }).into(),
        outline: None,
        shadow: None,
    });
    scene.push(SceneOp::PopClip);
}
