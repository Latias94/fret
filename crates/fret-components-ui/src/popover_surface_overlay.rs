use std::collections::HashMap;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, NodeId, Point, Px, Rect,
    SceneOp, Size,
};
use fret_runtime::CommandId;
use fret_ui::paint::paint_shadow;
use fret_ui::{
    Theme, UiHost,
    widget::{EventCx, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_ui::overlay_placement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverSurfaceSide {
    Top,
    Bottom,
    Left,
    Right,
}

impl Default for PopoverSurfaceSide {
    fn default() -> Self {
        Self::Bottom
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverSurfaceAlign {
    Start,
    Center,
    End,
}

impl Default for PopoverSurfaceAlign {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, Clone)]
pub struct PopoverSurfaceRequest {
    pub owner: NodeId,
    pub anchor: Rect,
    pub content_node: NodeId,
    pub side: PopoverSurfaceSide,
    pub align: PopoverSurfaceAlign,
    pub request_focus: bool,
    pub close_on_escape: bool,
    pub close_on_click_outside: bool,
}

impl PopoverSurfaceRequest {
    pub fn new(owner: NodeId, anchor: Rect, content_node: NodeId) -> Self {
        Self {
            owner,
            anchor,
            content_node,
            side: PopoverSurfaceSide::default(),
            align: PopoverSurfaceAlign::default(),
            request_focus: true,
            close_on_escape: true,
            close_on_click_outside: true,
        }
    }

    pub fn side(mut self, side: PopoverSurfaceSide) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: PopoverSurfaceAlign) -> Self {
        self.align = align;
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
}

#[derive(Debug, Clone)]
pub struct PopoverSurfaceStyle {
    pub padding: Px,
    pub corner_radius: Px,
    pub background: Color,
    pub border_color: Color,
    pub border_width: Px,
    pub shadow: Option<fret_ui::element::ShadowStyle>,
    pub side_offset: Px,
}

impl Default for PopoverSurfaceStyle {
    fn default() -> Self {
        Self {
            padding: Px(10.0),
            corner_radius: Px(10.0),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            border_width: Px(1.0),
            shadow: None,
            side_offset: Px(8.0),
        }
    }
}

#[derive(Debug, Default)]
pub struct PopoverSurfaceService {
    next_serial: u64,
    by_window: HashMap<fret_core::AppWindowId, PopoverSurfaceEntry>,
}

#[derive(Debug)]
struct PopoverSurfaceEntry {
    serial: u64,
    request: PopoverSurfaceRequest,
}

impl PopoverSurfaceService {
    pub fn set_request(&mut self, window: fret_core::AppWindowId, request: PopoverSurfaceRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window
            .insert(window, PopoverSurfaceEntry { serial, request });
    }

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &PopoverSurfaceRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.serial, &entry.request))
    }

    pub fn clear_request(&mut self, window: fret_core::AppWindowId) {
        self.by_window.remove(&window);
    }
}

/// Anchored popover surface overlay (non-modal).
///
/// This is the "popover shell" for components that need an anchored panel, but want to provide
/// their own content subtree (e.g. DatePicker, rich navigation menus).
///
/// Open/close + focus restoration is handled by `WindowOverlays`.
#[derive(Debug)]
pub struct PopoverSurfaceOverlay {
    style: PopoverSurfaceStyle,
    close_command: CommandId,
    last_theme_revision: Option<u64>,
    last_serial: Option<u64>,
    request: Option<PopoverSurfaceRequest>,
    panel_bounds: Rect,
    panel_border: Edges,
    panel_radii: Corners,
}

impl PopoverSurfaceOverlay {
    pub fn new() -> Self {
        Self {
            style: PopoverSurfaceStyle::default(),
            close_command: CommandId::from("popover_surface.close"),
            last_theme_revision: None,
            last_serial: None,
            request: None,
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

        self.style.padding = theme.metrics.padding_sm;
        self.style.corner_radius = theme.metrics.radius_md;
        self.style.background = theme
            .color_by_key("popover.background")
            .unwrap_or(theme.colors.menu_background);
        self.style.border_color = theme
            .color_by_key("popover.border")
            .or_else(|| theme.color_by_key("border"))
            .unwrap_or(theme.colors.menu_border);
        self.style.border_width = Px(1.0);
        self.style.shadow = Some(crate::declarative::style::shadow_md(
            theme,
            self.style.corner_radius,
        ));
        self.style.side_offset = theme
            .metric_by_key("component.popover_surface.side_offset")
            .unwrap_or(Px(8.0));
    }

    fn compute_panel_bounds(
        &self,
        outer: Rect,
        request: &PopoverSurfaceRequest,
        content: Size,
    ) -> Rect {
        compute_anchored_panel_bounds(
            outer,
            request.anchor,
            content,
            self.style.side_offset,
            request.side,
            request.align,
        )
    }

    fn close(&self, cx: &mut EventCx<'_, impl UiHost>, window: fret_core::AppWindowId) {
        cx.dispatch_command(self.close_command.clone());
        cx.request_redraw();
        cx.stop_propagation();

        let _ = window;
    }
}

fn compute_anchored_panel_bounds(
    outer: Rect,
    anchor: Rect,
    content: Size,
    side_offset: Px,
    preferred_side: PopoverSurfaceSide,
    align: PopoverSurfaceAlign,
) -> Rect {
    overlay_placement::anchored_panel_bounds(
        outer,
        anchor,
        content,
        side_offset,
        match preferred_side {
            PopoverSurfaceSide::Top => overlay_placement::Side::Top,
            PopoverSurfaceSide::Bottom => overlay_placement::Side::Bottom,
            PopoverSurfaceSide::Left => overlay_placement::Side::Left,
            PopoverSurfaceSide::Right => overlay_placement::Side::Right,
        },
        match align {
            PopoverSurfaceAlign::Start => overlay_placement::Align::Start,
            PopoverSurfaceAlign::Center => overlay_placement::Align::Center,
            PopoverSurfaceAlign::End => overlay_placement::Align::End,
        },
    )
}

impl Default for PopoverSurfaceOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for PopoverSurfaceOverlay {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Panel);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };
        let Some(request) = self.request.clone() else {
            return;
        };

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if request.close_on_click_outside && !self.panel_bounds.contains(*position) {
                        self.close(cx, window);
                    }
                }
                _ => {}
            },
            Event::KeyDown { key, modifiers, .. } => {
                if modifiers.ctrl || modifiers.meta || modifiers.alt {
                    return;
                }
                if request.close_on_escape && *key == KeyCode::Escape {
                    self.close(cx, window);
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            self.request = None;
            return cx.available;
        };

        let Some((serial, request)) = cx
            .app
            .global::<PopoverSurfaceService>()
            .and_then(|s| s.request(window))
            .map(|(s, r)| (s, r.clone()))
        else {
            self.request = None;
            self.last_serial = None;
            self.panel_bounds = Rect::default();
            return cx.available;
        };

        if self.last_serial != Some(serial) {
            self.last_serial = Some(serial);
        }
        self.request = Some(request.clone());

        let outer = Rect::new(cx.bounds.origin, cx.available);

        // Measure the content with a large probe, then place it under the anchor.
        let probe = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(1.0e9), Px(1.0e9)),
        );
        let content_size = cx.layout_in(request.content_node, probe);

        let pad = Px(self.style.padding.0.max(0.0));
        let desired = Size::new(
            Px((content_size.width.0 + pad.0 * 2.0).max(0.0)),
            Px((content_size.height.0 + pad.0 * 2.0).max(0.0)),
        );
        self.panel_bounds = self.compute_panel_bounds(outer, &request, desired);

        // Layout all children: only the requested content node is visible; others collapse.
        for &child in cx.children {
            if child == request.content_node {
                let inner = Rect::new(
                    Point::new(
                        Px(self.panel_bounds.origin.x.0 + pad.0),
                        Px(self.panel_bounds.origin.y.0 + pad.0),
                    ),
                    Size::new(
                        Px((self.panel_bounds.size.width.0 - pad.0 * 2.0).max(0.0)),
                        Px((self.panel_bounds.size.height.0 - pad.0 * 2.0).max(0.0)),
                    ),
                );
                let _ = cx.layout_in(child, inner);
            } else {
                let _ = cx.layout_in(
                    child,
                    Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0))),
                );
            }
        }

        self.panel_border = Edges::all(self.style.border_width);
        self.panel_radii = Corners::all(self.style.corner_radius);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };
        let _ = window;

        let Some(request) = self.request.clone() else {
            return;
        };

        if let Some(mut shadow) = self.style.shadow {
            shadow.corner_radii = self.panel_radii;
            paint_shadow(cx.scene, DrawOrder(0), self.panel_bounds, shadow);
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: self.panel_bounds,
            background: self.style.background,
            border: self.panel_border,
            border_color: self.style.border_color,
            corner_radii: self.panel_radii,
        });

        for &child in cx.children {
            if child == request.content_node {
                let bounds = cx.child_bounds(child).unwrap_or(cx.bounds);
                cx.paint(child, bounds);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flips_from_bottom_to_top_when_bottom_overflows() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(190.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let content = Size::new(Px(120.0), Px(80.0));

        let placed = compute_anchored_panel_bounds(
            outer,
            anchor,
            content,
            Px(8.0),
            PopoverSurfaceSide::Bottom,
            PopoverSurfaceAlign::Start,
        );
        // Top placement should be above the anchor.
        assert!(placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0);
        // Always clamped to outer.
        assert!(outer.contains(placed.origin));
    }

    #[test]
    fn keeps_bottom_when_it_fits() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(400.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(10.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let content = Size::new(Px(120.0), Px(80.0));

        let placed = compute_anchored_panel_bounds(
            outer,
            anchor,
            content,
            Px(8.0),
            PopoverSurfaceSide::Bottom,
            PopoverSurfaceAlign::Start,
        );
        assert!(placed.origin.y.0 >= anchor.origin.y.0 + anchor.size.height.0);
    }
}
