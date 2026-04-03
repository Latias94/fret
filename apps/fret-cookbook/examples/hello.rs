use fret::app::prelude::*;
use fret::style::Space;

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
        let render_marker = "ViewReady";
        let count_state = cx.state().local_init(|| 0u32);
        let count_value = count_state.layout_value(cx);

        cx.actions().local(&count_state).update::<act::Click>(|v| {
            *v = v.saturating_add(1);
            println!("hello: clicked");
        });

        ui::single(cx, hello_page(render_marker, count_value))
    }
}

fn hello_page(render_marker: &'static str, count_value: u32) -> impl UiChild {
    ui::v_flex(move |cx| {
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
    .test_id(TEST_ID_ROOT)
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-hello")
        .window("cookbook-hello", (560.0, 360.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<HelloView>()?
        .run()
        .map_err(anyhow::Error::from)
}
