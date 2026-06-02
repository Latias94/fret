//! InspectorPanel search field owner.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, MiniSearchBox, MiniSearchBoxOptions,
    TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface, TextFieldOptions,
};

use super::super::InspectorPanelSearchAssistOptions;

pub(super) fn inspector_panel_search_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    search: Model<String>,
    enabled: bool,
    search_test_id: Option<Arc<str>>,
    search_clear_test_id: Option<Arc<str>>,
    search_assist: Option<InspectorPanelSearchAssistOptions>,
) -> AnyElement {
    if let Some(search_assist) = search_assist {
        return TextAssistField::new(
            search,
            search_assist.dismissed_query_model,
            search_assist.active_item_id_model,
            search_assist.items,
        )
        .options(TextAssistFieldOptions {
            field: TextFieldOptions {
                enabled,
                focusable: enabled,
                placeholder: Some(Arc::from("Search…")),
                clear_button: true,
                buffered: false,
                selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
                cancel_behavior: EditorTextCancelBehavior::Clear,
                test_id: search_test_id,
                clear_test_id: search_clear_test_id,
                ..Default::default()
            },
            surface: TextAssistFieldSurface::AnchoredOverlay,
            list_label: search_assist.list_label,
            empty_label: search_assist.empty_label,
            key_options: search_assist.key_options,
            list_test_id: search_assist.list_test_id,
            item_test_id_prefix: search_assist.item_test_id_prefix,
            empty_test_id: search_assist.empty_test_id,
            max_list_height: search_assist.max_list_height,
        })
        .into_element(cx);
    }

    MiniSearchBox::new(search)
        .options(MiniSearchBoxOptions {
            enabled,
            focusable: enabled,
            test_id: search_test_id,
            clear_test_id: search_clear_test_id,
            ..Default::default()
        })
        .into_element(cx)
}
