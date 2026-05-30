use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, RowProps, SpacingLength};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::BuiltTabItem;
use crate::imui::{TabBarOptions, TabTriggerResponse};
use crate::primitives::tabs;

pub(super) struct BuiltTabList {
    pub(super) element: AnyElement,
    pub(super) selected_trigger_id: Option<GlobalElementId>,
    pub(super) first_focusable: Option<GlobalElementId>,
    pub(super) trigger_responses: Vec<TabTriggerResponse>,
}

pub(super) fn render_tab_list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    selected: Option<&str>,
    items: &[BuiltTabItem],
    options: &TabBarOptions,
) -> BuiltTabList {
    let set_size = items.len().min(u32::MAX as usize) as u32;
    let mut selected_trigger_id = None;
    let mut first_focusable = None;
    let mut trigger_responses = Vec::with_capacity(items.len());

    let triggers = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_selected = selected == Some(item.id.as_ref());
            let built = super::super::trigger::render_tab_trigger(
                cx,
                selected_model,
                item,
                is_selected,
                index.min(u32::MAX as usize - 1) as u32 + 1,
                set_size,
            );
            if first_focusable.is_none() && item.enabled {
                first_focusable = built.response.id();
            }
            if is_selected {
                selected_trigger_id = built.response.id();
            }
            trigger_responses.push(TabTriggerResponse {
                id: item.id.clone(),
                selected: is_selected,
                trigger: built.response,
            });
            built.element
        })
        .collect::<Vec<_>>();

    let list_layout = LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    };
    let element = cx.semantics(
        {
            let mut props =
                tabs::tab_list_semantics_props(list_layout, tabs::TabsOrientation::Horizontal);
            props.test_id = options.test_id.clone();
            props
        },
        move |cx| {
            let mut row = RowProps::default();
            row.layout.size.width = Length::Fill;
            row.layout.size.height = Length::Auto;
            row.gap = SpacingLength::Px(Px(0.0));
            vec![cx.row(row, move |cx| {
                vec![
                    crate::ui::h_flex(move |_cx| triggers)
                        .gap_metric(options.gap.clone())
                        .justify(crate::Justify::Start)
                        .items(crate::Items::Center)
                        .no_wrap()
                        .into_element(cx),
                ]
            })]
        },
    );

    BuiltTabList {
        element,
        selected_trigger_id,
        first_focusable,
        trigger_responses,
    }
}
