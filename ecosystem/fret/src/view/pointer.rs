use std::any::Any;
use std::sync::Arc;

pub use fret_core::{CursorIcon, MouseButton, Point, PointerId};
use fret_core::{Px, Rect};
use fret_runtime::DefaultAction;
use fret_ui::action::{ActionCx, UiPointerActionHost};
pub use fret_ui::action::{
    PointerCancelCx as PointerCancel, PointerDownCx as PointerDown, PointerMoveCx as PointerMove,
    PointerUpCx as PointerUp, WheelCx as Wheel,
};
use fret_ui::element::Length;
use fret_ui::{Invalidation, UiHost};

use super::{AppUi, LocalState, LocalStateModelStoreExt as _};

/// App-facing pointer listener region props.
///
/// This is the default-lane wrapper around `fret-ui`'s mechanism-level `PointerRegionProps`.
/// It exposes the stable authoring knobs ordinary app code needs without teaching raw element
/// props or pointer action host plumbing.
#[derive(Debug, Clone)]
pub struct PointerRegion {
    props: fret_ui::element::PointerRegionProps,
}

impl Default for PointerRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerRegion {
    pub fn new() -> Self {
        Self {
            props: fret_ui::element::PointerRegionProps::default(),
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.props.enabled = enabled;
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.props.layout.size.width = Length::Fill;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.props.layout.size.height = Length::Fill;
        self
    }

    pub fn size_full(mut self) -> Self {
        self.props.layout.size.width = Length::Fill;
        self.props.layout.size.height = Length::Fill;
        self
    }

    pub fn width_px(mut self, width: Px) -> Self {
        self.props.layout.size.width = Length::Px(width);
        self
    }

    pub fn height_px(mut self, height: Px) -> Self {
        self.props.layout.size.height = Length::Px(height);
        self
    }

    pub fn capture_phase_pointer_moves(mut self, enabled: bool) -> Self {
        self.props.capture_phase_pointer_moves = enabled;
        self
    }

    pub(crate) fn into_raw(self) -> fret_ui::element::PointerRegionProps {
        self.props
    }
}

/// Builder passed to `AppUi::pointer_region(...)`.
#[doc(hidden)]
pub struct AppPointerRegion<'cx, 'a, H: UiHost> {
    cx: &'cx mut fret_ui::ElementContext<'a, H>,
}

impl<'cx, 'a, H: UiHost> AppPointerRegion<'cx, 'a, H> {
    pub fn on_pointer_down(
        &mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerDown) -> bool + 'static,
    ) -> &mut Self {
        self.cx
            .pointer_region_on_pointer_down(Arc::new(move |host, action_cx, down| {
                let mut cx = PointerActionCx::new(host, action_cx);
                handler(&mut cx, down)
            }));
        self
    }

    pub fn on_pointer_move(
        &mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerMove) -> bool + 'static,
    ) -> &mut Self {
        self.cx
            .pointer_region_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                let mut cx = PointerActionCx::new(host, action_cx);
                handler(&mut cx, mv)
            }));
        self
    }

    pub fn on_pointer_up(
        &mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerUp) -> bool + 'static,
    ) -> &mut Self {
        self.cx
            .pointer_region_on_pointer_up(Arc::new(move |host, action_cx, up| {
                let mut cx = PointerActionCx::new(host, action_cx);
                handler(&mut cx, up)
            }));
        self
    }

    pub fn on_pointer_cancel(
        &mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerCancel) -> bool + 'static,
    ) -> &mut Self {
        self.cx
            .pointer_region_on_pointer_cancel(Arc::new(move |host, action_cx, cancel| {
                let mut cx = PointerActionCx::new(host, action_cx);
                handler(&mut cx, cancel)
            }));
        self
    }
}

/// App-facing action context for pointer-region handlers.
///
/// This owns the default app-lane operations that ordinary custom pointer surfaces need during
/// dispatch: capture/release, cursor updates, runtime-default suppression, LocalState reads and
/// writes, view-cache notification, redraw, and node invalidation.
pub struct PointerActionCx<'a> {
    host: &'a mut dyn UiPointerActionHost,
    action_cx: ActionCx,
}

impl<'a> PointerActionCx<'a> {
    pub(super) fn new(host: &'a mut dyn UiPointerActionHost, action_cx: ActionCx) -> Self {
        Self { host, action_cx }
    }

    pub fn bounds(&self) -> Rect {
        self.host.bounds()
    }

    pub fn capture_pointer(&mut self) {
        self.host.capture_pointer();
    }

    pub fn release_pointer_capture(&mut self) {
        self.host.release_pointer_capture();
    }

    pub fn set_cursor_icon(&mut self, icon: CursorIcon) {
        self.host.set_cursor_icon(icon);
    }

    pub fn prevent_focus_on_pointer_down(&mut self) {
        self.host.prevent_default(DefaultAction::FocusOnPointerDown);
    }

    pub fn invalidate_paint(&mut self) {
        self.host.invalidate(Invalidation::Paint);
    }

    pub fn invalidate_layout(&mut self) {
        self.host.invalidate(Invalidation::Layout);
    }

    pub fn request_redraw(&mut self) {
        self.host.request_redraw(self.action_cx.window);
    }

    pub fn notify(&mut self) {
        self.host.notify(self.action_cx);
    }

    pub fn local_value<T>(&mut self, local: &LocalState<T>) -> T
    where
        T: Any + Clone,
    {
        local
            .value_in(self.host.models_mut())
            .expect("LocalState-first pointer handlers should read initialized locals")
    }

    pub fn local_value_or<T>(&mut self, local: &LocalState<T>, default: T) -> T
    where
        T: Any + Clone,
    {
        local.value_in_or(self.host.models_mut(), default)
    }

    pub fn set_local<T>(&mut self, local: &LocalState<T>, value: T) -> bool
    where
        T: Any,
    {
        self.update_local(local, move |slot| *slot = value)
    }

    pub fn update_local<T>(&mut self, local: &LocalState<T>, update: impl FnOnce(&mut T)) -> bool
    where
        T: Any,
    {
        let handled = local.update_in(self.host.models_mut(), update);
        if handled {
            self.request_redraw();
            self.notify();
        }
        handled
    }

    pub fn update_local_if<T>(
        &mut self,
        local: &LocalState<T>,
        update: impl FnOnce(&mut T) -> bool,
    ) -> bool
    where
        T: Any,
    {
        let handled = local.update_in_if(self.host.models_mut(), update);
        if handled {
            self.request_redraw();
            self.notify();
        }
        handled
    }
}

impl<'cx, 'a, H: UiHost> AppUi<'cx, 'a, H> {
    #[track_caller]
    pub fn pointer_region<I>(
        &mut self,
        region: PointerRegion,
        f: impl for<'b> FnOnce(&mut AppPointerRegion<'b, 'a, H>) -> I,
    ) -> fret_ui::element::AnyElement
    where
        I: IntoIterator,
        I::Item: fret_ui_kit::IntoUiElement<H>,
    {
        self.cx.pointer_region(region.into_raw(), |cx| {
            let mut pointer = AppPointerRegion { cx };
            let built = f(&mut pointer);
            let mut children = Vec::new();
            for child in built {
                children.push(fret_ui_kit::land_child(&mut *pointer.cx, child));
            }
            children
        })
    }
}
