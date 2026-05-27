use std::hash::Hash;
use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableA11y, PressableKeyActivation, PressableProps,
    PressableState,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::imui::{ResponseExt, TableSortDirection, imui_is_disabled};

use super::{table_header_label_text, table_sort_indicator_text};
use crate::imui::table_controls::cell::table_cell_padding;

mod behavior;

pub(super) struct BuiltHeaderTrigger {
    pub(super) element: AnyElement,
    pub(super) trigger: ResponseExt,
}

pub(super) fn header_trigger_surface<H, K, F>(
    cx: &mut ElementContext<'_, H>,
    key: K,
    a11y_label: Option<Arc<str>>,
    activates_on_primary: bool,
    render: F,
) -> BuiltHeaderTrigger
where
    H: UiHost,
    K: Hash + Eq + Clone + 'static,
    F: Fn(&mut ElementContext<'_, H>, bool, PressableState) -> Vec<AnyElement> + 'static,
{
    let mut trigger = ResponseExt::default();
    let trigger_element = cx.keyed(key, |cx| {
        let trigger = &mut trigger;
        let enabled = !imui_is_disabled(cx);
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.layout.size.width = Length::Fill;
        props.layout.flex.grow = 1.0;
        props.layout.flex.shrink = 1.0;
        if !activates_on_primary {
            props.key_activation = PressableKeyActivation::None;
        }
        props.a11y = PressableA11y {
            role: Some(if activates_on_primary {
                SemanticsRole::Button
            } else {
                SemanticsRole::Group
            }),
            label: a11y_label.clone(),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, element_id| {
            behavior::install_header_trigger_behavior(
                cx,
                element_id,
                state,
                enabled,
                activates_on_primary,
                trigger,
            );

            render(cx, enabled, state)
        })
    });

    BuiltHeaderTrigger {
        element: trigger_element,
        trigger,
    }
}

pub(super) fn sortable_header_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    visible_label: Option<Arc<str>>,
    sort_direction: Option<TableSortDirection>,
    enabled: bool,
    state: PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let hover_bg = if enabled && (state.hovered || state.focused || state.pressed) {
        Some(
            theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_token("muted")),
        )
    } else {
        None
    };
    let mut cell = ContainerProps::default();
    cell.layout.size.width = Length::Fill;
    cell.layout.size.height = Length::Auto;
    cell.padding = table_cell_padding().into();
    cell.background = hover_bg;

    cx.container(cell, move |cx| {
        let mut children = Vec::new();
        if let Some(label) = visible_label.clone() {
            children.push(table_header_label_text(cx, label));
        }
        if let Some(direction) = sort_direction {
            children.push(table_sort_indicator_text(cx, direction));
        }
        if children.is_empty() {
            Vec::new()
        } else if children.len() == 1 {
            children
        } else {
            vec![
                crate::ui::h_flex(move |_cx| children)
                    .gap_metric(crate::MetricRef::space(crate::Space::N1))
                    .justify(crate::Justify::Start)
                    .items(crate::Items::Center)
                    .no_wrap()
                    .into_element(cx),
            ]
        }
    })
}
