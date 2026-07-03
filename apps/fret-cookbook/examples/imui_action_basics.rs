use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::commands::{CommandId, CommandMeta, CommandScope};
use fret::imui::{
    kit::{ButtonOptions, InputTextOptions, SameLineOptions},
    prelude::*,
};
use fret::semantics::SemanticsRole;
use fret::style::Space;

mod act {
    fret::actions!([Inc = "cookbook.imui_action_basics.inc.v1"]);
    fret::payload_actions!([SetCount(u32) = "cookbook.imui_action_basics.set_count.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.imui_action_basics.root";
const TEST_ID_COUNT: &str = "cookbook.imui_action_basics.count";
const TEST_ID_BUTTON_DECL: &str = "cookbook.imui_action_basics.button.declarative";
const TEST_ID_BUTTON_IMUI: &str = "cookbook.imui_action_basics.button.imui";
const TEST_ID_BUTTON_IMUI_PAYLOAD_1: &str = "cookbook.imui_action_basics.button.imui.payload.1";
const TEST_ID_BUTTON_IMUI_PAYLOAD_5: &str = "cookbook.imui_action_basics.button.imui.payload.5";
const TEST_ID_BUTTON_IMUI_PAYLOAD_10: &str = "cookbook.imui_action_basics.button.imui.payload.10";
const TEST_ID_PAYLOAD_ROW: &str = "cookbook.imui_action_basics.button.imui.payload.row";
const TEST_ID_INPUT_FILTER: &str = "cookbook.imui_action_basics.input.filter";
const TEST_ID_INPUT_SNAPSHOT: &str = "cookbook.imui_action_basics.input.snapshot";

struct ImUiActionBasicsView {
    filter_text: LocalState<String>,
    snapshot_text: LocalState<String>,
}

impl View for ImUiActionBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        let filter_text = app.local_state(String::from("Actions"));
        let snapshot_text = app.local_state(String::from("Read-only action snapshot"));

        app.commands_mut().register(
            CommandId::new(act::Inc::ID_STR),
            CommandMeta::new("Increment (action-first)")
                .with_category("Cookbook")
                .with_keywords(["action-first", "view runtime", "imui", "increment"])
                .with_scope(CommandScope::Widget),
        );

        Self {
            filter_text,
            snapshot_text,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let count_state = cx.state().local_init(|| 0u32);
        let count_value = count_state.layout_value(cx);

        cx.actions().local(&count_state).update::<act::Inc>(|v| {
            *v = v.saturating_add(1);
        });
        cx.actions()
            .local(&count_state)
            .payload_update_if::<act::SetCount>(|value, preset| {
                *value = preset;
                true
            });

        ui::v_flex(|cx| {
            let imui_panel = ui::v_flex(|cx| {
                imui_in(cx, |ui| {
                    ui.text("IMUI");
                    ui.action_button_with_options(
                        Arc::from("Increment (imui)"),
                        act::Inc,
                        ButtonOptions {
                            test_id: Some(Arc::from(TEST_ID_BUTTON_IMUI)),
                            ..Default::default()
                        },
                    );
                    let _ = ui.input_text_local_with_options(
                        &self.filter_text,
                        InputTextOptions {
                            select_all_on_focus: true,
                            placeholder: Some(Arc::from("Filter")),
                            test_id: Some(Arc::from(TEST_ID_INPUT_FILTER)),
                            ..Default::default()
                        },
                    );
                    let _ = ui.input_text_local_with_options(
                        &self.snapshot_text,
                        InputTextOptions {
                            read_only: true,
                            test_id: Some(Arc::from(TEST_ID_INPUT_SNAPSHOT)),
                            ..Default::default()
                        },
                    );
                    ui.same_line_with_options(
                        SameLineOptions {
                            test_id: Some(Arc::from(TEST_ID_PAYLOAD_ROW)),
                            ..Default::default()
                        },
                        |ui| {
                            for (preset, test_id) in [
                                (1u32, TEST_ID_BUTTON_IMUI_PAYLOAD_1),
                                (5u32, TEST_ID_BUTTON_IMUI_PAYLOAD_5),
                                (10u32, TEST_ID_BUTTON_IMUI_PAYLOAD_10),
                            ] {
                                ui.action_payload_button_with_options(
                                    Arc::from(format!("Set {preset}")),
                                    act::SetCount,
                                    preset,
                                    ButtonOptions {
                                        test_id: Some(Arc::from(test_id)),
                                        ..Default::default()
                                    },
                                );
                            }
                        },
                    );
                })
            });

            ui::children![
                cx;
                shadcn::Label::new("Action dispatch"),
                cx.text(format!("Count: {count_value}"))
                    .test_id(TEST_ID_COUNT),
                shadcn::Button::new("Increment (declarative)")
                    .action(act::Inc)
                    .a11y_role(SemanticsRole::Button)
                    .test_id(TEST_ID_BUTTON_DECL),
                imui_panel,
            ]
        })
        .size_full()
        .gap(Space::N4)
        .test_id(TEST_ID_ROOT)
        .into_element_in(cx)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-action-basics")
        .window("cookbook-imui-action-basics", (720.0, 420.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .command_palette(true)
        .view::<ImUiActionBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
