//! Drag-to-edit (scrub) primitive for numeric values.
//!
//! This is an editor-grade "hand feel" primitive:
//! - pointer down begins an edit session and captures the pre-edit value,
//! - pointer move scrubs horizontally with modifier-based multipliers,
//! - pointer up commits,
//! - Escape cancels to the pre-edit value.
mod behavior;
mod options;
mod state;

use std::sync::{Arc, Mutex};

use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{AnyElement, Length, PressableA11y, PressableProps};
use fret_ui::{ElementContext, Theme, UiHost};

use super::EditorDensity;
use behavior::install_drag_value_core_behavior;
pub use options::DragValueCoreOptions;
use options::resolve_options;
use state::DragState;

#[cfg(test)]
mod tests;

pub trait DragValueScalar: Copy + PartialEq + 'static {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
}

impl DragValueScalar for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }

    fn from_f64(v: f64) -> Self {
        v as f32
    }
}

impl DragValueScalar for f64 {
    fn to_f64(self) -> f64 {
        self
    }

    fn from_f64(v: f64) -> Self {
        v
    }
}

impl DragValueScalar for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }

    fn from_f64(v: f64) -> Self {
        v.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DragValueCoreResponse {
    dragging: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl DragValueCoreResponse {
    pub(crate) fn new(dragging: bool, hovered: bool, pressed: bool, focused: bool) -> Self {
        Self {
            dragging,
            hovered,
            pressed,
            focused,
        }
    }

    pub fn dragging(self) -> bool {
        self.dragging
    }

    pub fn hovered(self) -> bool {
        self.hovered
    }

    pub fn pressed(self) -> bool {
        self.pressed
    }

    pub fn focused(self) -> bool {
        self.focused
    }
}

#[derive(Clone)]
pub struct DragValueCore<T> {
    value: T,
    on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static>,
    on_commit: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    on_cancel: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    a11y_label: Option<Arc<str>>,
    options: DragValueCoreOptions,
}

impl<T> DragValueCore<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(
        value: T,
        on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static>,
    ) -> Self {
        Self {
            value,
            on_change_live,
            on_commit: None,
            on_cancel: None,
            a11y_label: None,
            options: DragValueCoreOptions::default(),
        }
    }

    pub fn on_commit(
        mut self,
        on_commit: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    ) -> Self {
        self.on_commit = on_commit;
        self
    }

    pub fn on_cancel(
        mut self,
        on_cancel: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    ) -> Self {
        self.on_cancel = on_cancel;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn options(mut self, options: DragValueCoreOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, DragValueCoreResponse) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let opts = resolve_options(theme, self.options);
        let state: Arc<Mutex<DragState<T>>> = cx.slot_state(
            || Arc::new(Mutex::new(DragState::<T>::default())),
            |s| s.clone(),
        );

        if !opts.enabled {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.clear_pointer_session();
            let _ = st.commit_session();
        }

        let on_change_live = self.on_change_live.clone();
        let on_commit = self.on_commit.clone();
        let on_cancel = self.on_cancel.clone();

        let enabled = opts.enabled;
        let a11y_label = self.a11y_label.clone();
        let value = self.value;

        let mut layout = opts.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.row_height));
        }

        cx.pressable(
            PressableProps {
                enabled,
                layout,
                a11y: PressableA11y {
                    label: a11y_label,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, pressable| {
                {
                    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                    st.current_value = value;
                }
                install_drag_value_core_behavior(
                    cx,
                    state.clone(),
                    opts,
                    on_change_live.clone(),
                    on_commit.clone(),
                    on_cancel.clone(),
                );

                let dragging = state.lock().unwrap_or_else(|e| e.into_inner()).dragging;
                children(
                    cx,
                    DragValueCoreResponse::new(
                        dragging,
                        pressable.hovered,
                        pressable.pressed,
                        pressable.focused,
                    ),
                )
            },
        )
    }
}
