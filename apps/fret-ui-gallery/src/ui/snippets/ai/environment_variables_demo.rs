pub const SOURCE: &str = include_str!("environment_variables_demo.rs");

// region: example
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let env = ui_ai::EnvironmentVariables::new()
        .test_id_root("ui-ai-env-vars-root")
        .into_element_with_children(cx, move |cx, controller| {
            let show_values = cx
                .get_model_copied(&controller.show_values, Invalidation::Layout)
                .unwrap_or(false);

            let marker = show_values
                .then(|| cx.text("").test_id("ui-ai-env-vars-show-values-true"))
                .unwrap_or_else(|| cx.text(""));

            let header = ui_ai::EnvironmentVariablesHeader::new([
                ui_ai::EnvironmentVariablesTitle::new()
                    .text("Environment variables")
                    .into_element(cx),
                ui_ai::EnvironmentVariablesToggle::new()
                    .test_id_switch("ui-ai-env-vars-toggle-switch")
                    .into_element(cx),
            ])
            .into_element(cx);

            let export_row = ui_ai::EnvironmentVariable::new("OPENAI_API_KEY", "sk-example")
                .into_element_with_children(cx, |cx| {
                    vec![
                        ui_ai::EnvironmentVariableGroup::new([
                            ui_ai::EnvironmentVariableName::new().into_element(cx),
                            ui_ai::EnvironmentVariableValue::new().into_element(cx),
                        ])
                        .into_element(cx),
                        ui_ai::EnvironmentVariableCopyButton::new()
                            .copy_format(ui_ai::EnvironmentVariableCopyFormat::Export)
                            .test_id("ui-ai-env-var-copy-export")
                            .copied_marker_test_id("ui-ai-env-var-copy-export-copied")
                            .into_element(cx),
                    ]
                });

            let content = ui_ai::EnvironmentVariablesContent::new([
                export_row,
                ui_ai::EnvironmentVariable::new("RUST_LOG", "info").into_element(cx),
                ui_ai::EnvironmentVariable::new("FRET_DIAG", "1").into_element(cx),
            ])
            .into_element(cx);

            vec![header, marker, content]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Environment Variables (AI Elements)"),
            cx.text("Toggle to reveal values; copy uses a clipboard effect."),
            env,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
