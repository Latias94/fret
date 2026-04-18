pub const SOURCE: &str = include_str!("environment_variables_custom_children.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let env = ui_ai::EnvironmentVariables::new()
        .default_show_values(false)
        .into_element_with_children(cx, move |cx, _controller| {
            let header = ui_ai::EnvironmentVariablesHeader::new([
                ui_ai::EnvironmentVariablesTitle::new_children([cx.text("Runtime Secrets")])
                    .into_element(cx),
                ui_ai::EnvironmentVariablesToggle::new().into_element(cx),
            ])
            .into_element(cx);

            let row = ui_ai::EnvironmentVariable::new("OPENAI_API_KEY", "sk-live-1234567890")
                .into_element_with_children(cx, move |cx| {
                    vec![
                        ui_ai::EnvironmentVariableGroup::new([
                            ui_ai::EnvironmentVariableName::new()
                                .children([cx.text("Primary API Key")])
                                .into_element(cx),
                            ui_ai::EnvironmentVariableRequired::new()
                                .children([cx.text("Secret")])
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        ui_ai::EnvironmentVariableGroup::new([
                            ui_ai::EnvironmentVariableValue::new()
                                .children([cx.text("App-owned masked preview")])
                                .into_element(cx),
                            ui_ai::EnvironmentVariableCopyButton::new()
                                .copy_format(ui_ai::EnvironmentVariableCopyFormat::Export)
                                .children([shadcn::raw::typography::muted("E").into_element(cx)])
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ]
                });

            let content = ui_ai::EnvironmentVariablesContent::new([row]).into_element(cx);
            vec![header, content]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Custom children take ownership of the visible content."),
            env,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
