//! Text-assist suggestion option row owner.

use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};
use fret_ui_kit::headless::text_assist::TextAssistMatch;

use super::super::{OnTextAssistFieldAccept, accept::accept_text_assist_match};
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_palette, editor_popup_list_row_radius,
};
use crate::primitives::readout::editor_popup_list_row_text_props;

pub(super) struct TextAssistOptionRowInput {
    pub(super) index: usize,
    pub(super) total: usize,
    pub(super) entry: TextAssistMatch,
    pub(super) is_active: bool,
    pub(super) item_test_id_prefix: Option<Arc<str>>,
    pub(super) row_height: Px,
    pub(super) padding_x: Px,
    pub(super) query_model: Model<String>,
    pub(super) dismissed_query_model: Model<String>,
    pub(super) active_item_id_model: Model<Option<Arc<str>>>,
    pub(super) on_accept: Option<OnTextAssistFieldAccept>,
}

pub(super) fn text_assist_option_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: TextAssistOptionRowInput,
) -> (AnyElement, GlobalElementId) {
    let TextAssistOptionRowInput {
        index,
        total,
        entry,
        is_active,
        item_test_id_prefix,
        row_height,
        padding_x,
        query_model,
        dismissed_query_model,
        active_item_id_model,
        on_accept,
    } = input;

    let option_test_id = item_test_id_prefix
        .as_ref()
        .map(|prefix| Arc::<str>::from(format!("{prefix}.item.{}", entry.item_id)));
    let label = entry.label.clone();
    let disabled = entry.disabled;
    let active = entry.clone();

    let row = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: !disabled,
            focusable: false,
            a11y: PressableA11y {
                role: Some(SemanticsRole::ListBoxOption),
                label: Some(label.clone()),
                test_id: option_test_id,
                selected: is_active,
                pos_in_set: Some((index as u32) + 1),
                set_size: Some(total as u32),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let query_model = query_model.clone();
            let dismissed_query_model = dismissed_query_model.clone();
            let active_item_id_model = active_item_id_model.clone();
            let active = active.clone();
            let on_accept = on_accept.clone();
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx, _reason: ActivateReason| {
                    accept_text_assist_match(
                        host,
                        action_cx,
                        &query_model,
                        &dismissed_query_model,
                        &active_item_id_model,
                        active.clone(),
                        on_accept.as_ref(),
                    );
                });
            cx.pressable_add_on_activate(on_activate);

            let hovered = st.hovered || st.hovered_raw;
            let row_palette = {
                let theme = Theme::global(&*cx.app);
                editor_popup_list_row_palette(
                    theme,
                    hovered,
                    EditorPopupListRowState {
                        active: is_active,
                        disabled,
                    },
                )
            };
            let label = label.clone();

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
                    padding: Edges::symmetric(padding_x, Px(0.0)).into(),
                    background: row_palette.bg,
                    corner_radii: Corners::all(editor_popup_list_row_radius()),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(editor_popup_list_row_text_props(
                        label.clone(),
                        row_palette.fg,
                        row_height,
                    ))]
                },
            )]
        },
    );

    let row_id = row.id;
    (row, row_id)
}
