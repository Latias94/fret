fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn low_risk_function_driver_demos_use_explicit_advanced_imports() {
    for (name, source, expected_driver_import, expected_window_import) in [
        (
            "echarts_demo",
            include_str!("../src/echarts_demo.rs"),
            "usefret::advanced::driver::{ViewElements,ui_app_with_hooks};",
            "usefret_core::AppWindowId;",
        ),
        (
            "extras_marquee_perf_demo",
            include_str!("../src/extras_marquee_perf_demo.rs"),
            "usefret::advanced::driver::{ViewElements,ui_app_with_hooks};",
            "usefret_core::{AppWindowId,Px};",
        ),
        (
            "launcher_utility_window_demo",
            include_str!("../src/launcher_utility_window_demo.rs"),
            "usefret::advanced::driver::{UiAppDriver,ViewElements,ui_app_with_hooks};",
            "usefret_core::{AppWindowId,",
        ),
        (
            "launcher_utility_window_materials_demo",
            include_str!("../src/launcher_utility_window_materials_demo.rs"),
            "usefret::advanced::driver::{UiAppDriver,ViewElements,ui_app_with_hooks};",
            "usefret_core::{AppWindowId,Px};",
        ),
    ] {
        let compact = compact(source);
        assert!(
            compact.contains("usefret::advanced::KernelApp;"),
            "{name} should import the kernel app explicitly from the advanced surface",
        );
        assert!(
            compact.contains(expected_driver_import),
            "{name} should import function-driver helpers explicitly from the advanced driver surface; missing `{expected_driver_import}`",
        );
        assert!(
            compact.contains(expected_window_import),
            "{name} should import `AppWindowId` explicitly from `fret_core`; missing `{expected_window_import}`",
        );
        for forbidden in [
            "use fret::advanced::prelude",
            "advanced::prelude::*",
            "component::prelude::*",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} should not reintroduce broad prelude imports: `{forbidden}`",
            );
        }
    }

    for (name, source, expected_app_import) in [
        (
            "launcher_utility_window_demo",
            include_str!("../src/launcher_utility_window_demo.rs"),
            "usefret::app::{AppLocalStateExtas_,AppRenderDataExtas_,LocalState};",
        ),
        (
            "launcher_utility_window_materials_demo",
            include_str!("../src/launcher_utility_window_materials_demo.rs"),
            "usefret::app::{AppLocalStateExtas_,LocalState};",
        ),
    ] {
        let compact = compact(source);
        assert!(
            compact.contains(expected_app_import),
            "{name} should import `LocalState` from the app surface; missing `{expected_app_import}`",
        );
        assert!(
            !source.contains("advanced::prelude::LocalState")
                && !source.contains("advanced::prelude::{"),
            "{name} should not import app local state from the advanced prelude",
        );
        assert!(
            compact.contains("usefret::advanced::raw::LocalStateModelStoreExtas_;"),
            "{name} should keep the command-handler ModelStore bridge explicit while it still uses function-driver hooks",
        );
        assert!(
            !source.contains("LocalStateElementContextExt"),
            "{name} should use app-facing `LocalState::layout_value(...)` instead of the raw ElementContext bridge",
        );
    }
}

#[test]
fn baseline_memory_demos_use_app_view_surface() {
    for (name, source, view_name, window_name, size) in [
        (
            "empty_idle_demo",
            include_str!("../src/empty_idle_demo.rs"),
            "EmptyIdleView",
            "empty_idle_demo",
            "(520.0,240.0)",
        ),
        (
            "text_heavy_memory_demo",
            include_str!("../src/text_heavy_memory_demo.rs"),
            "TextHeavyMemoryView",
            "text_heavy_memory_demo",
            "(980.0,720.0)",
        ),
    ] {
        let compact = compact(source);
        let app_name = name.replace('_', "-");
        let needles = vec![
            "usefret::app::prelude::*;".to_string(),
            format!("struct{view_name}"),
            format!("FretApp::new(\"{app_name}\")"),
            format!(".window(\"{window_name}\",{size})"),
            ".setup(fret_bootstrap::install_default_i18n_backend)".to_string(),
            format!(".view::<{view_name}>()?"),
            "fninit(_app:&mutApp,_window:WindowId)->Self".to_string(),
            "fnrender(&mutself,".to_string(),
            ")->Ui".to_string(),
        ];

        for needle in needles {
            assert!(
                compact.contains(&needle),
                "{name} should use the app view surface; missing `{needle}`",
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
                "{name} should not reintroduce the old function-driver surface: `{forbidden}`",
            );
        }
    }
}

#[test]
fn app_state_demos_use_app_local_state_imports() {
    for (name, source, expected_import) in [
        (
            "datatable_demo",
            include_str!("../src/datatable_demo.rs"),
            "usefret::app::LocalState;",
        ),
        (
            "date_picker_demo",
            include_str!("../src/date_picker_demo.rs"),
            "usefret::app::{LocalState,TrackedStateExtas_};",
        ),
        (
            "emoji_conformance_demo",
            include_str!("../src/emoji_conformance_demo.rs"),
            "usefret::app::LocalState;",
        ),
        (
            "form_demo",
            include_str!("../src/form_demo.rs"),
            "usefret::app::{LocalState,TrackedStateExtas_};",
        ),
        (
            "ime_smoke_demo",
            include_str!("../src/ime_smoke_demo.rs"),
            "usefret::app::LocalState;",
        ),
        (
            "sonner_demo",
            include_str!("../src/sonner_demo.rs"),
            "usefret::app::LocalState;",
        ),
        (
            "table_demo",
            include_str!("../src/table_demo.rs"),
            "usefret::app::LocalState;",
        ),
        (
            "components_gallery",
            include_str!("../src/components_gallery.rs"),
            "usefret::app::TrackedStateExtas_;",
        ),
    ] {
        let compact = compact(source);
        assert!(
            compact.contains(expected_import),
            "{name} should import app state helpers from the app surface; missing `{expected_import}`",
        );

        for forbidden in [
            "usefret::advanced::prelude::LocalState;",
            "usefret::advanced::prelude::TrackedStateExtas_;",
            "usefret::advanced::prelude::{LocalState,TrackedStateExtas_};",
        ] {
            assert!(
                !compact.contains(forbidden),
                "{name} should not import app state helpers from the advanced prelude: `{forbidden}`",
            );
        }
    }
}

#[test]
fn hello_world_compare_demo_uses_explicit_public_surfaces() {
    let source = include_str!("../src/hello_world_compare_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "usefret::advanced::KernelApp;",
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::style::{Color,ColorRef,Radius,Space,TextAlign};",
    ] {
        assert!(
            compact.contains(needle),
            "hello_world_compare_demo should name the required app/advanced/style surfaces explicitly; missing `{needle}`",
        );
    }

    for forbidden in ["advanced::prelude::*", "component::prelude::*"] {
        assert!(
            !source.contains(forbidden),
            "hello_world_compare_demo should not reintroduce broad prelude imports: `{forbidden}`",
        );
    }
}
