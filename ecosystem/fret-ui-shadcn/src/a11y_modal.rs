use std::collections::HashMap;

use fret_runtime::ModelId;
use fret_ui::UiHost;
use fret_ui::elements::GlobalElementId;

#[derive(Default)]
pub(crate) struct ModalA11yRegistry {
    stack: Vec<ModelId>,
    title_by_modal: HashMap<ModelId, GlobalElementId>,
    description_by_modal: HashMap<ModelId, GlobalElementId>,
    content_max_width_by_modal: HashMap<ModelId, fret_core::Px>,
}

pub(crate) fn begin_modal_a11y_scope<H: UiHost>(app: &mut H, modal: ModelId) {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        reg.stack.push(modal);
        reg.title_by_modal.remove(&modal);
        reg.description_by_modal.remove(&modal);
        // Keep the last committed content width across frames so header/footer recipe nodes that
        // are built before `*Content::into_element()` can still infer size on the next frame.
        // Unlike title/description ids, this value is not tied to per-frame element identity.
    });
}

pub(crate) fn end_modal_a11y_scope<H: UiHost>(app: &mut H, expected: ModelId) {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        let popped = reg.stack.pop();
        if cfg!(debug_assertions) {
            assert_eq!(popped, Some(expected), "modal a11y scope stack mismatch");
        } else if popped != Some(expected) {
            reg.stack.clear();
        }
    });
}

pub(crate) fn register_modal_title<H: UiHost>(app: &mut H, element: GlobalElementId) {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        let Some(active) = reg.stack.last().copied() else {
            return;
        };
        reg.title_by_modal.insert(active, element);
    });
}

pub(crate) fn register_modal_description<H: UiHost>(app: &mut H, element: GlobalElementId) {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        let Some(active) = reg.stack.last().copied() else {
            return;
        };
        reg.description_by_modal.insert(active, element);
    });
}

pub(crate) fn register_modal_content_max_width<H: UiHost>(app: &mut H, max_width: fret_core::Px) {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        let Some(active) = reg.stack.last().copied() else {
            return;
        };
        reg.content_max_width_by_modal.insert(active, max_width);
    });
}

pub(crate) fn modal_content_max_width_for_current_scope<H: UiHost>(
    app: &mut H,
) -> Option<fret_core::Px> {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        let Some(active) = reg.stack.last().copied() else {
            return None;
        };
        reg.content_max_width_by_modal.get(&active).copied()
    })
}

pub(crate) fn modal_relations_for_current_scope<H: UiHost>(
    app: &mut H,
) -> (Option<u64>, Option<u64>) {
    app.with_global_mut_untracked(ModalA11yRegistry::default, |reg, _app| {
        let Some(active) = reg.stack.last().copied() else {
            return (None, None);
        };
        let labelled_by = reg.title_by_modal.get(&active).map(|id| id.0);
        let described_by = reg.description_by_modal.get(&active).map(|id| id.0);
        (labelled_by, described_by)
    })
}
