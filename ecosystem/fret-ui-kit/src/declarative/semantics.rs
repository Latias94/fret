use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsDecoration};

use crate::UiIntoElement;

pub trait AnyElementSemanticsExt {
    fn a11y(self, decoration: SemanticsDecoration) -> AnyElement;
    fn a11y_role(self, role: SemanticsRole) -> AnyElement;
    fn role(self, role: SemanticsRole) -> AnyElement;
    fn a11y_label(self, label: impl Into<Arc<str>>) -> AnyElement;
    fn test_id(self, id: impl Into<Arc<str>>) -> AnyElement;
    fn a11y_value(self, value: impl Into<Arc<str>>) -> AnyElement;
    fn a11y_disabled(self, disabled: bool) -> AnyElement;
    fn a11y_selected(self, selected: bool) -> AnyElement;
    fn a11y_expanded(self, expanded: bool) -> AnyElement;
    fn a11y_checked(self, checked: Option<bool>) -> AnyElement;
}

impl AnyElementSemanticsExt for AnyElement {
    fn a11y(self, decoration: SemanticsDecoration) -> AnyElement {
        self.a11y(decoration)
    }

    fn a11y_role(self, role: SemanticsRole) -> AnyElement {
        self.a11y_role(role)
    }

    fn role(self, role: SemanticsRole) -> AnyElement {
        self.a11y_role(role)
    }

    fn a11y_label(self, label: impl Into<Arc<str>>) -> AnyElement {
        self.a11y_label(label)
    }

    fn test_id(self, id: impl Into<Arc<str>>) -> AnyElement {
        self.test_id(id)
    }

    fn a11y_value(self, value: impl Into<Arc<str>>) -> AnyElement {
        self.a11y_value(value)
    }

    fn a11y_disabled(self, disabled: bool) -> AnyElement {
        self.a11y_disabled(disabled)
    }

    fn a11y_selected(self, selected: bool) -> AnyElement {
        self.a11y_selected(selected)
    }

    fn a11y_expanded(self, expanded: bool) -> AnyElement {
        self.a11y_expanded(expanded)
    }

    fn a11y_checked(self, checked: Option<bool>) -> AnyElement {
        self.a11y_checked(checked)
    }
}

#[derive(Debug, Clone)]
pub struct UiElementWithTestId<T> {
    inner: T,
    test_id: Arc<str>,
}

impl<T> UiElementWithTestId<T> {
    pub fn new(inner: T, test_id: impl Into<Arc<str>>) -> Self {
        Self {
            inner,
            test_id: test_id.into(),
        }
    }
}

impl<T: UiIntoElement> UiIntoElement for UiElementWithTestId<T> {
    #[track_caller]
    fn into_element<H: fret_ui::UiHost>(
        self,
        cx: &mut fret_ui::ElementContext<'_, H>,
    ) -> fret_ui::element::AnyElement {
        self.inner.into_element(cx).test_id(self.test_id)
    }
}

pub trait UiElementTestIdExt: UiIntoElement + Sized {
    fn test_id(self, id: impl Into<Arc<str>>) -> UiElementWithTestId<Self> {
        UiElementWithTestId::new(self, id)
    }
}

impl<T: UiIntoElement> UiElementTestIdExt for T {}

#[derive(Debug, Clone)]
pub struct UiElementWithA11y<T> {
    inner: T,
    decoration: SemanticsDecoration,
}

impl<T> UiElementWithA11y<T> {
    pub fn new(inner: T, decoration: SemanticsDecoration) -> Self {
        Self { inner, decoration }
    }
}

impl<T: UiIntoElement> UiIntoElement for UiElementWithA11y<T> {
    #[track_caller]
    fn into_element<H: fret_ui::UiHost>(
        self,
        cx: &mut fret_ui::ElementContext<'_, H>,
    ) -> fret_ui::element::AnyElement {
        self.inner.into_element(cx).a11y(self.decoration)
    }
}

pub trait UiElementA11yExt: UiIntoElement + Sized {
    fn a11y(self, decoration: SemanticsDecoration) -> UiElementWithA11y<Self> {
        UiElementWithA11y::new(self, decoration)
    }

    fn a11y_role(self, role: SemanticsRole) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().role(role))
    }

    fn a11y_label(self, label: impl Into<Arc<str>>) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().label(label))
    }

    fn a11y_value(self, value: impl Into<Arc<str>>) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().value(value))
    }

    fn a11y_disabled(self, disabled: bool) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().disabled(disabled))
    }

    fn a11y_selected(self, selected: bool) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().selected(selected))
    }

    fn a11y_expanded(self, expanded: bool) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().expanded(expanded))
    }

    fn a11y_checked(self, checked: Option<bool>) -> UiElementWithA11y<Self> {
        self.a11y(SemanticsDecoration::default().checked(checked))
    }
}

impl<T: UiIntoElement> UiElementA11yExt for T {}

#[derive(Debug, Clone)]
pub struct UiElementWithKeyContext<T> {
    inner: T,
    key_context: Arc<str>,
}

impl<T> UiElementWithKeyContext<T> {
    pub fn new(inner: T, key_context: impl Into<Arc<str>>) -> Self {
        Self {
            inner,
            key_context: key_context.into(),
        }
    }
}

impl<T: UiIntoElement> UiIntoElement for UiElementWithKeyContext<T> {
    #[track_caller]
    fn into_element<H: fret_ui::UiHost>(
        self,
        cx: &mut fret_ui::ElementContext<'_, H>,
    ) -> fret_ui::element::AnyElement {
        self.inner.into_element(cx).key_context(self.key_context)
    }
}

pub trait UiElementKeyContextExt: UiIntoElement + Sized {
    fn key_context(self, key_context: impl Into<Arc<str>>) -> UiElementWithKeyContext<Self> {
        UiElementWithKeyContext::new(self, key_context)
    }
}

impl<T: UiIntoElement> UiElementKeyContextExt for T {}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{AppWindowId, Point, PointerId, Px, Rect, Size};
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };

    use super::*;

    #[derive(Default)]
    struct TestUiHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let existing = self.globals.remove(&type_id);
            let mut value = existing
                .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestUiHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestUiHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            Vec::new()
        }
    }

    impl CommandsHost for TestUiHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestUiHost {
        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn push_effect(&mut self, _effect: Effect) {}
    }

    impl TimeHost for TestUiHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let out = TimerToken(self.next_timer_token);
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            out
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let out = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            out
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let out = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            out
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let out = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            out
        }
    }

    impl DragHost for TestUiHost {
        fn drag(&self, _pointer_id: PointerId) -> Option<&DragSession> {
            None
        }

        fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: PointerId) {}

        fn any_drag_session(&self, _predicate: impl FnMut(&DragSession) -> bool) -> bool {
            false
        }

        fn find_drag_pointer_id(
            &self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<PointerId> {
            None
        }

        fn cancel_drag_sessions(
            &mut self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<PointerId> {
            Vec::new()
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }
    }

    #[test]
    fn ui_into_element_exts_allow_semantics_and_key_context_without_early_into_element() {
        struct Dummy;
        impl UiIntoElement for Dummy {
            #[track_caller]
            fn into_element<H: fret_ui::UiHost>(
                self,
                cx: &mut fret_ui::ElementContext<'_, H>,
            ) -> fret_ui::element::AnyElement {
                cx.text("dummy")
            }
        }

        let mut host = TestUiHost::default();
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            window,
            bounds,
            "root",
        );

        let el = Dummy
            .a11y_role(SemanticsRole::Button)
            .test_id("dummy.btn")
            .key_context("dummy.ctx")
            .into_element(&mut cx);

        assert_eq!(el.key_context.as_deref(), Some("dummy.ctx"));
        let deco = el
            .semantics_decoration
            .expect("expected semantics decoration");
        assert_eq!(deco.role, Some(SemanticsRole::Button));
        assert_eq!(deco.test_id.as_deref(), Some("dummy.btn"));
    }
}
