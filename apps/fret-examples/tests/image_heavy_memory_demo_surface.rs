fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn image_heavy_memory_demo_uses_app_view_imports_with_explicit_gpu_hooks() {
    let source = include_str!("../src/image_heavy_memory_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "usefret::app::ElementContextAccess;",
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::view::ViewWindowState;",
        ".view_with_hooks::<ImageHeavyMemoryView>(|driver|{",
        "driver.record_engine_frame(record_engine_frame)",
        ".on_gpu_ready(upload_images)",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        "fnrecord_engine_frame(app:&mutApp,_window:WindowId,_ui:&mutfret_ui::UiTree<App>,_st:&mutViewWindowState<ImageHeavyMemoryView>,",
        "fnupload_images(app:&mutApp,context:&WgpuContext,renderer:&mutRenderer)",
        "Cx:ElementContextAccess<'a,App>,",
    ] {
        assert!(
            compact.contains(needle),
            "image_heavy_memory_demo should keep app view imports and explicit GPU hooks; missing `{needle}`",
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
    ] {
        assert!(
            !source.contains(forbidden),
            "image_heavy_memory_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}
