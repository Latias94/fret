use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutTypographyRecipe {
    TableCellGeometryLight,
    TableCellGeometryDark,
    H1GeometryLight,
    H2GeometryLight,
    H3GeometryLight,
    H4GeometryLight,
    PGeometryLight,
    LeadGeometryLight,
    MutedGeometryLight,
    LargeGeometryLight,
    BlockquoteGeometryLight,
    ListGeometryLight,
    InlineCodePaddingAndStyleLight,
    SmallTextStyleLight,
    DemoGeometrySmokeLight,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutTypographyCase {
    id: String,
    web_name: String,
    recipe: LayoutTypographyRecipe,
}

#[test]
fn web_vs_fret_layout_typography_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_typography_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutTypographyCase> =
        serde_json::from_str(raw).expect("layout typography fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout typography case={}", case.id);
        match case.recipe {
            LayoutTypographyRecipe::TableCellGeometryLight => {
                assert_eq!(case.web_name, "typography-table");
                web_vs_fret_layout_typography_table_cell_geometry_light();
            }
            LayoutTypographyRecipe::TableCellGeometryDark => {
                assert_eq!(case.web_name, "typography-table");
                web_vs_fret_layout_typography_table_cell_geometry_dark();
            }
            LayoutTypographyRecipe::H1GeometryLight => {
                assert_eq!(case.web_name, "typography-h1");
                web_vs_fret_layout_typography_h1_geometry_light();
            }
            LayoutTypographyRecipe::H2GeometryLight => {
                assert_eq!(case.web_name, "typography-h2");
                web_vs_fret_layout_typography_h2_geometry_light();
            }
            LayoutTypographyRecipe::H3GeometryLight => {
                assert_eq!(case.web_name, "typography-h3");
                web_vs_fret_layout_typography_h3_geometry_light();
            }
            LayoutTypographyRecipe::H4GeometryLight => {
                assert_eq!(case.web_name, "typography-h4");
                web_vs_fret_layout_typography_h4_geometry_light();
            }
            LayoutTypographyRecipe::PGeometryLight => {
                assert_eq!(case.web_name, "typography-p");
                web_vs_fret_layout_typography_p_geometry_light();
            }
            LayoutTypographyRecipe::LeadGeometryLight => {
                assert_eq!(case.web_name, "typography-lead");
                web_vs_fret_layout_typography_lead_geometry_light();
            }
            LayoutTypographyRecipe::MutedGeometryLight => {
                assert_eq!(case.web_name, "typography-muted");
                web_vs_fret_layout_typography_muted_geometry_light();
            }
            LayoutTypographyRecipe::LargeGeometryLight => {
                assert_eq!(case.web_name, "typography-large");
                web_vs_fret_layout_typography_large_geometry_light();
            }
            LayoutTypographyRecipe::BlockquoteGeometryLight => {
                assert_eq!(case.web_name, "typography-blockquote");
                web_vs_fret_layout_typography_blockquote_geometry_light();
            }
            LayoutTypographyRecipe::ListGeometryLight => {
                assert_eq!(case.web_name, "typography-list");
                web_vs_fret_layout_typography_list_geometry_light();
            }
            LayoutTypographyRecipe::InlineCodePaddingAndStyleLight => {
                assert_eq!(case.web_name, "typography-inline-code");
                web_vs_fret_layout_typography_inline_code_padding_and_style_light();
            }
            LayoutTypographyRecipe::SmallTextStyleLight => {
                assert_eq!(case.web_name, "typography-small-text");
                web_vs_fret_layout_typography_small_text_style_light();
            }
            LayoutTypographyRecipe::DemoGeometrySmokeLight => {
                assert_eq!(case.web_name, "typography-demo");
                web_vs_fret_layout_typography_demo_geometry_smoke_light();
            }
        }
    }
}
