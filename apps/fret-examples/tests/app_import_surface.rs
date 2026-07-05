fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn low_risk_function_driver_demos_use_explicit_advanced_imports() {
    for (name, source, expected_driver_import) in [
        (
            "echarts_demo",
            include_str!("../src/echarts_demo.rs"),
            "ui_app_with_hooks",
        ),
        (
            "extras_marquee_perf_demo",
            include_str!("../src/extras_marquee_perf_demo.rs"),
            "ui_app_with_hooks",
        ),
        (
            "launcher_utility_window_demo",
            include_str!("../src/launcher_utility_window_demo.rs"),
            "ui_app_with_hooks",
        ),
        (
            "launcher_utility_window_materials_demo",
            include_str!("../src/launcher_utility_window_materials_demo.rs"),
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

#[test]
fn empty_idle_demo_uses_app_view_surface() {
    let source = include_str!("../src/empty_idle_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "structEmptyIdleView;",
        "FretApp::new(\"empty-idle-demo\")",
        ".window(\"empty_idle_demo\",(520.0,240.0))",
        ".setup(fret_bootstrap::install_default_i18n_backend)",
        ".view::<EmptyIdleView>()?",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        "fnrender(&mutself,_cx:&mutAppUi<'_,'_>)->Ui",
        "Vec::new().into()",
    ] {
        assert!(
            compact.contains(needle),
            "empty_idle_demo should use the app view surface; missing `{needle}`",
        );
    }

    for forbidden in [
        "advanced::prelude",
        "component::prelude",
        "ui_app",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
        "ElementContext<'_,KernelApp>",
    ] {
        assert!(
            !source.contains(forbidden),
            "empty_idle_demo should not reintroduce the old function-driver surface: `{forbidden}`",
        );
    }
}
