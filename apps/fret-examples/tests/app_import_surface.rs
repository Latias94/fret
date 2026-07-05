fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn low_risk_function_driver_demos_use_explicit_advanced_imports() {
    for (name, source, expected_driver_import) in [
        (
            "empty_idle_demo",
            include_str!("../src/empty_idle_demo.rs"),
            "ui_app",
        ),
        (
            "echarts_demo",
            include_str!("../src/echarts_demo.rs"),
            "ui_app_with_hooks",
        ),
        (
            "text_heavy_memory_demo",
            include_str!("../src/text_heavy_memory_demo.rs"),
            "ui_app",
        ),
    ] {
        let compact = compact(source);
        assert!(
            compact.contains("usefret::advanced::prelude::{"),
            "{name} should keep function-driver imports explicit while it still uses the advanced `ui_app` surface",
        );
        for symbol in [
            "AppWindowId",
            "KernelApp",
            "ViewElements",
            expected_driver_import,
        ] {
            assert!(
                compact.contains(symbol),
                "{name} should import `{symbol}` explicitly from the advanced prelude",
            );
        }
        for forbidden in ["advanced::prelude::*", "component::prelude::*"] {
            assert!(
                !source.contains(forbidden),
                "{name} should not reintroduce broad prelude imports: `{forbidden}`",
            );
        }
    }
}
