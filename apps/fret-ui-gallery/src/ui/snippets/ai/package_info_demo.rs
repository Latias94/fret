pub const SOURCE: &str = include_str!("package_info_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let react = ui_ai::PackageInfo::new("react")
        .current_version("18.2.0")
        .new_version("19.0.0")
        .change_type(ui_ai::PackageInfoChangeKind::Major)
        .header(ui_ai::PackageInfoHeader::new().children(vec![
            ui_ai::PackageInfoHeaderChild::Name(ui_ai::PackageInfoName::new()),
            ui_ai::PackageInfoHeaderChild::ChangeType(
                ui_ai::PackageInfoChangeType::new().test_id("ui-ai-package-info-demo-badge-major"),
            ),
        ]))
        .version(ui_ai::PackageInfoVersion::new().test_id("ui-ai-package-info-demo-version-major"))
        .description(ui_ai::PackageInfoDescription::new(
            "A JavaScript library for building user interfaces.",
        ))
        .content(ui_ai::PackageInfoContent::new(vec![
            ui_ai::PackageInfoContentChild::Dependencies(ui_ai::PackageInfoDependencies::new(
                vec![
                    ui_ai::PackageInfoDependency::new("react-dom").version("^19.0.0"),
                    ui_ai::PackageInfoDependency::new("scheduler").version("^0.24.0"),
                ],
            )),
        ]))
        .into_element(cx);

    let lodash = ui_ai::PackageInfo::new("lodash")
        .change_type(ui_ai::PackageInfoChangeKind::Added)
        .header(ui_ai::PackageInfoHeader::new().children(vec![
            ui_ai::PackageInfoHeaderChild::Name(ui_ai::PackageInfoName::new()),
            ui_ai::PackageInfoHeaderChild::ChangeType(ui_ai::PackageInfoChangeType::new()),
        ]))
        .into_element(cx);

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

// region: custom_children
pub fn render_custom_children(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui_ai::PackageInfo::new("react")
        .current_version("18.2.0")
        .new_version("19.0.0")
        .change_type(ui_ai::PackageInfoChangeKind::Major)
        .header(ui_ai::PackageInfoHeader::new().children(vec![
            ui_ai::PackageInfoHeaderChild::Name(
                ui_ai::PackageInfoName::new().children([decl_text::text_code_label(
                    cx,
                    "pkg/react",
                )]),
            ),
            ui_ai::PackageInfoHeaderChild::ChangeType(
                ui_ai::PackageInfoChangeType::new()
                    .children([decl_text::text_button_label(cx, "Breaking")]),
            ),
        ]))
        .version(
            ui_ai::PackageInfoVersion::new()
                .children([decl_text::text_code_label(cx, "18.2.0 -> 19.0.0 (custom)")]),
        )
        .description(ui_ai::PackageInfoDescription::new("ignored").children([
            decl_text::text_paragraph(cx, "Custom summary supplied by the app."),
        ]))
        .content(ui_ai::PackageInfoContent::new(vec![
            ui_ai::PackageInfoContentChild::Dependencies(ui_ai::PackageInfoDependencies::new(
                vec![
                    ui_ai::PackageInfoDependency::new("react-dom")
                        .children([decl_text::text_code_label(cx, "react-dom @ ^19.0.0")]),
                ],
            )),
        ]))
        .into_element(cx)
}
// endregion: custom_children
