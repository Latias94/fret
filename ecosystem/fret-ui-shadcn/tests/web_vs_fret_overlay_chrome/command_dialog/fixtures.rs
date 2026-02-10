use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WebThemeName {
    Light,
    Dark,
}

impl WebThemeName {
    fn as_str(&self) -> &'static str {
        match self {
            WebThemeName::Light => "light",
            WebThemeName::Dark => "dark",
        }
    }

    fn scheme(&self) -> fret_ui_shadcn::shadcn_themes::ShadcnColorScheme {
        match self {
            WebThemeName::Light => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
            WebThemeName::Dark => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CommandDialogOverlayChromeRecipe {
    PanelChrome,
    FocusedItemChrome,
    HighlightedOptionChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct CommandDialogOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: CommandDialogOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_command_dialog_panel_chrome(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

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
}

#[test]
fn web_vs_fret_command_dialog_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_command_dialog_cases_v1.json"
    ));
    let suite: FixtureSuite<CommandDialogOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome command-dialog fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome command-dialog case={}", case.id);
        match case.recipe {
            CommandDialogOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_command_dialog_panel_chrome,
                );
            }
            CommandDialogOverlayChromeRecipe::FocusedItemChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("focused_item_chrome requires theme");
                assert_command_dialog_focused_item_chrome_matches_web_named(
                    &case.web_name,
                    theme.as_str(),
                );
            }
            CommandDialogOverlayChromeRecipe::HighlightedOptionChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("highlighted_option_chrome requires theme");
                assert_listbox_highlighted_option_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    "command-item",
                    theme.scheme(),
                    build_shadcn_command_dialog_page,
                );
            }
        }
    }
}
