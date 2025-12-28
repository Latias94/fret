use crate::flow_column::FlowColumn;
use fret_core::{CursorIcon, Event, KeyCode, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};

/// shadcn/ui `Collapsible` root.
///
/// In the DOM, the root does not prescribe layout; children participate in normal flow.
/// In Fret's retained tree we need a layout container, so we map it to a vertical flow container.
pub type Collapsible = FlowColumn;

pub struct CollapsibleTrigger {
    open: Model<bool>,
    disabled: bool,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
}

impl CollapsibleTrigger {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            disabled: false,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn is_open<H: UiHost>(&self, app: &H) -> bool {
        app.models().get(self.open).copied().unwrap_or(false)
    }

    fn toggle_open<H: UiHost>(&self, app: &mut H) {
        let _ = app.models_mut().update(self.open, |v| *v = !*v);
    }
}

impl<H: UiHost> Widget<H> for CollapsibleTrigger {
    fn cleanup_resources(&mut self, _services: &mut dyn fret_core::UiServices) {}

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Button);
        cx.set_disabled(self.disabled);
        cx.set_focusable(!self.disabled);
        cx.set_invokable(!self.disabled);
        cx.set_expanded(self.is_open(cx.app));
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.last_bounds = cx.bounds;

        if self.disabled {
            return;
        }

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.last_bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if hovered || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if !self.last_bounds.contains(*position) {
                        return;
                    }
                    self.pressed = true;
                    cx.capture_pointer(cx.node);
                    cx.request_focus(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let was_pressed = self.pressed;
                    self.pressed = false;
                    cx.release_pointer_capture();

                    let hovered = self.last_bounds.contains(*position);
                    self.hovered = hovered;
                    if was_pressed && hovered {
                        self.toggle_open(cx.app);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    } else {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
                    return;
                }
                if !self.pressed {
                    self.pressed = true;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                cx.stop_propagation();
            }
            Event::KeyUp { key, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
                    return;
                }
                if self.pressed {
                    self.pressed = false;
                    self.toggle_open(cx.app);
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.open, Invalidation::Layout);
        self.last_bounds = cx.bounds;

        let Some(&child) = cx.children.first() else {
            return Size::new(Px(0.0), Px(0.0));
        };

        let probe = Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
        let size = cx.layout_in(child, probe);
        let final_bounds = Rect::new(cx.bounds.origin, Size::new(cx.available.width, size.height));
        let _ = cx.layout_in(child, final_bounds);
        Size::new(cx.available.width, size.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.open, Invalidation::Paint);
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

pub struct CollapsibleContent {
    open: Model<bool>,
    open_cached: bool,
}

impl CollapsibleContent {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            open_cached: false,
        }
    }

    fn is_open<H: UiHost>(&self, app: &H) -> bool {
        app.models().get(self.open).copied().unwrap_or(false)
    }
}

impl<H: UiHost> Widget<H> for CollapsibleContent {
    fn is_focusable(&self) -> bool {
        false
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        self.open_cached
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.open, Invalidation::Layout);
        let open = self.is_open(cx.app);
        self.open_cached = open;
        if !open {
            return Size::new(Px(0.0), Px(0.0));
        }

        let Some(&child) = cx.children.first() else {
            return Size::new(Px(0.0), Px(0.0));
        };

        let probe = Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
        let size = cx.layout_in(child, probe);
        let final_bounds = Rect::new(cx.bounds.origin, Size::new(cx.available.width, size.height));
        let _ = cx.layout_in(child, final_bounds);
        Size::new(cx.available.width, size.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.open, Invalidation::Paint);
        let open = self.is_open(cx.app);
        self.open_cached = open;
        if !open {
            return;
        }
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
