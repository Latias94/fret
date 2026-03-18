use fret::app::prelude::*;
use fret::{icons::IconId, style::Space};

mod act {
    fret::actions!([ToggleBookmark = "cookbook.toggle_basics.toggle_bookmark.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.toggle_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.toggle_basics.toggle";
const TEST_ID_STATE: &str = "cookbook.toggle_basics.state";

struct ToggleBasicsView;

impl View for ToggleBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let pressed_state = cx.state().local_init(|| false);
        let pressed = pressed_state.layout_value(cx);
        let status = if pressed { "Pressed" } else { "Not pressed" };

        cx.actions()
            .toggle_local_bool::<act::ToggleBookmark>(&pressed_state);

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

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Toggle basics"),
                        shadcn::card_description(
                            "Demonstrates `Toggle::from_pressed(...)` plus typed actions on view-local state.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; body]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(520.0));

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-toggle-basics")
        .window("cookbook-toggle-basics", (720.0, 420.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<ToggleBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
