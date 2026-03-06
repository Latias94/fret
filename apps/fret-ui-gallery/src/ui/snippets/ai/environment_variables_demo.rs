pub const SOURCE: &str = include_str!("environment_variables_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

struct Variable {
    name: &'static str,
    value: &'static str,
    required: bool,
}

const VARIABLES: &[Variable] = &[
    Variable {
        name: "DATABASE_URL",
        required: true,
        value: "postgresql://localhost:5432/mydb",
    },
    Variable {
        name: "API_KEY",
        required: true,
        value: "sk-1234567890abcdef",
    },
    Variable {
        name: "NODE_ENV",
        required: false,
        value: "production",
    },
    Variable {
        name: "PORT",
        required: false,
        value: "3000",
    },
];

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let env = ui_ai::EnvironmentVariables::new()
        .default_show_values(false)
        .test_id_root("ui-ai-env-vars-root")
        .into_element_with_children(cx, move |cx, _controller| {
            let header = ui_ai::EnvironmentVariablesHeader::new([
                ui_ai::EnvironmentVariablesTitle::new().into_element(cx),
                ui_ai::EnvironmentVariablesToggle::new()
                    .test_id_switch("ui-ai-env-vars-toggle-switch")
                    .into_element(cx),
            ])
            .into_element(cx);

            let mut rows = Vec::with_capacity(VARIABLES.len());
            for var in VARIABLES {
                let name = var.name;
                let value = var.value;
                let required = var.required;

                let row = ui_ai::EnvironmentVariable::new(name, value).into_element_with_children(
                    cx,
                    move |cx| {
                        let mut copy = ui_ai::EnvironmentVariableCopyButton::new()
                            .copy_format(ui_ai::EnvironmentVariableCopyFormat::Export);
                        if name == "DATABASE_URL" {
                            copy = copy
                                .test_id("ui-ai-env-var-copy-export")
                                .copied_marker_test_id("ui-ai-env-var-copy-export-copied");
                        }
                        let copy = copy.into_element(cx);

                        let mut name_children =
                            vec![ui_ai::EnvironmentVariableName::new().into_element(cx)];
                        if required {
                            name_children
                                .push(ui_ai::EnvironmentVariableRequired::new().into_element(cx));
                        }
                        let name_group =
                            ui_ai::EnvironmentVariableGroup::new(name_children).into_element(cx);

                        let value_group = ui_ai::EnvironmentVariableGroup::new([
                            ui_ai::EnvironmentVariableValue::new().into_element(cx),
                            copy,
                        ])
                        .into_element(cx);

                        vec![name_group, value_group]
                    },
                );

                rows.push(row);
            }

            let content = ui_ai::EnvironmentVariablesContent::new(rows).into_element(cx);
            vec![header, content]
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
