pub const SOURCE: &str = include_str!("package_info_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let package = ui_ai::PackageInfo::new("fret-ui-ai")
        .current_version("0.2.0")
        .new_version("0.3.0")
        .change_type(ui_ai::PackageInfoChangeKind::Minor)
        .into_element_with_children(cx, move |cx, controller| {
            let header = ui_ai::PackageInfoHeader::new()
                .children([
                    ui_ai::PackageInfoName::new().into_element(cx),
                    ui_ai::PackageInfoChangeType::new()
                        .test_id("ui-ai-package-info-demo-badge-minor")
                        .into_element(cx),
                ])
                .into_element(cx);

            let version = ui_ai::PackageInfoVersion::new()
                .test_id("ui-ai-package-info-demo-version-minor")
                .into_element(cx);

            vec![
                header,
                controller
                    .new_version
                    .is_some()
                    .then_some(version)
                    .unwrap_or_else(|| cx.text("")),
            ]
        });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("PackageInfo (AI Elements)"),
                cx.text("Version bump summary surface for dependency updates."),
                package,
            ]
        },
    )
}
// endregion: example
