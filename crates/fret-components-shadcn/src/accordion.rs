use std::sync::Arc;

use fret_core::{CursorIcon, Event, KeyCode, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use fret_components_ui::widget_primitives::Column;

/// shadcn/ui `Accordion` root.
///
/// In the DOM, children participate in normal flow; in Fret's retained tree we map this to a
/// vertical flow container.
pub type Accordion = Column;

/// shadcn/ui `AccordionItem`.
///
/// This is a neutral container; styling (e.g. borders) is left to the caller.
pub type AccordionItem = Column;

#[derive(Clone, Copy)]
enum SelectionModel {
    Single(Model<Option<Arc<str>>>),
    Multiple(Model<Vec<Arc<str>>>),
}

pub struct AccordionTrigger {
    selection: SelectionModel,
    value: Arc<str>,
    disabled: bool,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
}

impl AccordionTrigger {
    pub fn single(selection: Model<Option<Arc<str>>>, value: impl Into<Arc<str>>) -> Self {
        Self::new(SelectionModel::Single(selection), value)
    }

    pub fn multiple(selection: Model<Vec<Arc<str>>>, value: impl Into<Arc<str>>) -> Self {
        Self::new(SelectionModel::Multiple(selection), value)
    }

    fn new(selection: SelectionModel, value: impl Into<Arc<str>>) -> Self {
        Self {
            selection,
            value: value.into(),
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
        match self.selection {
            SelectionModel::Single(m) => {
                app.models().get(m).and_then(|v| v.as_ref()) == Some(&self.value)
            }
            SelectionModel::Multiple(m) => app
                .models()
                .get(m)
                .map(|v| v.iter().any(|x| x == &self.value))
                .unwrap_or(false),
        }
    }

    fn toggle_open<H: UiHost>(&self, app: &mut H) {
        match self.selection {
            SelectionModel::Single(m) => {
                let value = self.value.clone();
                let _ = app.models_mut().update(m, |v| {
                    if v.as_ref() == Some(&value) {
                        *v = None;
                    } else {
                        *v = Some(value);
                    }
                });
            }
            SelectionModel::Multiple(m) => {
                let value = self.value.clone();
                let _ = app.models_mut().update(m, |v| {
                    if let Some(pos) = v.iter().position(|x| x == &value) {
                        v.remove(pos);
                    } else {
                        v.push(value);
                    }
                });
            }
        }
    }

    fn observe_models<H: UiHost>(&self, cx: &mut impl ObserveModelCx<H>) {
        match self.selection {
            SelectionModel::Single(m) => cx.observe_model(m, Invalidation::Layout),
            SelectionModel::Multiple(m) => cx.observe_model(m, Invalidation::Layout),
        }
    }
}

trait ObserveModelCx<H: UiHost> {
    fn observe_model<T: std::any::Any>(&mut self, model: Model<T>, inv: Invalidation);
}

impl<'a, H: UiHost> ObserveModelCx<H> for LayoutCx<'a, H> {
    fn observe_model<T: std::any::Any>(&mut self, model: Model<T>, inv: Invalidation) {
        LayoutCx::observe_model(self, model, inv)
    }
}

impl<'a, H: UiHost> ObserveModelCx<H> for PaintCx<'a, H> {
    fn observe_model<T: std::any::Any>(&mut self, model: Model<T>, inv: Invalidation) {
        PaintCx::observe_model(self, model, inv)
    }
}

impl<H: UiHost> Widget<H> for AccordionTrigger {
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
        self.observe_models(cx);
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
        match self.selection {
            SelectionModel::Single(m) => cx.observe_model(m, Invalidation::Paint),
            SelectionModel::Multiple(m) => cx.observe_model(m, Invalidation::Paint),
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

pub struct AccordionContent {
    selection: SelectionModel,
    value: Arc<str>,
    open_cached: bool,
}

impl AccordionContent {
    pub fn single(selection: Model<Option<Arc<str>>>, value: impl Into<Arc<str>>) -> Self {
        Self::new(SelectionModel::Single(selection), value)
    }

    pub fn multiple(selection: Model<Vec<Arc<str>>>, value: impl Into<Arc<str>>) -> Self {
        Self::new(SelectionModel::Multiple(selection), value)
    }

    fn new(selection: SelectionModel, value: impl Into<Arc<str>>) -> Self {
        Self {
            selection,
            value: value.into(),
            open_cached: false,
        }
    }

    fn is_open<H: UiHost>(&self, app: &H) -> bool {
        match self.selection {
            SelectionModel::Single(m) => {
                app.models().get(m).and_then(|v| v.as_ref()) == Some(&self.value)
            }
            SelectionModel::Multiple(m) => app
                .models()
                .get(m)
                .map(|v| v.iter().any(|x| x == &self.value))
                .unwrap_or(false),
        }
    }
}

impl<H: UiHost> Widget<H> for AccordionContent {
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
        match self.selection {
            SelectionModel::Single(m) => cx.observe_model(m, Invalidation::Layout),
            SelectionModel::Multiple(m) => cx.observe_model(m, Invalidation::Layout),
        }

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
        match self.selection {
            SelectionModel::Single(m) => cx.observe_model(m, Invalidation::Paint),
            SelectionModel::Multiple(m) => cx.observe_model(m, Invalidation::Paint),
        }

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
