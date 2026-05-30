use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::super::super::{imui_is_disabled, model_value_changed_for};
use super::super::spec::DisclosureSpec;
use crate::declarative::ModelWatchExt;
use crate::primitives::collapsible as radix_collapsible;

pub(super) struct DisclosureEntryState {
    pub(super) open_model: Model<bool>,
    pub(super) open_now: bool,
    pub(super) toggled: bool,
    pub(super) enabled: bool,
}

pub(super) fn prepare_disclosure_entry_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
) -> DisclosureEntryState {
    let root = radix_collapsible::CollapsibleRoot::new()
        .open(spec.open.clone())
        .default_open(spec.default_open);
    let open_model = root.use_open_model(cx).model();
    let open_now = if spec.has_children() {
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    } else {
        false
    };
    let toggled = model_value_changed_for(cx, cx.root_id(), open_now);
    let enabled = spec.enabled && !imui_is_disabled(cx);

    DisclosureEntryState {
        open_model,
        open_now,
        toggled,
        enabled,
    }
}
