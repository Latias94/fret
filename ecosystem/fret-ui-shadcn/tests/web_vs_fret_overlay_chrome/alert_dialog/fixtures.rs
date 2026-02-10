use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AlertDialogOverlayChromeRecipe {
    PanelChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct AlertDialogOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: AlertDialogOverlayChromeRecipe,
}

fn build_alert_dialog_demo(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    AlertDialog::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Show Dialog")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")]).into_element(cx),
    )
}

#[test]
fn web_vs_fret_alert_dialog_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_alert_dialog_cases_v1.json"
    ));
    let suite: FixtureSuite<AlertDialogOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome alert-dialog fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome alert-dialog case={}", case.id);
        match case.recipe {
            AlertDialogOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "alertdialog",
                    SemanticsRole::AlertDialog,
                    build_alert_dialog_demo,
                );
            }
        }
    }
}
