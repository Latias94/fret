use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{
    AppWindowId, Event, MouseButton, PointerEvent, RenderTargetId, SemanticsRole, UiServices,
    ViewportFit, ViewportInputEvent, ViewportInputKind, ViewportMapping,
};
use fret_runtime::{Effect, Model};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{
    Invalidation, LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt, Widget,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plot3dViewport {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub opacity: f32,
}

impl Plot3dViewport {
    pub fn mapping(self, bounds: Rect) -> ViewportMapping {
        ViewportMapping {
            content_rect: bounds,
            target_px_size: self.target_px_size,
            fit: self.fit,
        }
    }

    pub fn draw_rect(self, bounds: Rect) -> Rect {
        self.mapping(bounds).map().draw_rect
    }
}

impl Default for Plot3dViewport {
    fn default() -> Self {
        Self {
            target: RenderTargetId::default(),
            target_px_size: (1, 1),
            fit: ViewportFit::Contain,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Plot3dModel {
    pub viewport: Plot3dViewport,
}

#[derive(Debug, Clone, Copy)]
pub struct Plot3dStyle {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Px,
}

impl Default for Plot3dStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            border_width: Px(1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ViewportCapture {
    window: AppWindowId,
    viewport: Plot3dViewport,
    bounds: Rect,
    button: MouseButton,
}

#[derive(Debug)]
pub struct Plot3dCanvas {
    model: Model<Plot3dModel>,
    style: Plot3dStyle,
    capture: Option<ViewportCapture>,
}

#[derive(Debug, Clone, Copy)]
struct ViewportInputArgs {
    window: AppWindowId,
    viewport: Plot3dViewport,
    bounds: Rect,
    position: Point,
    kind: ViewportInputKind,
    clamped: bool,
}

impl Plot3dCanvas {
    pub fn new(model: Model<Plot3dModel>) -> Self {
        Self {
            model,
            style: Plot3dStyle::default(),
            capture: None,
        }
    }

    pub fn style(mut self, style: Plot3dStyle) -> Self {
        self.style = style;
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        ui.create_node_retained(canvas)
    }

    fn push_viewport_input(&self, app: &mut impl UiHost, args: ViewportInputArgs) -> bool {
        let mapping = args.viewport.mapping(args.bounds);
        let (uv, target_px) = if args.clamped {
            (
                mapping.window_point_to_uv_clamped(args.position),
                mapping.window_point_to_target_px_clamped(args.position),
            )
        } else {
            let Some(uv) = mapping.window_point_to_uv(args.position) else {
                return false;
            };
            let Some(target_px) = mapping.window_point_to_target_px(args.position) else {
                return false;
            };
            (uv, target_px)
        };

        app.push_effect(Effect::ViewportInput(ViewportInputEvent {
            window: args.window,
            target: args.viewport.target,
            uv,
            target_px,
            kind: args.kind,
        }));
        true
    }
}

impl<H: UiHost> Widget<H> for Plot3dCanvas {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        let Ok(viewport) = self.model.read(cx.app, |_app, m| m.viewport) else {
            return;
        };

        let bounds = cx.bounds;
        let draw_rect = viewport.draw_rect(bounds);

        match event {
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                if !draw_rect.contains(*position) {
                    return;
                }

                let handled = self.push_viewport_input(
                    cx.app,
                    ViewportInputArgs {
                        window,
                        viewport,
                        bounds,
                        position: *position,
                        kind: ViewportInputKind::PointerDown {
                            button: *button,
                            modifiers: *modifiers,
                            click_count: *click_count,
                        },
                        clamped: false,
                    },
                );

                if handled {
                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    self.capture = Some(ViewportCapture {
                        window,
                        viewport,
                        bounds,
                        button: *button,
                    });
                    cx.stop_propagation();
                }
            }
            Event::Pointer(PointerEvent::Move {
                position,
                buttons,
                modifiers,
                ..
            }) => {
                if let Some(capture) = self.capture {
                    if capture.window != window {
                        return;
                    }
                    self.push_viewport_input(
                        cx.app,
                        ViewportInputArgs {
                            window,
                            viewport: capture.viewport,
                            bounds: capture.bounds,
                            position: *position,
                            kind: ViewportInputKind::PointerMove {
                                buttons: *buttons,
                                modifiers: *modifiers,
                            },
                            clamped: true,
                        },
                    );
                    cx.stop_propagation();
                    return;
                }

                if !draw_rect.contains(*position) {
                    return;
                }

                let handled = self.push_viewport_input(
                    cx.app,
                    ViewportInputArgs {
                        window,
                        viewport,
                        bounds,
                        position: *position,
                        kind: ViewportInputKind::PointerMove {
                            buttons: *buttons,
                            modifiers: *modifiers,
                        },
                        clamped: false,
                    },
                );
                if handled {
                    cx.stop_propagation();
                }
            }
            Event::Pointer(PointerEvent::Up {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                let Some(capture) = self.capture else {
                    return;
                };
                if capture.window != window || capture.button != *button {
                    return;
                }

                self.push_viewport_input(
                    cx.app,
                    ViewportInputArgs {
                        window,
                        viewport: capture.viewport,
                        bounds: capture.bounds,
                        position: *position,
                        kind: ViewportInputKind::PointerUp {
                            button: *button,
                            modifiers: *modifiers,
                            click_count: *click_count,
                        },
                        clamped: true,
                    },
                );

                self.capture = None;
                if cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                if !draw_rect.contains(*position) {
                    return;
                }
                let handled = self.push_viewport_input(
                    cx.app,
                    ViewportInputArgs {
                        window,
                        viewport,
                        bounds,
                        position: *position,
                        kind: ViewportInputKind::Wheel {
                            delta: *delta,
                            modifiers: *modifiers,
                        },
                        clamped: false,
                    },
                );
                if handled {
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Paint);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);

        let theme = cx.theme().snapshot();
        let background = self
            .style
            .background
            .unwrap_or(theme.colors.panel_background);
        let border = self.style.border.unwrap_or(theme.colors.panel_border);
        let border_width = self.style.border_width;

        let bounds = cx.bounds;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: bounds,
            background,
            border: fret_core::Edges::all(border_width),
            border_color: border,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let viewport = self
            .model
            .read(cx.app, |_app, m| m.viewport)
            .unwrap_or_default();
        let draw_rect = viewport.draw_rect(bounds);

        cx.scene.push(SceneOp::ViewportSurface {
            order: DrawOrder(2),
            rect: draw_rect,
            target: viewport.target,
            opacity: viewport.opacity.clamp(0.0, 1.0),
        });
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Viewport);
        cx.set_label("Plot3D");
    }

    fn cleanup_resources(&mut self, _services: &mut dyn UiServices) {}
}
