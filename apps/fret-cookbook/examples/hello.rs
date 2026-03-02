use fret::prelude::*;
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([Click = "cookbook.hello.click.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.hello.root";
const TEST_ID_LABEL: &str = "cookbook.hello.label";
const TEST_ID_BUTTON: &str = "cookbook.hello.button";
const TEST_ID_COUNT: &str = "cookbook.hello.count";

struct HelloState {
    count: Model<u32>,
}

struct HelloProgram;

impl MvuProgram for HelloProgram {
    type State = HelloState;
    type Message = ();

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            count: app.models_mut().insert(0),
        }
    }

    fn update(_app: &mut App, _state: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let base_root = cx.root_id();

        let count = cx.watch_model(&state.count).layout().copied_or(0);

        let count_model = state.count.clone();
        let (on_command, on_command_availability) = fret::actions::ActionHandlerTable::new()
            .on::<act::Click>(move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&count_model, |v| *v = v.saturating_add(1));

                println!("hello: clicked");
                host.request_redraw(acx.window);
                true
            })
            .availability::<act::Click>(|_host, _acx| CommandAvailability::Available)
            .build();

        cx.command_on_command_for(base_root, on_command);
        cx.command_on_command_availability_for(base_root, on_command_availability);

        ui::v_flex(cx, |cx| {
            [
                shadcn::Label::new("Hello, Fret cookbook!")
                    .into_element(cx)
                    .test_id(TEST_ID_LABEL),
                cx.text(format!("Count: {count}")).test_id(TEST_ID_COUNT),
                shadcn::Button::new("Click me")
                    .action(act::Click)
                    .into_element(cx)
                    .a11y_role(SemanticsRole::Button)
                    .test_id(TEST_ID_BUTTON),
            ]
        })
        .size_full()
        .gap(Space::N4)
        .items_center()
        .justify_center()
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-hello")
        .window("cookbook-hello", (560.0, 360.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<HelloProgram>()
        .map_err(anyhow::Error::from)
}
