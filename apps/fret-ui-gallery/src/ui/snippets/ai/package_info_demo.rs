pub const SOURCE: &str = include_str!("package_info_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let react = ui_ai::PackageInfo::new("react")
        .current_version("18.2.0")
        .new_version("19.0.0")
        .change_type(ui_ai::PackageInfoChangeKind::Major)
        .into_element_with_children(cx, move |cx, _controller| {
            let header = ui_ai::PackageInfoHeader::new()
                .children([
                    ui_ai::PackageInfoName::new().into_element(cx),
                    ui_ai::PackageInfoChangeType::new()
                        .test_id("ui-ai-package-info-demo-badge-major")
                        .into_element(cx),
                ])
                .into_element(cx);

            let version = ui_ai::PackageInfoVersion::new()
                .test_id("ui-ai-package-info-demo-version-major")
                .into_element(cx);

            vec![
                header,
                version,
                ui_ai::PackageInfoDescription::new(
                    "A JavaScript library for building user interfaces.",
                )
                .into_element(cx),
                ui_ai::PackageInfoContent::new([ui_ai::PackageInfoDependencies::new([
                    ui_ai::PackageInfoDependency::new("react-dom")
                        .version("^19.0.0")
                        .into_element(cx),
                    ui_ai::PackageInfoDependency::new("scheduler")
                        .version("^0.24.0")
                        .into_element(cx),
                ])
                .into_element(cx)])
                .into_element(cx),
            ]
        });

    let lodash = ui_ai::PackageInfo::new("lodash")
        .change_type(ui_ai::PackageInfoChangeKind::Added)
        .into_element_with_children(cx, move |cx, _controller| {
            vec![
                ui_ai::PackageInfoHeader::new()
                    .children([
                        ui_ai::PackageInfoName::new().into_element(cx),
                        ui_ai::PackageInfoChangeType::new().into_element(cx),
                    ])
                    .into_element(cx),
            ]
        });

    let moment = ui_ai::PackageInfo::new("moment")
        .current_version("2.29.4")
        .change_type(ui_ai::PackageInfoChangeKind::Removed)
        .into_element(cx);

    ui::v_flex(move |_cx| vec![react, lodash, moment])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
