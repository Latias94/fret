use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::{FrameId, Model};
use fret_ui::action::UiActionHost;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::headless::checked_state::CheckedState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControlId(Arc<str>);

impl ControlId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<Arc<str>> for ControlId {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl From<String> for ControlId {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}

impl From<&str> for ControlId {
    fn from(value: &str) -> Self {
        Self(Arc::from(value))
    }
}

#[derive(Debug, Clone)]
pub enum ControlAction {
    ToggleBool(Model<bool>),
    ToggleOptionalBool(Model<Option<bool>>),
    ToggleCheckedState(Model<CheckedState>),
    Noop,
}

impl ControlAction {
    pub fn invoke(&self, host: &mut dyn UiActionHost) {
        match self {
            ControlAction::ToggleBool(model) => {
                let _ = host.models_mut().update(model, |v: &mut bool| *v = !*v);
            }
            ControlAction::ToggleOptionalBool(model) => {
                let _ = host.models_mut().update(model, |v: &mut Option<bool>| {
                    *v = match *v {
                        None => Some(true),
                        Some(true) => Some(false),
                        Some(false) => Some(true),
                    };
                });
            }
            ControlAction::ToggleCheckedState(model) => {
                let _ = host
                    .models_mut()
                    .update(model, |v: &mut CheckedState| *v = v.toggle());
            }
            ControlAction::Noop => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlEntry {
    pub element: GlobalElementId,
    pub enabled: bool,
    pub action: ControlAction,
}

#[derive(Debug, Clone)]
pub struct LabelEntry {
    pub element: GlobalElementId,
}

#[derive(Debug, Clone)]
pub struct DescriptionEntry {
    pub element: GlobalElementId,
}

#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub element: GlobalElementId,
}

#[derive(Debug, Default, Clone)]
pub struct ControlRegistry {
    windows: HashMap<AppWindowId, WindowControlRegistry>,
}

#[derive(Debug, Default, Clone)]
struct WindowControlRegistry {
    frame_id: Option<FrameId>,
    controls: HashMap<ControlId, ControlEntry>,
    labels: HashMap<ControlId, LabelEntry>,
    descriptions: HashMap<ControlId, DescriptionEntry>,
    errors: HashMap<ControlId, ErrorEntry>,
}

impl ControlRegistry {
    fn begin_frame(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> &mut WindowControlRegistry {
        let entry = self.windows.entry(window).or_default();
        if entry.frame_id != Some(frame_id) {
            entry.frame_id = Some(frame_id);
            // Do not clear `controls`/`labels` on a new frame.
            //
            // Some app shells use view caching (GPUI-style reuse) where a subtree may be reused
            // without re-running the declarative builder for every child. Clearing the whole
            // registry would require *all* controls/labels to re-register on every frame, which
            // breaks label -> control forwarding for cached subtrees.
            //
            // Policy note: callers should treat `ControlId` as a stable, unique identifier within
            // a window. Reusing the same id for unrelated controls can lead to stale forwarding.
        }
        entry
    }

    pub fn register_control(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        id: ControlId,
        control: ControlEntry,
    ) {
        let st = self.begin_frame(window, frame_id);
        st.controls.insert(id, control);
    }

    pub fn register_label(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        id: ControlId,
        label: LabelEntry,
    ) {
        let st = self.begin_frame(window, frame_id);
        st.labels.insert(id, label);
    }

    pub fn register_description(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        id: ControlId,
        description: DescriptionEntry,
    ) {
        let st = self.begin_frame(window, frame_id);
        st.descriptions.insert(id, description);
    }

    pub fn register_error(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        id: ControlId,
        error: ErrorEntry,
    ) {
        let st = self.begin_frame(window, frame_id);
        st.errors.insert(id, error);
    }

    pub fn control_for(&self, window: AppWindowId, id: &ControlId) -> Option<&ControlEntry> {
        self.windows.get(&window)?.controls.get(id)
    }

    pub fn label_for(&self, window: AppWindowId, id: &ControlId) -> Option<&LabelEntry> {
        self.windows.get(&window)?.labels.get(id)
    }

    pub fn description_for(
        &self,
        window: AppWindowId,
        id: &ControlId,
    ) -> Option<&DescriptionEntry> {
        self.windows.get(&window)?.descriptions.get(id)
    }

    pub fn error_for(&self, window: AppWindowId, id: &ControlId) -> Option<&ErrorEntry> {
        self.windows.get(&window)?.errors.get(id)
    }

    pub fn described_by_for(&self, window: AppWindowId, id: &ControlId) -> Option<GlobalElementId> {
        let st = self.windows.get(&window)?;
        st.errors
            .get(id)
            .map(|e| e.element)
            .or_else(|| st.descriptions.get(id).map(|d| d.element))
    }
}

#[derive(Default)]
struct ControlRegistryService {
    model: Option<Model<ControlRegistry>>,
}

pub fn control_registry_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<ControlRegistry> {
    cx.app
        .with_global_mut(ControlRegistryService::default, |svc, app| {
            svc.model
                .get_or_insert_with(|| app.models_mut().insert(ControlRegistry::default()))
                .clone()
        })
}
