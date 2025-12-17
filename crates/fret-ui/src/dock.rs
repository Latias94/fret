use fret_app::{CreateDockFloatingWindow, Effect, WindowEffect};
use fret_core::{
    geometry::{Point, Px, Rect, Size},
    Color, DockGraph, DockNode, DockNodeId, DropZone, Edges, PanelId, Scene, SceneOp,
};
use slotmap::SlotMap;

use crate::widget::{EventCx, LayoutCx, PaintCx, Widget};

pub struct DockPanel {
    pub title: String,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
struct DockDrag {
    source_window: fret_core::AppWindowId,
    panel: PanelId,
    pointer_start: Point,
    dragging: bool,
}

#[derive(Debug, Clone, Copy)]
enum DockDropTarget {
    Dock(HoverTarget),
    Float { window: fret_core::AppWindowId },
}

pub struct DockManager {
    pub graph: DockGraph,
    pub panels: SlotMap<PanelId, DockPanel>,
    drag: Option<DockDrag>,
    hover: Option<DockDropTarget>,
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            graph: DockGraph::new(),
            panels: SlotMap::with_key(),
            drag: None,
            hover: None,
        }
    }
}

impl DockManager {
    pub fn create_panel(&mut self, panel: DockPanel) -> PanelId {
        self.panels.insert(panel)
    }

    pub fn panel(&self, id: PanelId) -> Option<&DockPanel> {
        self.panels.get(id)
    }
}

#[derive(Debug, Clone, Copy)]
struct DividerDragState {
    split: DockNodeId,
    axis: fret_core::Axis,
    bounds: Rect,
}

#[derive(Debug, Clone, Copy)]
struct HoverTarget {
    tabs: DockNodeId,
    zone: DropZone,
    insert_index: Option<usize>,
}

pub struct DockSpace {
    pub window: fret_core::AppWindowId,
    last_bounds: Rect,
    divider_drag: Option<DividerDragState>,
}

impl DockSpace {
    pub fn new(window: fret_core::AppWindowId) -> Self {
        Self {
            window,
            last_bounds: Rect::default(),
            divider_drag: None,
        }
    }
}

impl Widget for DockSpace {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &fret_core::Event) {
        let mut pending_effects: Vec<Effect> = Vec::new();
        let mut pending_redraws: Vec<fret_core::AppWindowId> = Vec::new();

        {
            let Some(dock) = cx.app.global_mut::<DockManager>() else {
                return;
            };
            let Some(root) = dock.graph.window_root(self.window) else {
                return;
            };

            match event {
                fret_core::Event::Pointer(p) => match p {
                    fret_core::PointerEvent::Down {
                        position, button, ..
                    } => {
                        if *button != fret_core::MouseButton::Left {
                            return;
                        }
                        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                        if let Some(handle) = hit_test_split_handle(&dock.graph, &layout, *position) {
                            self.divider_drag = Some(handle);
                            cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                            return;
                        }
                        if let Some(hit) = hit_test_tab(&dock.graph, &layout, *position) {
                            let (tabs_node, tab_index, panel_id) = hit;
                            dock.graph.set_active_tab(tabs_node, tab_index);
                            dock.drag = Some(DockDrag {
                                source_window: self.window,
                                panel: panel_id,
                                pointer_start: *position,
                                dragging: false,
                            });
                            dock.hover = None;
                            cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                        }
                    }
                    fret_core::PointerEvent::Move { position } => {
                        if let Some(divider) = self.divider_drag {
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            if let Some((left, right)) =
                                split_children_two(&dock.graph, divider.split).and_then(|(a, b)| {
                                    Some((layout.get(&a).copied()?, layout.get(&b).copied()?))
                                })
                            {
                                if let Some(f0) = compute_split_fraction(
                                    divider.axis,
                                    divider.bounds,
                                    left,
                                    right,
                                    *position,
                                ) {
                                    dock.graph.update_split_two(divider.split, f0);
                                    cx.invalidate(cx.node, crate::widget::Invalidation::Layout);
                                    cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                                }
                            }
                            return;
                        }

                        let Some(mut drag) = dock.drag else {
                            dock.hover = None;
                            return;
                        };

                        if drag.source_window == self.window {
                            let dx = position.x.0 - drag.pointer_start.x.0;
                            let dy = position.y.0 - drag.pointer_start.y.0;
                            let dist2 = dx * dx + dy * dy;
                            if !drag.dragging && dist2 > 16.0 {
                                drag.dragging = true;
                            }
                        } else if !drag.dragging {
                            drag.dragging = true;
                        }
                        dock.drag = Some(drag);

                        if !drag.dragging {
                            dock.hover = None;
                            return;
                        }

                        let (chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                        if chrome.contains(*position) {
                            dock.hover = Some(DockDropTarget::Float {
                                window: self.window,
                            });
                        } else if dock_bounds.contains(*position) {
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            dock.hover = hit_test_drop_target(&dock.graph, &layout, *position)
                                .map(DockDropTarget::Dock);
                        } else {
                            dock.hover = None;
                        }
                        cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                    }
                    fret_core::PointerEvent::Up {
                        position, button, ..
                    } => {
                        if *button != fret_core::MouseButton::Left {
                            return;
                        }
                        self.divider_drag = None;

                        let Some(drag) = dock.drag.take() else {
                            dock.hover = None;
                            cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                            return;
                        };
                        if drag.dragging {
                            match dock.hover {
                                Some(DockDropTarget::Dock(target)) => {
                                    dock.graph.move_panel_between_windows(
                                        drag.source_window,
                                        drag.panel,
                                        self.window,
                                        target.tabs,
                                        target.zone,
                                        target.insert_index,
                                    );

                                    pending_redraws.push(drag.source_window);
                                    pending_redraws.push(self.window);

                                    if dock
                                        .graph
                                        .collect_panels_in_window(drag.source_window)
                                        .is_empty()
                                    {
                                        pending_effects.push(Effect::Window(WindowEffect::Close(
                                            drag.source_window,
                                        )));
                                    }
                                }
                                Some(DockDropTarget::Float { .. }) => {
                                    pending_effects.push(Effect::Window(
                                        WindowEffect::CreateDockFloating(CreateDockFloatingWindow {
                                            source_window: drag.source_window,
                                            panel: drag.panel,
                                            anchor_window: self.window,
                                            anchor_position: *position,
                                        }),
                                    ));
                                }
                                None => {
                                    let (chrome, _dock_bounds) =
                                        dock_space_regions(self.last_bounds);
                                    if chrome.contains(*position) {
                                        pending_effects.push(Effect::Window(
                                            WindowEffect::CreateDockFloating(
                                                CreateDockFloatingWindow {
                                                    source_window: drag.source_window,
                                                    panel: drag.panel,
                                                    anchor_window: self.window,
                                                    anchor_position: *position,
                                                },
                                            ),
                                        ));
                                    }
                                }
                            }
                        }

                        dock.hover = None;
                        cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        for window in pending_redraws {
            cx.app.request_redraw(window);
        }
        for effect in pending_effects {
            cx.app.push_effect(effect);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.last_bounds = cx.bounds;
        let Some(dock) = cx.app.global::<DockManager>() else {
            return;
        };
        let Some(root) = dock.graph.window_root(self.window) else {
            return;
        };

        let (chrome, dock_bounds) = dock_space_regions(cx.bounds);
        let layout = compute_layout_map(&dock.graph, root, dock_bounds);

        paint_chrome(chrome, cx.scene);
        paint_dock(dock, &layout, cx.scene);
        paint_split_handles(&dock.graph, &layout, cx.scene);

        paint_drop_overlay(dock.hover, self.window, chrome, &layout, cx.scene);
    }
}

fn compute_layout_map(
    graph: &DockGraph,
    root: DockNodeId,
    bounds: Rect,
) -> std::collections::HashMap<DockNodeId, Rect> {
    let mut layout = std::collections::HashMap::new();
    graph.compute_layout(root, bounds, &mut layout);
    layout
}

fn paint_dock(
    dock: &DockManager,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let graph = &dock.graph;
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (tab_bar, content) = split_tab_bar(rect);

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect,
            background: Color {
                r: 0.12,
                g: 0.13,
                b: 0.14,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(1),
            rect: tab_bar,
            background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.11,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let tab_w = Px(120.0);
        for (i, panel) in tabs.iter().copied().enumerate() {
            let tab_rect = Rect {
                origin: Point::new(
                    Px(tab_bar.origin.x.0 + tab_w.0 * i as f32),
                    tab_bar.origin.y,
                ),
                size: Size::new(tab_w, tab_bar.size.height),
            };

            let is_active = i == *active;
            let bg = if is_active {
                Color {
                    r: 0.18,
                    g: 0.18,
                    b: 0.20,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.14,
                    g: 0.14,
                    b: 0.15,
                    a: 1.0,
                }
            };

            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(2),
                rect: tab_rect,
                background: bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });

            let _ = dock.panel(panel);
        }

        let active_panel = tabs.get(*active).copied();
        if let Some(panel) = active_panel.and_then(|p| dock.panel(p)) {
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(3),
                rect: content,
                background: panel.color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(6.0)),
            });
        }
    }
}

fn split_tab_bar(rect: Rect) -> (Rect, Rect) {
    let tab_h = Px(28.0);
    let tab_bar = Rect {
        origin: rect.origin,
        size: Size::new(rect.size.width, Px(tab_h.0.min(rect.size.height.0))),
    };
    let content = Rect {
        origin: Point::new(rect.origin.x, Px(rect.origin.y.0 + tab_bar.size.height.0)),
        size: Size::new(
            rect.size.width,
            Px((rect.size.height.0 - tab_bar.size.height.0).max(0.0)),
        ),
    };
    (tab_bar, content)
}

fn drop_zone_rect(rect: Rect, zone: DropZone) -> Rect {
    if zone == DropZone::Center {
        return rect;
    }
    let thickness = 0.25 * rect.size.width.0.min(rect.size.height.0);
    match zone {
        DropZone::Left => Rect {
            origin: rect.origin,
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Right => Rect {
            origin: Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 - thickness),
                rect.origin.y,
            ),
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Top => Rect {
            origin: rect.origin,
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Bottom => Rect {
            origin: Point::new(
                rect.origin.x,
                Px(rect.origin.y.0 + rect.size.height.0 - thickness),
            ),
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Center => rect,
    }
}

fn float_zone(bounds: Rect) -> Rect {
    let size = Px(34.0);
    Rect {
        origin: Point::new(Px(bounds.origin.x.0 + 8.0), Px(bounds.origin.y.0 + 8.0)),
        size: Size::new(size, size),
    }
}

fn hit_test_tab(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<(DockNodeId, usize, PanelId)> {
    let tab_w = Px(120.0);
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let rel_x = position.x.0 - tab_bar.origin.x.0;
        let idx = (rel_x / tab_w.0).floor() as isize;
        if idx < 0 {
            continue;
        }
        let idx = idx as usize;
        let panel = *tabs.get(idx)?;
        return Some((node, idx, panel));
    }
    None
}

fn hit_test_drop_target(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<HoverTarget> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if !rect.contains(position) {
            continue;
        }

        let (tab_bar, _content) = split_tab_bar(rect);
        if tab_bar.contains(position) {
            let insert_index = compute_tab_insert_index(tab_bar, tabs.len(), position);
            return Some(HoverTarget {
                tabs: node,
                zone: DropZone::Center,
                insert_index: Some(insert_index),
            });
        }

        let thickness = 0.25 * rect.size.width.0.min(rect.size.height.0);
        let left = position.x.0 - rect.origin.x.0;
        let right = rect.origin.x.0 + rect.size.width.0 - position.x.0;
        let top = position.y.0 - rect.origin.y.0;
        let bottom = rect.origin.y.0 + rect.size.height.0 - position.y.0;

        let zone = if left < thickness {
            DropZone::Left
        } else if right < thickness {
            DropZone::Right
        } else if top < thickness {
            DropZone::Top
        } else if bottom < thickness {
            DropZone::Bottom
        } else {
            DropZone::Center
        };

        return Some(HoverTarget {
            tabs: node,
            zone,
            insert_index: None,
        });
    }
    None
}

fn compute_tab_insert_index(tab_bar: Rect, tab_count: usize, position: Point) -> usize {
    let tab_w = Px(120.0).0;
    let rel_x = position.x.0 - tab_bar.origin.x.0;
    let raw = (rel_x / tab_w) + 0.5;
    let idx = raw.floor() as isize;
    idx.clamp(0, tab_count as isize) as usize
}

fn split_children_two(graph: &DockGraph, split: DockNodeId) -> Option<(DockNodeId, DockNodeId)> {
    let Some(DockNode::Split { children, .. }) = graph.node(split) else {
        return None;
    };
    if children.len() != 2 {
        return None;
    }
    Some((children[0], children[1]))
}

fn hit_test_split_handle(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<DividerDragState> {
    let thickness = Px(6.0);

    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split { axis, children, .. }) = graph.node(node) else {
            continue;
        };
        if children.len() != 2 {
            continue;
        }
        if !bounds.contains(position) {
            continue;
        }

        let Some(left) = layout.get(&children[0]).copied() else {
            continue;
        };
        let Some(_right) = layout.get(&children[1]).copied() else {
            continue;
        };

        let handle = match axis {
            fret_core::Axis::Horizontal => {
                let x = left.origin.x.0 + left.size.width.0 - thickness.0 * 0.5;
                Rect {
                    origin: Point::new(Px(x), bounds.origin.y),
                    size: Size::new(thickness, bounds.size.height),
                }
            }
            fret_core::Axis::Vertical => {
                let y = left.origin.y.0 + left.size.height.0 - thickness.0 * 0.5;
                Rect {
                    origin: Point::new(bounds.origin.x, Px(y)),
                    size: Size::new(bounds.size.width, thickness),
                }
            }
        };

        if handle.contains(position) {
            return Some(DividerDragState {
                split: node,
                axis: *axis,
                bounds,
            });
        }
    }

    None
}

fn compute_split_fraction(
    axis: fret_core::Axis,
    bounds: Rect,
    _first: Rect,
    _second: Rect,
    position: Point,
) -> Option<f32> {
    let min_px = 120.0;
    match axis {
        fret_core::Axis::Horizontal => {
            let w = bounds.size.width.0;
            if w <= min_px * 2.0 {
                return None;
            }
            let x = (position.x.0 - bounds.origin.x.0).clamp(min_px, w - min_px);
            Some(x / w)
        }
        fret_core::Axis::Vertical => {
            let h = bounds.size.height.0;
            if h <= min_px * 2.0 {
                return None;
            }
            let y = (position.y.0 - bounds.origin.y.0).clamp(min_px, h - min_px);
            Some(y / h)
        }
    }
}

fn paint_split_handles(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let thickness = Px(4.0);
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split { axis, children, .. }) = graph.node(node) else {
            continue;
        };
        if children.len() != 2 {
            continue;
        }
        let Some(first) = layout.get(&children[0]).copied() else {
            continue;
        };

        let rect = match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(
                    Px(first.origin.x.0 + first.size.width.0 - thickness.0 * 0.5),
                    bounds.origin.y,
                ),
                size: Size::new(thickness, bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(
                    bounds.origin.x,
                    Px(first.origin.y.0 + first.size.height.0 - thickness.0 * 0.5),
                ),
                size: Size::new(bounds.size.width, thickness),
            },
        };

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(9_000),
            rect,
            background: Color {
                r: 0.06,
                g: 0.06,
                b: 0.07,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

fn paint_drop_overlay(
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    match target {
        DockDropTarget::Float { window: w } => {
            if w != window {
                return;
            }
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: float_zone(bounds),
                background: Color {
                    r: 0.20,
                    g: 0.55,
                    b: 1.00,
                    a: 0.45,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(8.0)),
            });
        }
        DockDropTarget::Dock(target) => {
            let Some(rect) = layout.get(&target.tabs).copied() else {
                return;
            };

            if target.zone == DropZone::Center {
                let (tab_bar, _content) = split_tab_bar(rect);
                if let Some(i) = target.insert_index {
                    let x = tab_bar.origin.x.0 + Px(120.0).0 * i as f32;
                    let marker = Rect {
                        origin: Point::new(Px(x - 2.0), tab_bar.origin.y),
                        size: Size::new(Px(4.0), tab_bar.size.height),
                    };
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(10_000),
                        rect: marker,
                        background: Color {
                            r: 0.20,
                            g: 0.55,
                            b: 1.00,
                            a: 0.65,
                        },
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(2.0)),
                    });
                    return;
                }
            }

            let overlay = drop_zone_rect(rect, target.zone);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: overlay,
                background: Color {
                    r: 0.20,
                    g: 0.55,
                    b: 1.00,
                    a: 0.22,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(6.0)),
            });
        }
    }
}

fn paint_chrome(bounds: Rect, scene: &mut Scene) {
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(50),
        rect: bounds,
        background: Color {
            r: 0.08,
            g: 0.08,
            b: 0.09,
            a: 1.0,
        },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let rect = float_zone(bounds);
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(8_000),
        rect,
        background: Color {
            r: 0.10,
            g: 0.10,
            b: 0.11,
            a: 1.0,
        },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(8.0)),
    });
}

fn dock_space_regions(bounds: Rect) -> (Rect, Rect) {
    let chrome_h = Px(44.0);
    let chrome = Rect {
        origin: bounds.origin,
        size: Size::new(bounds.size.width, Px(chrome_h.0.min(bounds.size.height.0))),
    };
    let dock = Rect {
        origin: Point::new(
            bounds.origin.x,
            Px(bounds.origin.y.0 + chrome.size.height.0),
        ),
        size: Size::new(
            bounds.size.width,
            Px((bounds.size.height.0 - chrome.size.height.0).max(0.0)),
        ),
    };
    (chrome, dock)
}
