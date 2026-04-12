#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui_kit::imui::{ImUiMultiSelectState, SelectableOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn selectable_api_compiles<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    selection_model: &fret_runtime::Model<ImUiMultiSelectState<Arc<str>>>,
) {
    let items = [Arc::<str>::from("alpha"), Arc::<str>::from("beta")];

    let _ = ui.selectable("selectable.basic", false);
    let _ = ui.selectable_with_options(
        "selectable.with_options",
        SelectableOptions {
            selected: true,
            a11y_role: Some(SemanticsRole::TreeItem),
            ..Default::default()
        },
    );
    let _ = ui.multi_selectable(
        "selectable.multi",
        selection_model,
        &items,
        items[0].clone(),
    );
}

#[test]
fn selectable_option_defaults_compile() {
    let options = SelectableOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert!(!options.selected);
    assert!(options.close_popup.is_none());
    assert_eq!(options.a11y_role, Some(SemanticsRole::ListBoxOption));
}
