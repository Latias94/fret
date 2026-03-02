use std::sync::Arc;

use fret::prelude::*;
use fret_ui::CommandAvailability;
use fret_ui_kit::imui::ButtonOptions;

mod act {
    fret::actions!([Inc = "cookbook.imui_action_basics.inc.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.imui_action_basics.root";
const TEST_ID_COUNT: &str = "cookbook.imui_action_basics.count";
const TEST_ID_BUTTON_DECL: &str = "cookbook.imui_action_basics.button.declarative";
const TEST_ID_BUTTON_IMUI: &str = "cookbook.imui_action_basics.button.imui";

struct ImUiActionBasicsView;

impl View for ImUiActionBasicsView {
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let count = cx.use_state::<u32>();
        let count_value = cx.watch_model(&count).layout().copied_or(0);

        cx.on_action::<act::Inc>({
            let count = count.clone();
            move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&count, |v| *v = v.saturating_add(1));
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action_availability::<act::Inc>(|_host, _acx| CommandAvailability::Available);

        ui::v_flex(cx, |cx| {
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

            [
                shadcn::Label::new("Cross-frontend action dispatch").into_element(cx),
                cx.text(format!("Count: {count_value}"))
                    .test_id(TEST_ID_COUNT),
                shadcn::Button::new("Increment (declarative)")
                    .action(act::Inc)
                    .into_element(cx)
                    .a11y_role(SemanticsRole::Button)
                    .test_id(TEST_ID_BUTTON_DECL),
                imui_panel,
            ]
        })
        .size_full()
        .gap(Space::N4)
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-action-basics")
        .window("cookbook-imui-action-basics", (720.0, 420.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<ImUiActionBasicsView>()
        .map_err(anyhow::Error::from)
}
