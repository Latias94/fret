// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::hit_test::{tab_close_rect, tab_scroll_for_node};
use super::layout::{dock_hint_rects_with_font, drop_zone_rect, split_tab_bar};
use super::manager::DockManager;
use super::prelude_core::*;
use super::tab_bar_geometry::TabBarGeometry;
use super::tab_bar_geometry::dock_tab_width_for_title;
use fret_ui::retained_bridge::ResizeHandle;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

pub(super) struct PaintDockParams<'a> {
    pub(super) window: fret_core::AppWindowId,
    pub(super) layout: &'a std::collections::HashMap<DockNodeId, Rect>,
    pub(super) tab_titles: &'a HashMap<PanelKey, PreparedTabTitle>,
    pub(super) tab_widths: &'a HashMap<DockNodeId, Arc<[Px]>>,
    pub(super) hovered_tab: Option<(DockNodeId, usize)>,
    pub(super) hovered_tab_close: bool,
    pub(super) pressed_tab_close: Option<(DockNodeId, usize)>,
    pub(super) tab_scroll: &'a HashMap<DockNodeId, Px>,
    pub(super) tab_close_glyph: Option<PreparedTabTitle>,
}

pub(super) fn paint_dock(
    theme: fret_ui::ThemeSnapshot,
    dock: &DockManager,
    params: PaintDockParams<'_>,
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    let panel_bg = theme.color_required("card");
    let surface_bg = theme.color_required("background");
    let hover_bg = theme.color_required("accent");
    let primary = theme.color_required("primary");
    let fg = theme.color_required("foreground");
    let fg_muted = theme.color_required("muted-foreground");
    let pad_md = theme.metric_required("metric.padding.md");
    let radius_sm = theme.metric_required("metric.radius.sm");

    let PaintDockParams {
        window,
        layout,
        tab_titles,
        tab_widths,
        hovered_tab,
        hovered_tab_close,
        pressed_tab_close,
        tab_scroll,
        tab_close_glyph,
    } = params;
    let graph = &dock.graph;
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (tab_bar, content) = split_tab_bar(rect);

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect,
            background: panel_bg,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(1),
            rect: tab_bar,
            background: surface_bg,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let scroll = tab_scroll_for_node(tab_scroll, node_id);
        let tab_geom = tab_widths
            .get(&node_id)
            .filter(|w| w.len() == tabs.len())
            .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tabs.len()));
        scene.push(SceneOp::PushClipRect { rect: tab_bar });

        for (i, panel) in tabs.iter().enumerate() {
            let tab_rect = tab_geom.tab_rect(i, scroll);
            if tab_rect.origin.x.0 + tab_rect.size.width.0 < tab_bar.origin.x.0
                || tab_rect.origin.x.0 > tab_bar.origin.x.0 + tab_bar.size.width.0
            {
                continue;
            }

            let is_active = i == *active;
            let is_hovered = hovered_tab == Some((node_id, i));
            let bg = if is_active {
                panel_bg
            } else if is_hovered {
                hover_bg
            } else {
                Color { a: 0.0, ..panel_bg }
            };

            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(2),
                rect: tab_rect,
                background: bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
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
                    background: primary,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            if let Some(title) = tab_titles.get(panel) {
                let pad_x = pad_md;
                let text_x = Px(tab_rect.origin.x.0 + pad_x.0);
                let inner_y = tab_rect.origin.y.0
                    + ((tab_rect.size.height.0 - title.metrics.size.height.0) * 0.5);
                let text_y = Px(inner_y + title.metrics.baseline.0);
                let text_color = if is_active || is_hovered {
                    fg
                } else {
                    fg_muted
                };

                scene.push(SceneOp::PushClipRect { rect: tab_rect });
                scene.push(SceneOp::Text {
                    order: fret_core::DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: title.blob,
                    color: text_color,
                });
                scene.push(SceneOp::PopClip);
            }

            if (is_active || is_hovered) && tab_close_glyph.is_some() {
                let close_rect = tab_close_rect(theme, tab_rect);
                let close_hovered = is_hovered && hovered_tab_close;
                let close_pressed = pressed_tab_close == Some((node_id, i));

                if close_pressed || close_hovered {
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(5),
                        rect: close_rect,
                        background: hover_bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(radius_sm),
                    });
                }

                if let Some(glyph) = tab_close_glyph {
                    let text_x = Px(close_rect.origin.x.0
                        + (close_rect.size.width.0 - glyph.metrics.size.width.0) * 0.5);
                    let inner_y = close_rect.origin.y.0
                        + ((close_rect.size.height.0 - glyph.metrics.size.height.0) * 0.5);
                    let text_y = Px(inner_y + glyph.metrics.baseline.0);
                    let color = if close_pressed || close_hovered {
                        fg
                    } else {
                        fg_muted
                    };
                    scene.push(SceneOp::Text {
                        order: fret_core::DrawOrder(6),
                        origin: Point::new(text_x, text_y),
                        text: glyph.blob,
                        color,
                    });
                }
            }
        }

        scene.push(SceneOp::PopClip);

        let active_panel = tabs.get(*active);
        if let Some(panel) = active_panel.and_then(|p| dock.panel(p)) {
            if let Some(vp) = panel.viewport {
                let layout = dock
                    .viewport_layout(window, vp.target)
                    .filter(|layout| layout.content_rect == content)
                    .unwrap_or_else(|| {
                        let mapping = ViewportMapping {
                            content_rect: content,
                            target_px_size: vp.target_px_size,
                            fit: vp.fit,
                        };
                        super::DockViewportLayout {
                            content_rect: content,
                            mapping,
                            draw_rect: mapping.map().draw_rect,
                        }
                    });
                let draw_rect = layout.draw_rect;

                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(radius_sm),
                });

                scene.push(SceneOp::PushClipRect { rect: content });
                scene.push(SceneOp::ViewportSurface {
                    order: fret_core::DrawOrder(4),
                    rect: draw_rect,
                    target: vp.target,
                    opacity: 1.0,
                });
                if let Some(hooks) = overlay_hooks
                    && let Some(panel_key) = active_panel
                {
                    hooks.paint_with_layout(theme, window, panel_key, vp, layout, scene);
                }
                scene.push(SceneOp::PopClip);
            } else {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(radius_sm),
                });
            }
        }
    }
}

pub(super) fn paint_split_handles(
    theme: fret_ui::ThemeSnapshot,
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    active: Option<DockNodeId>,
    scale_factor: f32,
    scene: &mut Scene,
) {
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
        let computed = resizable::compute_layout(
            *axis,
            bounds,
            children.len(),
            fractions,
            DOCK_SPLIT_HANDLE_GAP,
            DOCK_SPLIT_HANDLE_HIT_THICKNESS,
            &[],
        );

        let background = if active == Some(node) {
            theme.color_required("ring")
        } else {
            theme.color_required("border")
        };

        let handle = ResizeHandle {
            axis: *axis,
            hit_thickness: DOCK_SPLIT_HANDLE_HIT_THICKNESS,
            paint_device_px: 1.0,
        };
        for center in computed.handle_centers {
            handle.paint(
                scene,
                // Keep split handle under component focus rings (typically DrawOrder(1)),
                // while still painting above panel backgrounds (DrawOrder(0)).
                fret_core::DrawOrder(0),
                bounds,
                center,
                scale_factor,
                background,
            );
        }
    }
}

pub(super) fn paint_drop_overlay(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    bounds: Rect,
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    drag_tab_title: Option<PreparedTabTitle>,
    close_glyph_present: bool,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    let primary = theme.color_required("primary");
    let radius_sm = theme.metric_required("metric.radius.sm");
    let radius_md = theme.metric_required("metric.radius.md");

    match target {
        DockDropTarget::Float { window: w } => {
            if w != window {
                return;
            }
            let zone = bounds;
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: zone,
                background: Color { a: 0.10, ..primary },
                border: Edges::all(Px(3.0)),
                border_color: Color { a: 0.85, ..primary },
                corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::Dock(target) => {
            let Some(rect) = layout.get(&target.tabs).copied() else {
                return;
            };

            if target.zone == DropZone::Center {
                let (tab_bar, content) = split_tab_bar(rect);
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(9_985),
                    rect: content,
                    background: Color { a: 0.12, ..primary },
                    border: Edges::all(Px(2.0)),
                    border_color: Color { a: 0.65, ..primary },
                    corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
                });
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(9_990),
                    rect: tab_bar,
                    background: Color { a: 0.14, ..primary },
                    border: Edges::all(Px(1.0)),
                    border_color: Color { a: 0.45, ..primary },
                    corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
                });
                if let Some(title) = drag_tab_title {
                    let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
                    let tab_count = match graph.node(target.tabs) {
                        Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
                        _ => 0,
                    };
                    let geom = tab_widths
                        .get(&target.tabs)
                        .filter(|w| w.len() == tab_count)
                        .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                        .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));

                    let insert_index = target.insert_index.unwrap_or(tab_count);
                    let mut x = geom.insert_x(insert_index.min(tab_count), scroll).0;
                    let mut w = dock_tab_width_for_title(
                        theme,
                        title.metrics.size.width,
                        close_glyph_present,
                    )
                    .0;

                    let min_x = tab_bar.origin.x.0;
                    let max_x = tab_bar.origin.x.0 + tab_bar.size.width.0;
                    if x < min_x {
                        x = min_x;
                    }
                    if x > max_x {
                        x = max_x;
                    }
                    w = w.max(0.0).min((max_x - x).max(0.0));
                    if w > 6.0 {
                        let preview = Rect::new(
                            Point::new(Px(x), tab_bar.origin.y),
                            Size::new(Px(w), tab_bar.size.height),
                        );
                        scene.push(SceneOp::Quad {
                            order: fret_core::DrawOrder(9_995),
                            rect: preview,
                            background: Color { a: 0.22, ..primary },
                            border: Edges::all(Px(2.0)),
                            border_color: Color { a: 0.85, ..primary },
                            corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
                        });

                        let pad_x = theme.metric_required("metric.padding.md");
                        let text_x = Px(preview.origin.x.0 + pad_x.0.max(0.0));
                        let inner_y = preview.origin.y.0
                            + ((preview.size.height.0 - title.metrics.size.height.0) * 0.5);
                        let text_y = Px(inner_y + title.metrics.baseline.0);
                        let fg = theme.color_required("foreground");
                        scene.push(SceneOp::PushClipRect { rect: preview });
                        scene.push(SceneOp::Text {
                            order: fret_core::DrawOrder(9_996),
                            origin: Point::new(text_x, text_y),
                            text: title.blob,
                            color: Color { a: 0.92, ..fg },
                        });
                        scene.push(SceneOp::PopClip);
                    }
                }
                if let Some(i) = target.insert_index {
                    let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
                    let tab_count = match graph.node(target.tabs) {
                        Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
                        _ => 0,
                    };
                    let geom = tab_widths
                        .get(&target.tabs)
                        .filter(|w| w.len() == tab_count)
                        .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                        .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));
                    let x = geom.insert_x(i.min(tab_count), scroll).0;
                    let marker = Rect::new(
                        Point::new(Px(x - 3.0), Px(tab_bar.origin.y.0 + 3.0)),
                        Size::new(Px(6.0), Px((tab_bar.size.height.0 - 6.0).max(0.0))),
                    );
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(10_000),
                        rect: marker,
                        background: Color { a: 0.85, ..primary },
                        border: Edges::all(Px(1.0)),
                        border_color: Color { a: 1.0, ..primary },
                        corner_radii: fret_core::Corners::all(Px(3.0)),
                    });

                    let cap_w = Px(14.0);
                    let cap_h = Px(3.0);
                    let cap_x = Px(x - cap_w.0 * 0.5);
                    let cap_top =
                        Rect::new(Point::new(cap_x, marker.origin.y), Size::new(cap_w, cap_h));
                    let cap_bottom = Rect::new(
                        Point::new(
                            cap_x,
                            Px(marker.origin.y.0 + marker.size.height.0 - cap_h.0),
                        ),
                        Size::new(cap_w, cap_h),
                    );
                    for cap in [cap_top, cap_bottom] {
                        scene.push(SceneOp::Quad {
                            order: fret_core::DrawOrder(10_001),
                            rect: cap,
                            background: Color { a: 0.92, ..primary },
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: fret_core::Corners::all(Px(2.0)),
                        });
                    }
                }
                return;
            }

            let overlay = drop_zone_rect(rect, target.zone);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: overlay,
                background: Color { a: 0.16, ..primary },
                border: Edges::all(Px(2.0)),
                border_color: Color { a: 0.85, ..primary },
                corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
            });
        }
    }
}

pub(super) fn paint_drop_hints(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    _window: fret_core::AppWindowId,
    _bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    let DockDropTarget::Dock(target) = target else {
        return;
    };

    let Some(active_rect) = layout.get(&target.tabs).copied() else {
        return;
    };

    let font_size = theme.metric_required("font.size");
    let inner_rect = layout
        .get(&target.leaf_tabs)
        .copied()
        .unwrap_or(active_rect);
    let root_rect = layout.get(&target.root).copied().unwrap_or(active_rect);

    let show_outer = target.root != target.leaf_tabs;
    let inner_rects = dock_hint_rects_with_font(inner_rect, font_size, false);
    let outer_rects = show_outer.then(|| dock_hint_rects_with_font(root_rect, font_size, true));

    let inactive_bg_base = theme.color_required("card");
    let inactive_border_base = theme.color_required("border");
    let active_base = theme.color_required("primary");
    let surface_bg = theme.color_required("background");
    let radius_sm = theme.metric_required("metric.radius.sm");
    let radius_md = theme.metric_required("metric.radius.md");
    let pad_sm = theme.metric_required("metric.padding.sm");

    let inactive_bg = Color {
        a: 0.64,
        ..inactive_bg_base
    };
    let inactive_border = Color {
        a: 0.95,
        ..inactive_border_base
    };
    let active_bg = Color {
        a: 0.92,
        ..active_base
    };
    let active_border = Color {
        a: 1.0,
        ..active_base
    };

    let order = fret_core::DrawOrder(9_500);
    let border = Edges::all(Px(2.0));
    let corner_radii = fret_core::Corners::all(Px(radius_sm.0.max(4.0)));

    // Draw a plate behind the inner 5-way pad, closer to ImGui/Godot affordances.
    let pad = Px(pad_sm.0.max(6.0));
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    for &(_zone, r) in inner_rects.iter() {
        min_x = min_x.min(r.origin.x.0);
        min_y = min_y.min(r.origin.y.0);
        max_x = max_x.max(r.origin.x.0 + r.size.width.0);
        max_y = max_y.max(r.origin.y.0 + r.size.height.0);
    }
    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        let plate = Rect::new(
            Point::new(Px(min_x - pad.0), Px(min_y - pad.0)),
            Size::new(
                Px((max_x - min_x + pad.0 * 2.0).max(0.0)),
                Px((max_y - min_y + pad.0 * 2.0).max(0.0)),
            ),
        );
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(order.0 - 2),
            rect: plate,
            background: Color {
                a: 0.70,
                ..surface_bg
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                a: 0.70,
                ..inactive_border_base
            },
            corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
        });
    }

    let mut paint_set = |hint_rects: &[(DropZone, Rect); 5],
                         active_set: bool,
                         skip_center: bool,
                         inactive_alpha: f32| {
        for &(zone, hint_rect) in hint_rects.iter() {
            if skip_center && zone == DropZone::Center {
                continue;
            }
            let is_active = active_set && zone == target.zone;
            let bg = if is_active {
                active_bg
            } else {
                Color {
                    a: inactive_bg.a * inactive_alpha,
                    ..inactive_bg
                }
            };
            let stroke = if is_active {
                active_border
            } else {
                Color {
                    a: inactive_border.a * inactive_alpha,
                    ..inactive_border
                }
            };

            scene.push(SceneOp::Quad {
                order,
                rect: hint_rect,
                background: bg,
                border,
                border_color: stroke,
                corner_radii,
            });
            paint_drop_hint_icon(theme, zone, hint_rect, is_active, scene, order.0 + 1);
        }
    };

    // Match ImGui's mental model: inner and outer hint sets can coexist; the active set is
    // determined by which family of drop rects was hit-tested.
    paint_set(&inner_rects, !target.outer, false, 1.0);
    if let Some(outer_rects) = outer_rects.as_ref() {
        paint_set(outer_rects, target.outer, true, 0.80);
    }
}

fn paint_drop_hint_icon(
    theme: fret_ui::ThemeSnapshot,
    zone: DropZone,
    hint_rect: Rect,
    is_active: bool,
    scene: &mut Scene,
    order: u32,
) {
    fn inset(rect: Rect, inset: Px) -> Rect {
        let w = (rect.size.width.0 - inset.0 * 2.0).max(0.0);
        let h = (rect.size.height.0 - inset.0 * 2.0).max(0.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + inset.0), Px(rect.origin.y.0 + inset.0)),
            Size::new(Px(w), Px(h)),
        )
    }

    let min_dim = hint_rect.size.width.0.min(hint_rect.size.height.0);
    let pad = Px((min_dim * 0.18).clamp(6.0, 10.0));
    let frame = inset(hint_rect, pad);
    let inner = inset(frame, Px((min_dim * 0.08).clamp(2.0, 4.0)));

    let fg = theme.color_required("foreground");
    let stroke = Color {
        a: if is_active { 0.92 } else { 0.80 },
        ..fg
    };
    let base = Color {
        a: if is_active { 0.16 } else { 0.12 },
        ..fg
    };
    let fill = Color {
        a: if is_active { 0.90 } else { 0.72 },
        ..fg
    };

    let frame_radius = Px(theme.metric_required("metric.radius.sm").0.clamp(2.0, 4.0));
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order),
        rect: frame,
        background: Color::TRANSPARENT,
        border: Edges::all(Px(2.0)),
        border_color: stroke,
        corner_radii: fret_core::Corners::all(frame_radius),
    });

    // Base fill so the highlighted region reads as "target placement" (ImGui-like).
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order + 1),
        rect: inner,
        background: base,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let split_ratio = 0.42_f32;
    let tab_ratio = 0.24_f32;
    let line_thickness = Px((min_dim * 0.04).clamp(1.5, 2.5));

    match zone {
        DropZone::Center => {
            let tab_h = Px((inner.size.height.0 * tab_ratio).max(0.0));
            let tab = Rect::new(inner.origin, Size::new(inner.size.width, tab_h));
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: tab,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Left | DropZone::Right => {
            let w = Px((inner.size.width.0 * split_ratio).max(0.0));
            let (highlight, line_x) = if zone == DropZone::Left {
                (
                    Rect::new(inner.origin, Size::new(w, inner.size.height)),
                    Px(inner.origin.x.0 + w.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(
                            Px(inner.origin.x.0 + inner.size.width.0 - w.0),
                            inner.origin.y,
                        ),
                        Size::new(w, inner.size.height),
                    ),
                    Px(inner.origin.x.0 + inner.size.width.0 - w.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(Px(line_x.0 - line_thickness.0 * 0.5), inner.origin.y),
                Size::new(line_thickness, inner.size.height),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: stroke,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Top | DropZone::Bottom => {
            let h = Px((inner.size.height.0 * split_ratio).max(0.0));
            let (highlight, line_y) = if zone == DropZone::Top {
                (
                    Rect::new(inner.origin, Size::new(inner.size.width, h)),
                    Px(inner.origin.y.0 + h.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(
                            inner.origin.x,
                            Px(inner.origin.y.0 + inner.size.height.0 - h.0),
                        ),
                        Size::new(inner.size.width, h),
                    ),
                    Px(inner.origin.y.0 + inner.size.height.0 - h.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(inner.origin.x, Px(line_y.0 - line_thickness.0 * 0.5)),
                Size::new(inner.size.width, line_thickness),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: stroke,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }
}
