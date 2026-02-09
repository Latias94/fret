use super::*;

#[test]
fn web_vs_fret_layout_resizable_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_resizable_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutResizableCase> =
        serde_json::from_str(raw).expect("layout resizable fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout resizable case={} web_name={}",
            case.id, case.web_name
        );
        match case.recipe {
            LayoutResizableRecipe::Demo => {
                assert_resizable_demo_geometry_matches_web(&case.web_name)
            }
            LayoutResizableRecipe::DemoWithHandle => {
                assert_resizable_demo_with_handle_geometry_matches_web(&case.web_name)
            }
            LayoutResizableRecipe::Handle => {
                assert_resizable_handle_geometry_matches_web(&case.web_name)
            }
            LayoutResizableRecipe::Vertical => {
                assert_resizable_vertical_geometry_matches_web(&case.web_name)
            }
        }
    }
}
