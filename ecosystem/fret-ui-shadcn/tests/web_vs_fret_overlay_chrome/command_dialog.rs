use super::*;

#[test]
fn web_vs_fret_command_dialog_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_overlay_chrome_matches(
        "command-dialog",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}
#[test]
fn web_vs_fret_command_dialog_focused_item_chrome_matches_web() {
    assert_command_dialog_focused_item_chrome_matches_web("light");
}
#[test]
fn web_vs_fret_command_dialog_focused_item_chrome_matches_web_dark() {
    assert_command_dialog_focused_item_chrome_matches_web("dark");
}
#[test]
fn web_vs_fret_command_dialog_focused_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_command_dialog_focused_item_chrome_matches_web_named(
        "command-dialog.focus-first-vp375x240",
        "light",
    );
}
#[test]
fn web_vs_fret_command_dialog_focused_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_command_dialog_focused_item_chrome_matches_web_named(
        "command-dialog.focus-first-vp375x240",
        "dark",
    );
}
#[test]
fn web_vs_fret_command_dialog_highlighted_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "command-dialog.highlight-first-vp375x240",
        "light",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_command_dialog_page,
    );
}
#[test]
fn web_vs_fret_command_dialog_highlighted_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "command-dialog.highlight-first-vp375x240",
        "dark",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_command_dialog_page,
    );
}
