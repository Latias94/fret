use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutScrollRecipe {
    ScrollAreaDemoRootSize,
    ScrollAreaDemoMaxOffsetY,
    ScrollAreaHorizontalDemoMaxOffset,
    ScrollAreaDemoScrollbarBoundsHover,
    ScrollAreaDemoThumbBackgroundHoverLight,
    ScrollAreaDemoThumbBackgroundHoverDark,
    ScrollAreaDemoScrollbarHidesAfterHoverOutDelay,
    ScrollAreaDemoThumbBoundsScrolled,
    ScrollAreaHorizontalDemoScrollbarBoundsHover,
    ScrollAreaHorizontalDemoThumbBackgroundHoverLight,
    ScrollAreaHorizontalDemoThumbBackgroundHoverDark,
    ScrollAreaHorizontalDemoScrollbarHidesAfterHoverOutDelay,
    ScrollAreaHorizontalDemoThumbBoundsScrolled,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutScrollCase {
    id: String,
    web_name: String,
    #[serde(default)]
    web_name_late: Option<String>,
    recipe: LayoutScrollRecipe,
}

#[test]
fn web_vs_fret_layout_scroll_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_scroll_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutScrollCase> =
        serde_json::from_str(raw).expect("layout scroll fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout scroll case={}", case.id);
        match case.recipe {
            LayoutScrollRecipe::ScrollAreaDemoRootSize => {
                assert_eq!(case.web_name, "scroll-area-demo");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_root_size();
            }
            LayoutScrollRecipe::ScrollAreaDemoMaxOffsetY => {
                assert_eq!(case.web_name, "scroll-area-demo");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_max_offset_y_matches_web();
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoMaxOffset => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_max_offset_matches_web();
            }
            LayoutScrollRecipe::ScrollAreaDemoScrollbarBoundsHover => {
                assert_eq!(case.web_name, "scroll-area-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_scrollbar_bounds_match_web_hover();
            }
            LayoutScrollRecipe::ScrollAreaDemoThumbBackgroundHoverLight => {
                assert_eq!(case.web_name, "scroll-area-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_light();
            }
            LayoutScrollRecipe::ScrollAreaDemoThumbBackgroundHoverDark => {
                assert_eq!(case.web_name, "scroll-area-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_dark();
            }
            LayoutScrollRecipe::ScrollAreaDemoScrollbarHidesAfterHoverOutDelay => {
                assert_eq!(case.web_name, "scroll-area-demo.hover-out-550ms");
                assert_eq!(
                    case.web_name_late.as_deref(),
                    Some("scroll-area-demo.hover-out-650ms")
                );
                web_vs_fret_layout_scroll_area_demo_scrollbar_hides_after_hover_out_delay();
            }
            LayoutScrollRecipe::ScrollAreaDemoThumbBoundsScrolled => {
                assert_eq!(case.web_name, "scroll-area-demo.scrolled");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_thumb_bounds_match_web_scrolled();
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoScrollbarBoundsHover => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_bounds_match_web_hover();
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoThumbBackgroundHoverLight => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_light(
                );
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoThumbBackgroundHoverDark => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_dark(
                );
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoScrollbarHidesAfterHoverOutDelay => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover-out-550ms");
                assert_eq!(
                    case.web_name_late.as_deref(),
                    Some("scroll-area-horizontal-demo.hover-out-650ms")
                );
                web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_hides_after_hover_out_delay(
                );
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoThumbBoundsScrolled => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.scrolled");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_thumb_bounds_match_web_scrolled();
            }
        }
    }
}
