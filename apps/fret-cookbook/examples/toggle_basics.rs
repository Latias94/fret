use fret::prelude::*;

mod act {
    fret::actions!([ToggleBookmark = "cookbook.toggle_basics.toggle_bookmark.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.toggle_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.toggle_basics.toggle";
const TEST_ID_STATE: &str = "cookbook.toggle_basics.state";

struct ToggleBasicsView;

impl View for ToggleBasicsView {
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let pressed_state = cx.use_local_with(|| false);
        let pressed = pressed_state.watch(cx).layout().value_or(false);
        let status = if pressed { "Pressed" } else { "Not pressed" };

        cx.on_action_notify_toggle_local_bool::<act::ToggleBookmark>(&pressed_state);

        let toggle = shadcn::Toggle::from_pressed(pressed)
            .action(act::ToggleBookmark)
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .a11y_label("Toggle bookmark")
            .leading_icon(IconId::new_static("lucide.bookmark"))
            .label("Bookmark");

        let body = ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("Action-first toggle"),
                cx.text("Renders from a plain local snapshot and mutates through a typed action."),
                toggle.into_element(cx).test_id(TEST_ID_TOGGLE),
                cx.text(format!("State: {status}")).test_id(TEST_ID_STATE),
            ]
        })
        .gap(Space::N3)
        .items_start();

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Toggle basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Demonstrates `Toggle::from_pressed(...)` plus typed actions on view-local state.",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, body);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(520.0));

        fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card)
            .into_element(cx)
            .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-toggle-basics")
        .window("cookbook-toggle-basics", (720.0, 420.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<ToggleBasicsView>()
        .map_err(anyhow::Error::from)
}
