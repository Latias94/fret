use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WorkspacePaneContentElementKey {
    pub(crate) window: AppWindowId,
    pub(crate) pane_id: Arc<str>,
}

#[derive(Debug, Default)]
pub(crate) struct WorkspacePaneContentElementRegistry {
    entries: HashMap<WorkspacePaneContentElementKey, GlobalElementId>,
}

impl WorkspacePaneContentElementRegistry {
    pub(crate) fn get(&self, key: &WorkspacePaneContentElementKey) -> Option<GlobalElementId> {
        self.entries.get(key).copied()
    }

    pub(crate) fn set_if_changed(
        &mut self,
        key: WorkspacePaneContentElementKey,
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
struct WorkspacePaneContentElementRegistryGlobal {
    model: Option<Model<WorkspacePaneContentElementRegistry>>,
}

pub(crate) fn workspace_pane_content_element_registry_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<WorkspacePaneContentElementRegistry> {
    cx.app.with_global_mut_untracked(
        WorkspacePaneContentElementRegistryGlobal::default,
        |global, app| {
            if let Some(model) = global.model.clone() {
                return model;
            }
            let model = app
                .models_mut()
                .insert(WorkspacePaneContentElementRegistry::default());
            global.model = Some(model.clone());
            model
        },
    )
}

/// Register a focus target that represents “pane content” for `workspace.pane.focus_content`.
///
/// This is a best-effort policy seam for editor shells:
/// - It allows `focus_content` / `Ctrl+F6` toggle to exit the tab strip even when no “return focus”
///   target was recorded (e.g. the tab strip became focused via pointer interaction).
/// - The registered element must be focusable for `request_focus` to succeed.
#[derive(Debug)]
pub struct WorkspacePaneContentFocusTarget<T = AnyElement> {
    pane_id: Arc<str>,
    child: T,
}

impl<T> WorkspacePaneContentFocusTarget<T> {
    pub fn new(pane_id: impl Into<Arc<str>>, child: T) -> Self {
        Self {
            pane_id: pane_id.into(),
            child,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        T: IntoUiElement<H>,
    {
        let registry = workspace_pane_content_element_registry_model(cx);
        let child = self.child.into_element(cx);
        let key = WorkspacePaneContentElementKey {
            window: cx.window,
            pane_id: self.pane_id,
        };
        let id = child.id;

        let _ = cx
            .app
            .models_mut()
            .update(&registry, |reg| reg.set_if_changed(key, id));

        child
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspacePaneContentFocusTarget;
    use fret_app::App;
    use fret_ui::ElementContext;
    use fret_ui::element::AnyElement;
    use fret_ui_kit::ui;

    #[allow(dead_code)]
    fn workspace_pane_content_focus_target_accepts_typed_children(
        cx: &mut ElementContext<'_, App>,
    ) -> AnyElement {
        WorkspacePaneContentFocusTarget::new("pane", ui::text("content")).into_element(cx)
    }
}
