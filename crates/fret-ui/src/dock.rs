use fret_app::{CommandId, Effect, InputContext, Menu, MenuItem};
use fret_core::{
    Color, DockGraph, DockNode, DockNodeId, DockOp, DropZone, Edges, PanelKey, RenderTargetId,
    Scene, SceneOp, ViewportFit, ViewportInputEvent, ViewportInputKind, ViewportMapping,
    geometry::{Point, Px, Rect, Size},
};
use std::{collections::HashMap, sync::Arc};

use crate::{
    widget::{EventCx, LayoutCx, PaintCx, Widget},
    widgets::{ContextMenuRequest, ContextMenuService},
};

pub struct DockPanel {
    pub title: String,
    pub color: Color,
    pub viewport: Option<ViewportPanel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportPanel {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
}

#[derive(Debug, Clone)]
struct DockPanelDragPayload {
    panel: PanelKey,
}

#[derive(Debug, Clone)]
enum DockDropTarget {
    Dock(HoverTarget),
    Float { window: fret_core::AppWindowId },
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportHover {
    window: fret_core::AppWindowId,
    panel: PanelKey,
    position: Point,
}

pub struct DockManager {
    pub graph: DockGraph,
    pub panels: HashMap<PanelKey, DockPanel>,
    hover: Option<DockDropTarget>,
    viewport_hover: Option<ViewportHover>,
    viewport_context_menu: Option<ViewportInputEvent>,
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            graph: DockGraph::new(),
            panels: HashMap::new(),
            hover: None,
            viewport_hover: None,
            viewport_context_menu: None,
        }
    }
}

impl DockManager {
    pub fn insert_panel(&mut self, key: PanelKey, panel: DockPanel) {
        self.panels.insert(key, panel);
    }

    pub fn ensure_panel(&mut self, key: &PanelKey, make: impl FnOnce() -> DockPanel) {
        self.panels.entry(key.clone()).or_insert_with(make);
    }

    pub fn panel(&self, key: &PanelKey) -> Option<&DockPanel> {
        self.panels.get(key)
    }
}

#[derive(Debug, Clone, Copy)]
struct DividerDragState {
    split: DockNodeId,
    axis: fret_core::Axis,
    bounds: Rect,
    fraction: f32,
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
        let mut invalidate_paint = false;
        let mut open_viewport_menu: Option<(Point, ViewportInputEvent)> = None;

        #[derive(Clone)]
        struct DockDragSnapshot {
            source_window: fret_core::AppWindowId,
            start: Point,
            dragging: bool,
            panel: PanelKey,
        }

        let allow_viewport_hover = cx.app.drag().map_or(true, |d| !d.dragging);
        let dock_drag = cx.app.drag().and_then(|d| {
            d.payload::<DockPanelDragPayload>()
                .map(|p| DockDragSnapshot {
                    source_window: d.source_window,
                    start: d.start,
                    dragging: d.dragging,
                    panel: p.panel.clone(),
                })
        });

        let mut begin_drag: Option<(Point, PanelKey)> = None;
        let mut update_drag: Option<(Point, bool)> = None;
        let mut end_dock_drag = false;

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
                        position,
                        button,
                        modifiers,
                    } => {
                        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                        let mut handled = false;
                        if *button == fret_core::MouseButton::Left {
                            if let Some(handle) =
                                hit_test_split_handle(&dock.graph, &layout, *position)
                            {
                                self.divider_drag = Some(handle);
                                cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                                return;
                            }
                            if let Some((tabs_node, tab_index, panel_key)) =
                                hit_test_tab(&dock.graph, &layout, *position)
                            {
                                pending_effects.push(Effect::Dock(DockOp::SetActiveTab {
                                    tabs: tabs_node,
                                    active: tab_index,
                                }));
                                begin_drag = Some((*position, panel_key));
                                dock.hover = None;
                                invalidate_paint = true;
                                handled = true;
                            }
                        }

                        if !handled {
                            if let Some(hit) = hit_test_active_viewport_panel(
                                &dock.graph,
                                &dock.panels,
                                &layout,
                                *position,
                            ) {
                                if *button == fret_core::MouseButton::Right {
                                    if let Some(e) = viewport_input_from_hit(
                                        self.window,
                                        hit,
                                        *position,
                                        ViewportInputKind::PointerDown {
                                            button: *button,
                                            modifiers: *modifiers,
                                        },
                                    ) {
                                        dock.viewport_context_menu = Some(e);
                                        open_viewport_menu = Some((*position, e));
                                        invalidate_paint = true;
                                    }
                                } else if let Some(e) = viewport_input_from_hit(
                                    self.window,
                                    hit,
                                    *position,
                                    ViewportInputKind::PointerDown {
                                        button: *button,
                                        modifiers: *modifiers,
                                    },
                                ) {
                                    pending_effects.push(Effect::ViewportInput(e));
                                    pending_redraws.push(self.window);
                                }
                            }
                        }
                    }
                    fret_core::PointerEvent::Move {
                        position,
                        buttons,
                        modifiers,
                    } => {
                        if let Some(mut divider) = self.divider_drag {
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
                                    divider.fraction = f0;
                                    self.divider_drag = Some(divider);
                                    cx.invalidate(cx.node, crate::widget::Invalidation::Layout);
                                    cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                                }
                            }
                            return;
                        }

                        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                        if allow_viewport_hover && dock_bounds.contains(*position) {
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            let hit = hit_test_active_viewport_panel(
                                &dock.graph,
                                &dock.panels,
                                &layout,
                                *position,
                            );

                            let next_hover = hit.as_ref().map(|hit| ViewportHover {
                                window: self.window,
                                panel: hit.panel.clone(),
                                position: *position,
                            });
                            if dock.viewport_hover != next_hover {
                                dock.viewport_hover = next_hover;
                                pending_redraws.push(self.window);
                            } else if next_hover.is_some() {
                                dock.viewport_hover = next_hover;
                                pending_redraws.push(self.window);
                            }

                            if let Some(hit) = hit {
                                if let Some(e) = viewport_input_from_hit(
                                    self.window,
                                    hit,
                                    *position,
                                    ViewportInputKind::PointerMove {
                                        buttons: *buttons,
                                        modifiers: *modifiers,
                                    },
                                ) {
                                    pending_effects.push(Effect::ViewportInput(e));
                                }
                            }
                        } else if dock
                            .viewport_hover
                            .as_ref()
                            .is_some_and(|h| h.window == self.window)
                        {
                            dock.viewport_hover = None;
                            pending_redraws.push(self.window);
                        }

                        if let Some(drag) = dock_drag.as_ref() {
                            let mut dragging = drag.dragging;
                            if drag.source_window == self.window {
                                let dx = position.x.0 - drag.start.x.0;
                                let dy = position.y.0 - drag.start.y.0;
                                let dist2 = dx * dx + dy * dy;
                                if !dragging && dist2 > 16.0 {
                                    dragging = true;
                                }
                            } else if !dragging {
                                dragging = true;
                            }

                            update_drag = Some((*position, dragging));

                            if dragging {
                                let (chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                                if chrome.contains(*position) {
                                    dock.hover = Some(DockDropTarget::Float {
                                        window: self.window,
                                    });
                                } else if dock_bounds.contains(*position) {
                                    let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                                    dock.hover =
                                        hit_test_drop_target(&dock.graph, &layout, *position)
                                            .map(DockDropTarget::Dock);
                                } else {
                                    dock.hover = None;
                                }
                                pending_redraws.push(self.window);
                            } else {
                                dock.hover = None;
                            }
                        } else {
                            dock.hover = None;
                        }
                    }
                    fret_core::PointerEvent::Wheel {
                        position,
                        delta,
                        modifiers,
                    } => {
                        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                        if !dock_bounds.contains(*position) {
                            return;
                        }
                        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                        if let Some(hit) = hit_test_active_viewport_panel(
                            &dock.graph,
                            &dock.panels,
                            &layout,
                            *position,
                        ) {
                            if let Some(e) = viewport_input_from_hit(
                                self.window,
                                hit,
                                *position,
                                ViewportInputKind::Wheel {
                                    delta: *delta,
                                    modifiers: *modifiers,
                                },
                            ) {
                                pending_effects.push(Effect::ViewportInput(e));
                                pending_redraws.push(self.window);
                            }
                        }
                    }
                    fret_core::PointerEvent::Up {
                        position,
                        button,
                        modifiers,
                    } => {
                        if *button == fret_core::MouseButton::Left {
                            if let Some(divider) = self.divider_drag.take() {
                                pending_effects.push(Effect::Dock(DockOp::SetSplitFractionTwo {
                                    split: divider.split,
                                    first_fraction: divider.fraction,
                                }));
                            }
                        }

                        if *button == fret_core::MouseButton::Left && dock_drag.is_some() {
                            let drag = dock_drag.unwrap();

                            if drag.dragging {
                                match dock.hover.clone() {
                                    Some(DockDropTarget::Dock(target)) => {
                                        pending_effects.push(Effect::Dock(DockOp::MovePanel {
                                            source_window: drag.source_window,
                                            panel: drag.panel.clone(),
                                            target_window: self.window,
                                            target_tabs: target.tabs,
                                            zone: target.zone,
                                            insert_index: target.insert_index,
                                        }));
                                    }
                                    Some(DockDropTarget::Float { .. }) => {
                                        pending_effects.push(Effect::Dock(
                                            DockOp::RequestFloatPanelToNewWindow {
                                                source_window: drag.source_window,
                                                panel: drag.panel.clone(),
                                                anchor: Some(fret_core::WindowAnchor {
                                                    window: self.window,
                                                    position: *position,
                                                }),
                                            },
                                        ));
                                    }
                                    None => {
                                        let (chrome, _dock_bounds) =
                                            dock_space_regions(self.last_bounds);
                                        if chrome.contains(*position) {
                                            pending_effects.push(Effect::Dock(
                                                DockOp::RequestFloatPanelToNewWindow {
                                                    source_window: drag.source_window,
                                                    panel: drag.panel.clone(),
                                                    anchor: Some(fret_core::WindowAnchor {
                                                        window: self.window,
                                                        position: *position,
                                                    }),
                                                },
                                            ));
                                        }
                                    }
                                }
                            }

                            dock.hover = None;
                            end_dock_drag = true;
                            invalidate_paint = true;
                        } else {
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            if dock_bounds.contains(*position) {
                                let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                                if let Some(hit) = hit_test_active_viewport_panel(
                                    &dock.graph,
                                    &dock.panels,
                                    &layout,
                                    *position,
                                ) {
                                    if let Some(e) = viewport_input_from_hit(
                                        self.window,
                                        hit,
                                        *position,
                                        ViewportInputKind::PointerUp {
                                            button: *button,
                                            modifiers: *modifiers,
                                        },
                                    ) {
                                        pending_effects.push(Effect::ViewportInput(e));
                                        pending_redraws.push(self.window);
                                    }
                                }
                            }
                            dock.hover = None;
                            invalidate_paint = true;
                        }
                    }
                },
                _ => {}
            }
        }

        if let Some((start, panel)) = begin_drag {
            cx.app
                .begin_drag(self.window, start, DockPanelDragPayload { panel });
        }

        if let Some((position, dragging)) = update_drag {
            if let Some(drag) = cx.app.drag_mut() {
                if drag.payload::<DockPanelDragPayload>().is_some() {
                    drag.position = position;
                    drag.dragging = dragging;
                }
            }
        }

        if end_dock_drag {
            if cx
                .app
                .drag()
                .and_then(|d| d.payload::<DockPanelDragPayload>())
                .is_some()
            {
                cx.app.cancel_drag();
            }
        }

        if let Some((position, _viewport_event)) = open_viewport_menu {
            let Some(window) = cx.window else {
                return;
            };

            cx.request_focus(cx.node);

            let inv_ctx = InputContext {
                platform: cx.input_ctx.platform,
                ui_has_modal: cx.input_ctx.ui_has_modal,
                focus_is_text_input: false,
            };

            let menu = Menu {
                title: Arc::from("Viewport"),
                items: vec![
                    MenuItem::Command {
                        command: CommandId::from("viewport.copy_uv"),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::from("viewport.copy_target_px"),
                        when: None,
                    },
                ],
            };

            cx.app
                .with_global_mut(ContextMenuService::default, |service, _app| {
                    service.set_request(
                        window,
                        ContextMenuRequest {
                            position,
                            menu,
                            input_ctx: inv_ctx,
                        },
                    );
                });
            cx.dispatch_command(CommandId::from("context_menu.open"));
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        if invalidate_paint {
            cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
        }

        for window in pending_redraws {
            cx.app.request_redraw(window);
        }
        for effect in pending_effects {
            cx.app.push_effect(effect);
        }
    }

    fn command(&mut self, cx: &mut crate::widget::CommandCx<'_>, command: &CommandId) -> bool {
        match command.as_str() {
            "viewport.copy_uv" => {
                let Some(dock) = cx.app.global::<DockManager>() else {
                    return false;
                };
                let Some(e) = dock.viewport_context_menu else {
                    return false;
                };
                cx.app.push_effect(Effect::ClipboardSetText {
                    text: format!("{:.6}, {:.6}", e.uv.0, e.uv.1),
                });
                cx.stop_propagation();
                true
            }
            "viewport.copy_target_px" => {
                let Some(dock) = cx.app.global::<DockManager>() else {
                    return false;
                };
                let Some(e) = dock.viewport_context_menu else {
                    return false;
                };
                cx.app.push_effect(Effect::ClipboardSetText {
                    text: format!("{}, {}", e.target_px.0, e.target_px.1),
                });
                cx.stop_propagation();
                true
            }
            _ => false,
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
        paint_dock(dock, self.window, &layout, cx.scene);
        paint_split_handles(&dock.graph, &layout, cx.scene);

        paint_drop_overlay(dock.hover.clone(), self.window, chrome, &layout, cx.scene);
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
    window: fret_core::AppWindowId,
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
        for (i, panel) in tabs.iter().enumerate() {
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

        let active_panel = tabs.get(*active);
        if let Some(panel) = active_panel.and_then(|p| dock.panel(p)) {
            if let Some(vp) = panel.viewport {
                let mapping = ViewportMapping {
                    content_rect: content,
                    target_px_size: vp.target_px_size,
                    fit: vp.fit,
                };
                let draw_rect = mapping.map().draw_rect;

                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                scene.push(SceneOp::PushClipRect { rect: content });
                scene.push(SceneOp::ViewportSurface {
                    order: fret_core::DrawOrder(4),
                    rect: draw_rect,
                    target: vp.target,
                    opacity: 1.0,
                });
                if let Some(h) = dock.viewport_hover.as_ref() {
                    if h.window == window && active_panel.is_some_and(|p| p == &h.panel) {
                        paint_viewport_crosshair(draw_rect, h.position, scene);
                    }
                }
                scene.push(SceneOp::PopClip);
            } else {
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
}

fn paint_viewport_crosshair(content: Rect, position: Point, scene: &mut Scene) {
    if !content.contains(position) {
        return;
    }

    let thickness = Px(1.5);
    let len = Px(12.0);
    let x = position.x;
    let y = position.y;

    let h = Rect {
        origin: Point::new(Px(x.0 - len.0), Px(y.0 - thickness.0 * 0.5)),
        size: Size::new(Px(len.0 * 2.0), thickness),
    };
    let v = Rect {
        origin: Point::new(Px(x.0 - thickness.0 * 0.5), Px(y.0 - len.0)),
        size: Size::new(thickness, Px(len.0 * 2.0)),
    };

    let color = Color {
        r: 0.95,
        g: 0.95,
        b: 0.97,
        a: 0.65,
    };

    for rect in [h, v] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(5),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportHit {
    panel: PanelKey,
    viewport: ViewportPanel,
    content: Rect,
    draw_rect: Rect,
}

fn viewport_input_from_hit(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> Option<ViewportInputEvent> {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv(position)?;
    let target_px = mapping.window_point_to_target_px(position)?;
    Some(ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    })
}

fn hit_test_active_viewport_panel(
    graph: &DockGraph,
    panels: &HashMap<PanelKey, DockPanel>,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<ViewportHit> {
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let Some(panel_key) = tabs.get(*active).cloned() else {
            continue;
        };
        let Some(panel) = panels.get(&panel_key) else {
            continue;
        };
        let Some(viewport) = panel.viewport else {
            continue;
        };

        let (_tab_bar, content) = split_tab_bar(rect);
        let mapping = ViewportMapping {
            content_rect: content,
            target_px_size: viewport.target_px_size,
            fit: viewport.fit,
        };
        let draw_rect = mapping.map().draw_rect;
        if draw_rect.contains(position) {
            return Some(ViewportHit {
                panel: panel_key,
                viewport,
                content,
                draw_rect,
            });
        }
    }
    None
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
) -> Option<(DockNodeId, usize, PanelKey)> {
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
        let panel = tabs.get(idx)?.clone();
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
        let Some(DockNode::Split {
            axis,
            children,
            fractions,
        }) = graph.node(node)
        else {
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
            let total = fractions.iter().take(2).sum::<f32>();
            let total = if total <= 0.0 { 1.0 } else { total };
            let f0 = fractions.get(0).copied().unwrap_or(0.5) / total;
            return Some(DividerDragState {
                split: node,
                axis: *axis,
                bounds,
                fraction: f0,
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
