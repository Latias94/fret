//! Immediate-mode tab-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ColumnProps, LayoutStyle, Length, RowProps, SpacingLength};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::containers::build_imui_children_with_focus;
use super::label_identity::parse_label_identity;
use super::{ImUiFacade, TabBarOptions, TabBarResponse, TabItemOptions, TabTriggerResponse};
use crate::primitives::tabs;

mod trigger;
mod visual;

struct BuiltTabItem {
    id: Arc<str>,
    label: Arc<str>,
    enabled: bool,
    default_selected: bool,
    test_id: Option<Arc<str>>,
    panel_test_id: Option<Arc<str>>,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
    panel_children: Vec<AnyElement>,
}

pub struct ImUiTabBar<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    items: &'cx mut Vec<BuiltTabItem>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

pub(super) fn tab_bar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TabBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
) -> (AnyElement, TabBarResponse) {
    let root_name = format!("fret-ui-kit.imui.tab_bar.{id}");
    cx.with_root_name(root_name.as_str(), |cx| {
        let selected = options
            .selected
            .clone()
            .unwrap_or_else(|| cx.local_model_keyed("selected", || None::<Arc<str>>));
        let mut items = Vec::new();

        {
            let mut tab_bar = ImUiTabBar {
                cx,
                items: &mut items,
                build_focus: build_focus.clone(),
            };
            f(&mut tab_bar);
        }

        render_tab_bar(cx, selected, items, build_focus, options)
    })
}

impl<'cx, 'a, H: UiHost> ImUiTabBar<'cx, 'a, H> {
    pub fn tab_item(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.tab_item_with_options(id, label, TabItemOptions::default(), f);
    }

    pub fn tab_item_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TabItemOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let id = Arc::<str>::from(id);
        let raw_label = label.into();
        let parts = parse_label_identity(raw_label.as_ref());
        let label = Arc::<str>::from(parts.visible);
        let test_id = options.test_id.clone();
        let panel_test_id = options.panel_test_id.or_else(|| {
            test_id
                .as_ref()
                .map(|test_id| Arc::from(format!("{test_id}.panel")))
        });
        let build_focus = self.build_focus.clone();
        let panel_children = self.cx.keyed(id.clone(), |cx| {
            let mut out = Vec::new();
            build_imui_children_with_focus(cx, &mut out, build_focus, f);
            out
        });
        self.items.push(BuiltTabItem {
            id,
            label,
            enabled: options.enabled,
            default_selected: options.default_selected,
            test_id,
            panel_test_id,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            panel_children,
        });
    }

    pub fn begin_tab_item(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.begin_tab_item_with_options(id, label, TabItemOptions::default(), f);
    }

    pub fn begin_tab_item_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TabItemOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.tab_item_with_options(id, label, options, f);
    }
}

fn render_tab_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: Model<Option<Arc<str>>>,
    items: Vec<BuiltTabItem>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TabBarOptions,
) -> (AnyElement, TabBarResponse) {
    let selected = normalize_selected_tab(cx, &selected_model, &items);
    let selected_changed = super::model_value_changed_for(cx, cx.root_id(), selected.clone());
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

#[cfg(test)]
mod tests;
