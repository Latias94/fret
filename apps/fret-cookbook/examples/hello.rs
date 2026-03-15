use fret::app::prelude::*;
use fret::style::Space;
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([Click = "cookbook.hello.click.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.hello.root";
const TEST_ID_LABEL: &str = "cookbook.hello.label";
const TEST_ID_BUTTON: &str = "cookbook.hello.button";
const TEST_ID_COUNT: &str = "cookbook.hello.count";
const TEST_ID_RENDER_MARKER: &str = "cookbook.hello.render_marker";

struct HelloView;

impl View for HelloView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let rendered_once = cx.slot_state(
            || false,
            |v| {
                let first = !*v;
                *v = true;
                first
            },
        );
        let render_marker = if rendered_once {
            "RenderedOnce"
        } else {
            "RenderedAgain"
        };

        let count_state = cx.state().local::<u32>();
        let count_value = cx.state().watch(&count_state).layout().value_or(0);

        cx.actions()
            .local_update::<act::Click, u32>(&count_state, |v| {
                *v = v.saturating_add(1);
                println!("hello: clicked");
            });

        cx.actions()
            .availability::<act::Click>(|_host, _acx| CommandAvailability::Available);

        let root = ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("Hello, Fret cookbook!").test_id(TEST_ID_LABEL),
                cx.text(render_marker).test_id(TEST_ID_RENDER_MARKER),
                cx.text(format!("Count: {count_value}")).test_id(TEST_ID_COUNT),
                shadcn::Button::new("Click me")
                    .action(act::Click)
                    .test_id(TEST_ID_BUTTON),
            ]
        })
        .size_full()
        .gap(Space::N4)
        .items_center()
        .justify_center()
        .test_id(TEST_ID_ROOT);

        root.into_element(cx).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-hello")
        .window("cookbook-hello", (560.0, 360.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<HelloView>()?
        .run()
        .map_err(anyhow::Error::from)
}
