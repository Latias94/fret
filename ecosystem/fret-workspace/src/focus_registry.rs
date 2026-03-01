use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WorkspaceTabElementKey {
    pub(crate) window: AppWindowId,
    pub(crate) pane_id: Option<Arc<str>>,
    pub(crate) tab_id: Arc<str>,
}

#[derive(Debug, Default)]
pub(crate) struct WorkspaceTabElementRegistry {
    entries: HashMap<WorkspaceTabElementKey, GlobalElementId>,
}

impl WorkspaceTabElementRegistry {
    pub(crate) fn get(&self, key: &WorkspaceTabElementKey) -> Option<GlobalElementId> {
        self.entries.get(key).copied()
    }

    pub(crate) fn contains_element_for_window(
        &self,
        window: AppWindowId,
        element: GlobalElementId,
    ) -> bool {
        self.entries
            .iter()
            .any(|(key, value)| key.window == window && *value == element)
    }

    pub(crate) fn set_if_changed(
        &mut self,
        key: WorkspaceTabElementKey,
        element: GlobalElementId,
    ) -> bool {
        if self.entries.get(&key).copied() == Some(element) {
            return false;
        }
        self.entries.insert(key, element);
        true
    }

    pub(crate) fn remove(&mut self, key: &WorkspaceTabElementKey) -> bool {
        self.entries.remove(key).is_some()
    }
}

#[derive(Default)]
struct WorkspaceTabElementRegistryGlobal {
    model: Option<Model<WorkspaceTabElementRegistry>>,
}

pub(crate) fn workspace_tab_element_registry_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<WorkspaceTabElementRegistry> {
    cx.app
        .with_global_mut_untracked(WorkspaceTabElementRegistryGlobal::default, |global, app| {
            if let Some(model) = global.model.clone() {
                return model;
            }
            let model = app
                .models_mut()
                .insert(WorkspaceTabElementRegistry::default());
            global.model = Some(model.clone());
            model
        })
}
