use std::collections::HashMap;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, NodeId, Px, Rect, SceneOp, Size,
};
use fret_runtime::CommandId;
use fret_ui::{
    Theme, UiHost,
    widget::{EventCx, LayoutCx, PaintCx, SemanticsCx, Widget},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct SheetRequest {
    pub owner: NodeId,
    pub side: SheetSide,
    pub request_focus: bool,
    pub close_on_escape: bool,
    pub close_on_click_outside: bool,
    /// Optional fixed extent for the sheet along its primary axis (width for left/right, height for top/bottom).
    pub extent: Option<Px>,
}

impl SheetRequest {
    pub fn new(owner: NodeId) -> Self {
        Self {
            owner,
            side: SheetSide::Right,
            request_focus: true,
            close_on_escape: true,
            close_on_click_outside: true,
            extent: None,
        }
    }

    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    pub fn request_focus(mut self, request_focus: bool) -> Self {
        self.request_focus = request_focus;
        self
    }

    pub fn close_on_escape(mut self, close_on_escape: bool) -> Self {
        self.close_on_escape = close_on_escape;
        self
    }

    pub fn close_on_click_outside(mut self, close_on_click_outside: bool) -> Self {
        self.close_on_click_outside = close_on_click_outside;
        self
    }

    pub fn extent(mut self, extent: Px) -> Self {
        self.extent = Some(extent);
        self
    }
}

#[derive(Debug, Default)]
pub struct SheetService {
    next_serial: u64,
    by_window: HashMap<fret_core::AppWindowId, SheetEntry>,
}

#[derive(Debug)]
struct SheetEntry {
    serial: u64,
    request: SheetRequest,
}

impl SheetService {
    pub fn set_request(&mut self, window: fret_core::AppWindowId, request: SheetRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window
            .insert(window, SheetEntry { serial, request });
    }

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &SheetRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.serial, &entry.request))
    }

    pub fn clear(&mut self, window: fret_core::AppWindowId) {
        self.by_window.remove(&window);
    }
}

#[derive(Debug, Clone)]
pub struct SheetStyle {
    pub backdrop_opacity: f32,
    pub padding: Px,
    pub corner_radius: Px,
    pub background: Color,
    pub border_color: Color,
    pub shadow: Option<fret_ui::element::ShadowStyle>,
    pub max_extent: Px,
}

impl Default for SheetStyle {
    fn default() -> Self {
        Self {
            backdrop_opacity: 0.55,
            padding: Px(14.0),
            corner_radius: Px(10.0),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            shadow: None,
            max_extent: Px(420.0),
        }
    }
}

/// Modal sheet overlay shell (shadcn-style).
///
/// This widget:
/// - paints a backdrop + a side-attached panel,
/// - lays out its children inside the panel (single content root recommended),
/// - closes on `Escape` and clicking outside the panel by dispatching `sheet.close` (policy-driven via `SheetRequest`).
///
/// Notes:
/// - The open/close + focus restoration policy is managed by `WindowOverlays`.
/// - The sheet panel content is provided by the app/component layer under the overlay root node.
#[derive(Debug)]
pub struct SheetOverlay {
    style: SheetStyle,
    close_command: CommandId,
    last_theme_revision: Option<u64>,
    last_serial: Option<u64>,
    panel_bounds: Rect,
    panel_border: Edges,
    panel_radii: Corners,
}

impl SheetOverlay {
    pub fn new() -> Self {
        Self {
            style: SheetStyle::default(),
            close_command: CommandId::from("sheet.close"),
            last_theme_revision: None,
            last_serial: None,
            panel_bounds: Rect::default(),
            panel_border: Edges::all(Px(0.0)),
            panel_radii: Corners::all(Px(0.0)),
        }
    }

    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = command;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.style.padding = theme.metrics.padding_md;
        self.style.corner_radius = theme.metrics.radius_lg;
        self.style.background = theme
            .color_by_key("card")
            .unwrap_or(theme.colors.panel_background);
        self.style.border_color = theme
            .color_by_key("border")
            .unwrap_or(theme.colors.panel_border);
        self.style.shadow = Some(crate::declarative::style::shadow_lg(
            theme,
            self.style.corner_radius,
        ));
        self.style.max_extent = theme
            .metric_by_key("component.sheet.max_extent")
            .unwrap_or(Px(420.0));
    }

    fn compute_panel_bounds(&self, outer: Rect, available: Size, request: &SheetRequest) -> Rect {
        let max_extent = self.style.max_extent.0.max(0.0);
        let default_extent = match request.side {
            SheetSide::Left | SheetSide::Right => (available.width.0 * 0.75).max(0.0),
            SheetSide::Top | SheetSide::Bottom => (available.height.0 * 0.75).max(0.0),
        };
        let extent = request
            .extent
            .map(|e| e.0.max(0.0))
            .unwrap_or(default_extent)
            .min(max_extent)
            .max(0.0);

        match request.side {
            SheetSide::Left => Rect::new(
                outer.origin,
                Size::new(Px(extent.min(available.width.0).max(0.0)), available.height),
            ),
            SheetSide::Right => Rect::new(
                fret_core::Point::new(
                    Px(outer.origin.x.0 + (available.width.0 - extent).max(0.0)),
                    outer.origin.y,
                ),
                Size::new(Px(extent.min(available.width.0).max(0.0)), available.height),
            ),
            SheetSide::Top => Rect::new(
                outer.origin,
                Size::new(available.width, Px(extent.min(available.height.0).max(0.0))),
            ),
            SheetSide::Bottom => Rect::new(
                fret_core::Point::new(
                    outer.origin.x,
                    Px(outer.origin.y.0 + (available.height.0 - extent).max(0.0)),
                ),
                Size::new(available.width, Px(extent.min(available.height.0).max(0.0))),
            ),
        }
    }

    fn compute_panel_border_and_radii(&mut self, request: &SheetRequest) {
        let w = Px(1.0);
        self.panel_border = match request.side {
            SheetSide::Left => Edges {
                top: Px(0.0),
                right: w,
                bottom: Px(0.0),
                left: Px(0.0),
            },
            SheetSide::Right => Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(0.0),
                left: w,
            },
            SheetSide::Top => Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: w,
                left: Px(0.0),
            },
            SheetSide::Bottom => Edges {
                top: w,
                right: Px(0.0),
                bottom: Px(0.0),
                left: Px(0.0),
            },
        };

        let r = self.style.corner_radius;
        self.panel_radii = match request.side {
            SheetSide::Left => Corners {
                top_left: Px(0.0),
                top_right: r,
                bottom_right: r,
                bottom_left: Px(0.0),
            },
            SheetSide::Right => Corners {
                top_left: r,
                top_right: Px(0.0),
                bottom_right: Px(0.0),
                bottom_left: r,
            },
            SheetSide::Top => Corners {
                top_left: Px(0.0),
                top_right: Px(0.0),
                bottom_right: r,
                bottom_left: r,
            },
            SheetSide::Bottom => Corners {
                top_left: r,
                top_right: r,
                bottom_right: Px(0.0),
                bottom_left: Px(0.0),
            },
        };
    }
}

impl Default for SheetOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for SheetOverlay {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };
        let Some((serial, request)) = cx
            .app
            .global::<SheetService>()
            .and_then(|s| s.request(window))
            .map(|(s, r)| (s, r.clone()))
        else {
            self.last_serial = None;
            return;
        };
        self.last_serial = Some(serial);

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if request.close_on_escape
                    && *key == KeyCode::Escape
                    && !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.meta
                {
                    cx.dispatch_command(self.close_command.clone());
                    cx.stop_propagation();
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                ..
            }) => {
                if request.close_on_click_outside
                    && *button == MouseButton::Left
                    && !self.panel_bounds.contains(*position)
                {
                    cx.dispatch_command(self.close_command.clone());
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            self.panel_bounds = Rect::default();
            self.last_serial = None;
            return cx.available;
        };

        let Some(service) = cx.app.global::<SheetService>() else {
            self.panel_bounds = Rect::default();
            self.last_serial = None;
            return cx.available;
        };
        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            self.panel_bounds = Rect::default();
            self.last_serial = None;
            return cx.available;
        };

        self.last_serial = Some(serial);
        self.panel_bounds = self.compute_panel_bounds(cx.bounds, cx.available, &request);
        self.compute_panel_border_and_radii(&request);

        let pad = self.style.padding.0.max(0.0);
        let inner = Rect::new(
            fret_core::Point::new(
                Px(self.panel_bounds.origin.x.0 + pad),
                Px(self.panel_bounds.origin.y.0 + pad),
            ),
            Size::new(
                Px((self.panel_bounds.size.width.0 - pad * 2.0).max(0.0)),
                Px((self.panel_bounds.size.height.0 - pad * 2.0).max(0.0)),
            ),
        );

        if let Some(&content) = cx.children.first() {
            let _ = cx.layout_in(content, inner);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());

        let Some(window) = cx.window else {
            return;
        };
        let Some(service) = cx.app.global::<SheetService>() else {
            return;
        };
        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            self.last_serial = None;
            self.panel_bounds = Rect::default();
            return;
        };
        if self.last_serial != Some(serial) {
            self.last_serial = Some(serial);
        }

        self.compute_panel_border_and_radii(&request);

        let base = cx.theme().colors.surface_background;
        let backdrop_opacity = self.style.backdrop_opacity.clamp(0.0, 1.0);
        let backdrop = Color {
            a: backdrop_opacity,
            ..base
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: backdrop,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        if let Some(shadow) = self.style.shadow {
            fret_ui::paint::paint_shadow(cx.scene, DrawOrder(1), self.panel_bounds, shadow);
        }
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: self.panel_bounds,
            background: self.style.background,
            border: self.panel_border,
            border_color: self.style.border_color,
            corner_radii: self.panel_radii,
        });

        if let Some(&content) = cx.children.first() {
            if let Some(bounds) = cx.child_bounds(content) {
                cx.paint(content, bounds);
            } else {
                cx.paint(content, cx.bounds);
            }
        }
    }

    fn cleanup_resources(&mut self, _services: &mut dyn fret_core::UiServices) {
        self.last_theme_revision = None;
        self.last_serial = None;
        self.panel_bounds = Rect::default();
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Panel);
    }
}
