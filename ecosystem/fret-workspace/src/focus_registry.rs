use std::collections::{HashMap, HashSet};
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

    pub(crate) fn pane_elements_for_window(
        &self,
        window: AppWindowId,
    ) -> Vec<(Arc<str>, GlobalElementId)> {
        self.entries
            .iter()
            .filter_map(|(key, element)| {
                (key.window == window)
                    .then(|| key.pane_id.clone().map(|pane_id| (pane_id, *element)))
                    .flatten()
            })
            .collect()
    }

    pub(crate) fn needs_workspace_reconciliation(
        &self,
        window: AppWindowId,
        live_tab_ids_by_pane: &HashMap<Arc<str>, HashSet<Arc<str>>>,
    ) -> bool {
        self.entries.keys().any(|key| match &key.pane_id {
            Some(pane_id) if key.window == window => live_tab_ids_by_pane
                .get(pane_id)
                .is_none_or(|tab_ids| !tab_ids.contains(&key.tab_id)),
            _ => false,
        })
    }

    pub(crate) fn reconcile_workspace_tabs_for_window(
        &mut self,
        window: AppWindowId,
        live_tab_ids_by_pane: &HashMap<Arc<str>, HashSet<Arc<str>>>,
    ) -> bool {
        let previous_len = self.entries.len();
        self.entries.retain(|key, _| match &key.pane_id {
            Some(pane_id) if key.window == window => live_tab_ids_by_pane
                .get(pane_id)
                .is_some_and(|tab_ids| tab_ids.contains(&key.tab_id)),
            _ => true,
        });
        self.entries.len() != previous_len
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_elements_for_window_returns_only_pane_scoped_tab_roots() {
        let window = AppWindowId::default();
        let pane_element = GlobalElementId(11);
        let unscoped_element = GlobalElementId(12);
        let pane_id = Arc::<str>::from("pane-a");
        let mut registry = WorkspaceTabElementRegistry::default();

        registry.set_if_changed(
            WorkspaceTabElementKey {
                window,
                pane_id: Some(pane_id.clone()),
                tab_id: Arc::from("alpha"),
            },
            pane_element,
        );
        registry.set_if_changed(
            WorkspaceTabElementKey {
                window,
                pane_id: None,
                tab_id: Arc::from("standalone"),
            },
            unscoped_element,
        );

        assert_eq!(
            registry.pane_elements_for_window(window),
            vec![(pane_id, pane_element)]
        );
    }

    #[test]
    fn workspace_reconciliation_removes_stale_tabs_and_removed_panes() {
        let window = AppWindowId::default();
        let pane_a = Arc::<str>::from("pane-a");
        let pane_b = Arc::<str>::from("pane-b");
        let alpha_key = WorkspaceTabElementKey {
            window,
            pane_id: Some(pane_a.clone()),
            tab_id: Arc::from("alpha"),
        };
        let stale_key = WorkspaceTabElementKey {
            window,
            pane_id: Some(pane_a.clone()),
            tab_id: Arc::from("stale"),
        };
        let other_pane_key = WorkspaceTabElementKey {
            window,
            pane_id: Some(pane_b),
            tab_id: Arc::from("other"),
        };
        let unscoped_key = WorkspaceTabElementKey {
            window,
            pane_id: None,
            tab_id: Arc::from("standalone"),
        };
        let mut registry = WorkspaceTabElementRegistry::default();
        registry.set_if_changed(alpha_key.clone(), GlobalElementId(21));
        registry.set_if_changed(stale_key.clone(), GlobalElementId(22));
        registry.set_if_changed(other_pane_key.clone(), GlobalElementId(23));
        registry.set_if_changed(unscoped_key.clone(), GlobalElementId(24));
        let live_tab_ids_by_pane =
            HashMap::from([(pane_a, HashSet::from([Arc::<str>::from("alpha")]))]);

        assert!(registry.needs_workspace_reconciliation(window, &live_tab_ids_by_pane));
        assert!(registry.reconcile_workspace_tabs_for_window(window, &live_tab_ids_by_pane));
        assert!(registry.get(&alpha_key).is_some());
        assert_eq!(registry.get(&stale_key), None);
        assert_eq!(
            registry.get(&other_pane_key),
            None,
            "entries for panes removed from the workspace model must not survive reconciliation"
        );
        assert!(registry.get(&unscoped_key).is_some());
        assert!(!registry.needs_workspace_reconciliation(window, &live_tab_ids_by_pane));
        assert!(
            !registry.reconcile_workspace_tabs_for_window(window, &live_tab_ids_by_pane),
            "reconciling an already-current registry should be a no-op"
        );
    }
}
