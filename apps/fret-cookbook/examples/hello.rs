use fret::prelude::*;

const TEST_ID_ROOT: &str = "cookbook.hello.root";
const TEST_ID_LABEL: &str = "cookbook.hello.label";
const TEST_ID_BUTTON: &str = "cookbook.hello.button";

#[derive(Debug, Clone)]
enum Msg {
    Click,
}

struct HelloProgram;

impl MvuProgram for HelloProgram {
    type State = ();
    type Message = Msg;

    fn init(_app: &mut App, _window: AppWindowId) -> Self::State {}

    fn update(_app: &mut App, _state: &mut Self::State, message: Self::Message) {
        match message {
            Msg::Click => {
                println!("hello: clicked");
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        _state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let click_cmd = msg.cmd(Msg::Click);

        ui::v_flex(cx, |cx| {
            [
                shadcn::Label::new("Hello, Fret cookbook!")
                    .into_element(cx)
                    .test_id(TEST_ID_LABEL),
                shadcn::Button::new("Click me")
                    .on_click(click_cmd)
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
