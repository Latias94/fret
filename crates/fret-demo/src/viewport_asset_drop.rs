use fret_app::DragKind;
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, PanelKey, Point, Px, Rect, SceneOp, Size,
    ViewportMapping,
};
use fret_editor::{ProjectEntryKind, ProjectService};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

use crate::asset_drop::{AssetDropRequest, AssetDropService, AssetDropTarget};
use crate::project_panel::ProjectDragPayload;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ViewportAssetDropHover {
    draw_rect: Rect,
    uv: (f32, f32),
}

pub struct ViewportAssetDropPanel {
    panel: PanelKey,
    last_bounds: Rect,
    hover: Option<ViewportAssetDropHover>,
}

impl ViewportAssetDropPanel {
    pub fn new(panel: PanelKey) -> Self {
        Self {
            panel,
            last_bounds: Rect::default(),
            hover: None,
        }
    }

    fn hover_at(&self, app: &fret_app::App, position: Point) -> Option<ViewportAssetDropHover> {
        // Keep the MVP focused on the Scene viewport; Game can opt-in later.
        if self.panel.kind.0.as_str() == "core.game" {
            return None;
        }

        let drag = app.drag()?;
        if drag.kind != DragKind::Custom || !drag.dragging {
            return None;
        }
        let payload = drag.payload::<ProjectDragPayload>()?;

        let project = app.global::<ProjectService>()?;
        let id = project.id_for_guid(payload.guid)?;
        if project.kind_for_id(id) != Some(ProjectEntryKind::File) {
            return None;
        }

        let dock = app.global::<fret_ui::DockManager>()?;
        let panel = dock.panels.get(&self.panel)?;
        let vp = panel.viewport?;

        let mapping = ViewportMapping {
            content_rect: self.last_bounds,
            target_px_size: vp.target_px_size,
            fit: vp.fit,
        };
        let draw_rect = mapping.map().draw_rect;
        if !draw_rect.contains(position) {
            return None;
        }
        let uv = mapping.window_point_to_uv(position)?;
        Some(ViewportAssetDropHover { draw_rect, uv })
    }

    fn paint_hover(&self, cx: &mut PaintCx<'_>, hover: ViewportAssetDropHover) {
        let theme = cx.theme().snapshot();
        let stroke = Color {
            a: 0.9,
            ..theme.colors.accent
        };
        let t = Px(2.0);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20),
            rect: hover.draw_rect,
            background: Color {
                a: 0.10,
                ..theme.colors.accent
            },
            border: Edges::all(t),
            border_color: stroke,
            corner_radii: Corners::all(Px(0.0)),
        });

        let r = Px(6.0);
        let x = hover.draw_rect.origin.x.0 + hover.draw_rect.size.width.0 * hover.uv.0;
        let y = hover.draw_rect.origin.y.0 + hover.draw_rect.size.height.0 * hover.uv.1;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(21),
            rect: Rect::new(
                Point::new(Px(x - r.0 * 0.5), Px(y - r.0 * 0.5)),
                Size::new(r, r),
            ),
            background: stroke,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(2.0)),
        });
    }
}

impl Widget for ViewportAssetDropPanel {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        let Event::InternalDrag(drag) = event else {
            return;
        };

        match drag.kind {
            fret_core::InternalDragKind::Enter | fret_core::InternalDragKind::Over => {
                let next = self.hover_at(cx.app, drag.position);
                if self.hover != next {
                    self.hover = next;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            fret_core::InternalDragKind::Leave | fret_core::InternalDragKind::Cancel => {
                if self.hover.take().is_some() {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            fret_core::InternalDragKind::Drop => {
                let Some(window) = cx.window else {
                    self.hover = None;
                    return;
                };

                let Some(guid) = cx
                    .app
                    .drag()
                    .and_then(|d| d.payload::<ProjectDragPayload>())
                    .map(|p| p.guid)
                else {
                    self.hover = None;
                    return;
                };
                let Some(hover) = self.hover_at(cx.app, drag.position) else {
                    self.hover = None;
                    return;
                };

                let dock = cx.app.global::<fret_ui::DockManager>();
                let target = dock
                    .and_then(|d| d.panels.get(&self.panel))
                    .and_then(|p| p.viewport)
                    .map(|vp| vp.target);
                let Some(target) = target else {
                    self.hover = None;
                    return;
                };

                cx.app
                    .with_global_mut(AssetDropService::default, |s, _app| {
                        s.push(AssetDropRequest {
                            window,
                            guid,
                            target: AssetDropTarget::SceneViewport {
                                panel: self.panel.clone(),
                                target,
                                uv: hover.uv,
                            },
                        });
                    });

                self.hover = None;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        if let Some(hover) = self.hover {
            self.paint_hover(cx, hover);
        }
    }
}
