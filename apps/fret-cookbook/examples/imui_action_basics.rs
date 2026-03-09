use std::sync::Arc;

use fret::prelude::*;
use fret_genui_core::catalog::CatalogActionV1;
use fret_genui_core::render::{GenUiRuntime, RenderLimits, render_spec};
use fret_genui_core::spec::SpecV1;
use fret_genui_core::validate::ValidationMode;
use fret_genui_shadcn::catalog::shadcn_catalog_v1;
use fret_genui_shadcn::resolver::ShadcnResolver;
use fret_runtime::{CommandId, CommandMeta, CommandScope};
use fret_ui::CommandAvailability;
use fret_ui_kit::imui::ButtonOptions;
use serde_json::{Value, json};

mod act {
    fret::actions!([Inc = "cookbook.imui_action_basics.inc.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.imui_action_basics.root";
const TEST_ID_COUNT: &str = "cookbook.imui_action_basics.count";
const TEST_ID_BUTTON_DECL: &str = "cookbook.imui_action_basics.button.declarative";
const TEST_ID_BUTTON_IMUI: &str = "cookbook.imui_action_basics.button.imui";

struct ImUiActionBasicsView {
    genui_state: Model<Value>,
    genui_spec: SpecV1,
    genui_catalog: Arc<fret_genui_core::catalog::CatalogV1>,
}

impl View for ImUiActionBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        let genui_state = app.models_mut().insert(json!({}));

        let genui_spec: SpecV1 = serde_json::from_value(json!({
            "schema_version": 1,
            "root": "cookbook.imui_action_basics.genui.root",
            "elements": {
                "cookbook.imui_action_basics.genui.root": {
                    "type": "VStack",
                    "props": { "gap": "N2" },
                    "children": [
                        "cookbook.imui_action_basics.genui.menu",
                        "cookbook.imui_action_basics.genui.button.inc"
                    ]
                },
                "cookbook.imui_action_basics.genui.menu": {
                    "type": "DropdownMenu",
                    "props": {
                        "trigger": "Open (genui menu)",
                        "items": [
                            {
                                "type": "item",
                                "label": "Increment (genui menu)",
                                "action": "cookbook.imui_action_basics.inc.v1",
                                "testId": "cookbook.imui_action_basics.genui.menu.inc"
                            }
                        ]
                    },
                    "children": []
                },
                "cookbook.imui_action_basics.genui.button.inc": {
                    "type": "Button",
                    "props": { "label": "Increment (genui)" },
                    "on": { "press": { "action": "cookbook.imui_action_basics.inc.v1" } },
                    "children": []
                }
            }
        }))
        .expect("hardcoded spec must be valid");

        let mut catalog = shadcn_catalog_v1();
        catalog.actions.insert(
            act::Inc::ID_STR.to_string(),
            CatalogActionV1 {
                description: Some(
                    "Dispatch a stable ActionId via the host command pipeline.".to_string(),
                ),
                params: Default::default(),
            },
        );

        app.commands_mut().register(
            CommandId::new(act::Inc::ID_STR),
            CommandMeta::new("Increment (action-first)")
                .with_category("Cookbook")
                .with_keywords(["action-first", "view runtime", "imui", "genui", "increment"])
                .with_scope(CommandScope::Widget),
        );

        Self {
            genui_state,
            genui_spec,
            genui_catalog: Arc::new(catalog),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let count = cx.use_state::<u32>();
        let count_value = cx.watch_model(&count).layout().value_or(0);

        cx.on_action_notify_model_update::<act::Inc, u32>(count.clone(), |v| {
            *v = v.saturating_add(1);
        });

        cx.on_action_availability::<act::Inc>(|_host, _acx| CommandAvailability::Available);

        ui::v_flex(|cx| {
            let genui_panel = cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                let runtime = GenUiRuntime {
                    state: self.genui_state.clone(),
                    action_queue: None,
                    auto_apply_standard_actions: false,
                    limits: RenderLimits::default(),
                    catalog: Some(self.genui_catalog.clone()),
                    catalog_validation: ValidationMode::Strict,
                };
                let mut resolver = ShadcnResolver::new();
                match render_spec(cx, &self.genui_spec, &runtime, &mut resolver) {
                    Ok(out) => {
                        if !out.issues.is_empty() {
                            vec![cx.text(format!("GenUI issues: {:?}", out.issues))]
                        } else {
                            out.roots
                        }
                    }
                    Err(err) => vec![cx.text(format!("GenUI render error: {err}"))],
                }
            });

            let imui_panel = cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                fret_imui::imui(cx, |ui| {
                    ui.text("IMUI");
                    ui.action_button_ex(
                        Arc::from("Increment (imui)"),
                        act::Inc,
                        ButtonOptions {
                            test_id: Some(Arc::from(TEST_ID_BUTTON_IMUI)),
                            ..Default::default()
                        },
                    );
                })
            });

            ui::children![
                cx;
                shadcn::Label::new("Cross-frontend action dispatch"),
                cx.text(format!("Count: {count_value}"))
                    .test_id(TEST_ID_COUNT),
                shadcn::Button::new("Increment (declarative)")
                    .action(act::Inc)
                    .a11y_role(SemanticsRole::Button)
                    .test_id(TEST_ID_BUTTON_DECL),
                genui_panel,
                imui_panel,
            ]
        })
        .size_full()
        .gap(Space::N4)
        .test_id(TEST_ID_ROOT)
        .into_element(cx)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-action-basics")
        .window("cookbook-imui-action-basics", (720.0, 420.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .command_palette(true)
        .run_view::<ImUiActionBasicsView>()
        .map_err(anyhow::Error::from)
}
