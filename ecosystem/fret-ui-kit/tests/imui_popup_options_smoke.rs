#![cfg(feature = "imui")]

use fret_core::{Point, Px, Rect, Size};
use fret_ui::UiHost;
use fret_ui_kit::imui::{PopupMenuOptions, PopupModalOptions, ResponseExt, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn popup_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    ui.open_popup("tools");
    ui.open_popup_at(
        "tools",
        Rect::new(Point::new(Px(4.0), Px(8.0)), Size::new(Px(1.0), Px(1.0))),
    );
    ui.close_popup("tools");
    ui.drop_popup_scope("tools");

    ui.begin_popup_menu("tools/menu", None, |_| {});
    ui.begin_popup_menu_with_options(
        "tools/menu/options",
        None,
        PopupMenuOptions::default(),
        |_| {},
    );
    ui.begin_popup_context_menu("tools/context", ResponseExt::default(), |_| {});
    ui.begin_popup_context_menu_with_options(
        "tools/context/options",
        ResponseExt::default(),
        PopupMenuOptions::default(),
        |_| {},
    );
    ui.begin_popup_modal("tools/modal", None, |_| {});
    ui.begin_popup_modal_with_options(
        "tools/modal/options",
        None,
        PopupModalOptions::default(),
        |_| {},
    );
}

#[test]
fn popup_option_defaults_compile() {
    let menu = PopupMenuOptions::default();
    assert_eq!(menu.estimated_size, Size::new(Px(160.0), Px(120.0)));
    assert!(menu.modal);
    assert!(menu.auto_focus);

    let modal = PopupModalOptions::default();
    assert_eq!(modal.size, Size::new(Px(320.0), Px(200.0)));
    assert!(!modal.close_on_outside_press);
}
