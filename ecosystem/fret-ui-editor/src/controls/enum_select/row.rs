use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::combobox as kit_combobox;

use crate::primitives::EditorDensity;
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_palette, editor_popup_list_row_radius,
};
use crate::primitives::readout::editor_popup_list_row_text_props;

use super::EnumSelectItem;

#[cfg(test)]
mod tests;

pub(super) fn enum_select_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    idx: usize,
    total: usize,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
    open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
    item: EnumSelectItem,
    density: EditorDensity,
    item_test_id_prefix: Option<Arc<str>>,
) -> (AnyElement, GlobalElementId, bool) {
    let selected = cx
        .get_model_cloned(&model, Invalidation::Paint)
        .unwrap_or(None)
        .as_deref()
        .is_some_and(|v| v == item.value.as_ref());
    let item_test_id = item_test_id_prefix.as_ref().map(|prefix| {
        Arc::<str>::from(format!(
            "{prefix}.item.{}",
            sanitize_test_id_segment(item.value.as_ref())
        ))
    });

    let value_for_activate = item.value.clone();
    let model_for_activate = model.clone();
    let open_for_activate = open.clone();
    let query_for_activate = query.clone();
    let open_change_reason_for_activate = open_change_reason.clone();

    let mut el = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::ListBoxOption),
                label: Some(item.label.clone()),
                test_id: item_test_id.clone(),
                selected,
                pos_in_set: Some((idx as u32) + 1),
                set_size: Some(total as u32),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(kit_combobox::commit_selection_on_activate(
                enum_select_selection_commit_policy(),
                model_for_activate.clone(),
                open_for_activate.clone(),
                query_for_activate.clone(),
                open_change_reason_for_activate.clone(),
                value_for_activate.clone(),
            ));

            let hovered = st.hovered || st.hovered_raw;
            let row_palette = {
                let theme = Theme::global(&*cx.app);
                editor_popup_list_row_palette(
                    theme,
                    hovered,
                    EditorPopupListRowState {
                        active: selected,
                        disabled: false,
                    },
                )
            };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::symmetric(density.padding_x, Px(0.0)).into(),
                    background: row_palette.bg,
                    corner_radii: Corners::all(editor_popup_list_row_radius()),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(editor_popup_list_row_text_props(
                        item.label.clone(),
                        row_palette.fg,
                        density.row_height,
                    ))]
                },
            )]
        },
    );

    if let Some(test_id) = item_test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }

    let el_id = el.id;
    (el, el_id, selected)
}

fn enum_select_selection_commit_policy() -> kit_combobox::SelectionCommitPolicy {
    kit_combobox::SelectionCommitPolicy {
        toggle_selected_to_none: false,
        close_on_commit: true,
        clear_query_on_commit: true,
    }
}

fn sanitize_test_id_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;

    for ch in raw.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        out.push_str("item");
    }

    out
}
