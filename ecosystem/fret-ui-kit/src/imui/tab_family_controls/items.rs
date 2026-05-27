use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ColumnProps, LayoutStyle, Length, RowProps, SpacingLength};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::trigger;
use crate::imui::{TabBarOptions, TabBarResponse, TabTriggerResponse};
use crate::primitives::tabs;

pub(super) struct BuiltTabItem {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
    pub(super) enabled: bool,
    pub(super) default_selected: bool,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) panel_test_id: Option<Arc<str>>,
    pub(super) activate_shortcut: Option<fret_runtime::KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) panel_children: Vec<AnyElement>,
}

pub(super) fn render_tab_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: Model<Option<Arc<str>>>,
    items: Vec<BuiltTabItem>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TabBarOptions,
) -> (AnyElement, TabBarResponse) {
    let selected = normalize_selected_tab(cx, &selected_model, &items);
    let selected_changed =
        super::super::model_value_changed_for(cx, cx.root_id(), selected.clone());
    let set_size = items.len().min(u32::MAX as usize) as u32;
    let mut selected_trigger_id = None;
    let mut first_focusable = None;
    let mut trigger_responses = Vec::with_capacity(items.len());

    let triggers = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_selected = selected.as_deref() == Some(item.id.as_ref());
            let built = trigger::render_tab_trigger(
                cx,
                &selected_model,
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

    if let Some(state) = build_focus.as_ref()
        && state.get().is_none()
    {
        state.set(selected_trigger_id.or(first_focusable));
    }

    let list_layout = LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    };
    let list = cx.semantics(
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
                        .gap_metric(options.gap)
                        .justify(crate::Justify::Start)
                        .items(crate::Items::Center)
                        .no_wrap()
                        .into_element(cx),
                ]
            })]
        },
    );

    let panel = selected.clone().and_then(|selected_id| {
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
    });

    let mut children = vec![list];
    if let Some(panel) = panel {
        children.push(panel);
    }

    let mut column = ColumnProps::default();
    column.layout.size.width = Length::Fill;
    column.layout.size.height = Length::Auto;
    column.gap = SpacingLength::Px(Px(0.0));
    (
        cx.column(column, move |_cx| children),
        TabBarResponse {
            selected,
            selected_changed,
            triggers: trigger_responses,
        },
    )
}

fn normalize_selected_tab<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    items: &[BuiltTabItem],
) -> Option<Arc<str>> {
    let current = cx
        .read_model(
            selected_model,
            fret_ui::Invalidation::Paint,
            |_app, value| value.clone(),
        )
        .unwrap_or(None);
    let current_is_valid = current.as_ref().is_some_and(|selected_id| {
        items
            .iter()
            .any(|item| item.enabled && item.id.as_ref() == selected_id.as_ref())
    });
    if current_is_valid {
        return current;
    }

    let next = items
        .iter()
        .find(|item| item.enabled && item.default_selected)
        .or_else(|| items.iter().find(|item| item.enabled))
        .map(|item| item.id.clone());
    let _ = cx.app.models_mut().update(selected_model, |value| {
        *value = next.clone();
    });
    next
}
