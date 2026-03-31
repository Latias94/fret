#![cfg(feature = "imui")]

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui_kit::imui::{SelectableOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn selectable_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let _ = ui.selectable("selectable.basic", false);
    let _ = ui.selectable_with_options(
        "selectable.with_options",
        SelectableOptions {
            selected: true,
            a11y_role: Some(SemanticsRole::TreeItem),
            ..Default::default()
        },
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
