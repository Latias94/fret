use std::sync::Arc;

use fret_ui::element::{AnyElement, LayoutStyle, Length};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::BuiltTabItem;
use crate::primitives::tabs;

pub(super) fn render_selected_tab_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected: Option<Arc<str>>,
    selected_trigger_id: Option<GlobalElementId>,
    items: Vec<BuiltTabItem>,
) -> Option<AnyElement> {
    selected.and_then(|selected_id| {
        items
            .into_iter()
            .find(|item| item.id.as_ref() == selected_id.as_ref())
            .map(|item| {
                let panel_layout = LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                cx.keyed(("tab-panel", item.id.clone()), |cx| {
                    let mut semantics = tabs::tab_panel_semantics_props(
                        panel_layout,
                        Some(item.label),
                        selected_trigger_id.map(|id| id.0),
                    );
                    semantics.test_id = item.panel_test_id;
                    cx.semantics(semantics, move |_cx| item.panel_children)
                })
            })
    })
}
