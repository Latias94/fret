use fret::prelude::*;
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
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let rendered_once = cx.with_state(
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

        let count = cx.use_state::<u32>();
        let count_value = cx.watch_model(&count).layout().copied_or(0);

        cx.on_action::<act::Click>({
            let count = count.clone();
            move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&count, |v| *v = v.saturating_add(1));
                println!("hello: clicked");
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action_availability::<act::Click>(|_host, _acx| CommandAvailability::Available);

        ui::v_flex(cx, |cx| {
            [
                shadcn::Label::new("Hello, Fret cookbook!")
                    .into_element(cx)
                    .test_id(TEST_ID_LABEL),
                cx.text(render_marker).test_id(TEST_ID_RENDER_MARKER),
                cx.text(format!("Count: {count_value}"))
                    .test_id(TEST_ID_COUNT),
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
        .run_view::<HelloView>()
        .map_err(anyhow::Error::from)
}
