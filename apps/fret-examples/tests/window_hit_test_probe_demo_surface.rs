fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn window_hit_test_probe_demo_keeps_fixed_text_on_roles() {
    let source = include_str!("../src/window_hit_test_probe_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnwindow_hit_test_title_text<H:UiHost>(",
        "fnwindow_hit_test_readout_text<H:UiHost>(",
        "fnwindow_hit_test_code_label_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text)",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_code_label(cx,text)",
        "window_hit_test_title_text(cx,\"Hit-testpassthroughprobe\",)",
        "window_hit_test_code_label_text(cx,format!(\"logical_window_id={logical}\"),)",
        "window_hit_test_readout_text(cx,status)",
    ] {
        assert!(
            source.contains(needle),
            "window hit-test probe should keep fixed chrome/readout text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(\"Hit-testpassthroughprobe\").font_semibold().text_sm()",
        "ui::text(format!(\"logical_window_id={logical}\")).font_monospace().text_sm()",
        "ui::text(status).text_sm()",
    ] {
        assert!(
            !source.contains(needle),
            "window hit-test probe should not render fixed chrome/readouts with local text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn window_hit_test_probe_demo_uses_explicit_manual_driver_imports() {
    let source = include_str!("../src/window_hit_test_probe_demo.rs");
    let source_compact = compact(source);

    for needle in [
        "usefret::advanced::KernelApp;",
        "usefret::advanced::interop::run_native_with_compat_driver;",
        "usefret_app::{CreateWindowKind,CreateWindowRequest,Effect,WindowRequest};",
        "usefret_bootstrap::ui_app_driver::{self,ViewElements};",
        "usefret_runtime::Model;",
        "usefret_ui_kit::IntoUiElementas_;",
        "usefret_ui_kit::declarative::{ElementContextThemeExtas_,UiElementTestIdExtas_};",
        "usefret_ui_kit::declarative::TrackedModelExtas_;",
        "ui_app_driver::UiAppDriver::new(\"window-hit-test-probe-demo\",init_window,view)",
        "run_native_with_compat_driver(config,KernelApp::new(),driver)?;",
    ] {
        assert!(
            source_compact.contains(needle),
            "window hit-test probe should keep manual driver capability imports explicit; missing `{needle}`",
        );
    }

    for forbidden in ["advanced::prelude::*", "component::prelude::*"] {
        assert!(
            !source.contains(forbidden),
            "window hit-test probe should not reintroduce broad prelude imports: `{forbidden}`",
        );
    }
}
