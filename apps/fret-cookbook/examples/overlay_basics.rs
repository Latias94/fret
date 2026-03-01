use fret::prelude::*;

const TEST_ID_ROOT: &str = "cookbook.overlays.root";
const TEST_ID_DIALOG_TRIGGER: &str = "cookbook.overlays.dialog.trigger";
const TEST_ID_DIALOG_CONTENT: &str = "cookbook.overlays.dialog.content";
const TEST_ID_DIALOG_CLOSE: &str = "cookbook.overlays.dialog.close";

struct OverlayBasicsState {
    dialog_open: Model<bool>,
}

struct OverlayBasicsProgram;

impl MvuProgram for OverlayBasicsProgram {
    type State = OverlayBasicsState;
    type Message = ();

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            dialog_open: app.models_mut().insert(false),
        }
    }

    fn update(_app: &mut App, _state: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();
        let dialog_open_for_dialog = state.dialog_open.clone();
        let dialog_open_for_trigger = state.dialog_open.clone();
        let dialog_open_for_footer = state.dialog_open.clone();
        let dialog_open_for_close = state.dialog_open.clone();

        let dialog = shadcn::Dialog::new(dialog_open_for_dialog).into_element(
            cx,
            move |cx| {
                let content = ui::v_flex(cx, |cx| {
                    [shadcn::Button::new("Open dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(dialog_open_for_trigger.clone())
                        .into_element(cx)
                        .a11y_role(SemanticsRole::Button)
                        .test_id(TEST_ID_DIALOG_TRIGGER)]
                })
                .gap(Space::N3)
                .items_center()
                .into_element(cx);

                shadcn::Card::new([
                    shadcn::CardHeader::new([
                        shadcn::CardTitle::new("Overlay basics").into_element(cx),
                        shadcn::CardDescription::new(
                            "A minimal Dialog example with stable test IDs.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new([content]).into_element(cx),
                ])
                .ui()
                .w_full()
                .max_w(Px(520.0))
                .into_element(cx)
            },
            move |cx| {
                shadcn::DialogContent::new([
                    shadcn::DialogHeader::new([
                        shadcn::DialogTitle::new("Hello overlays").into_element(cx),
                        shadcn::DialogDescription::new(
                            "This is a minimal dialog example with stable test IDs.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DialogFooter::new([shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(dialog_open_for_footer.clone())
                        .into_element(cx)])
                    .into_element(cx),
                    // Order matters: `DialogClose` is absolutely-positioned and should be the last
                    // child so it stays above the Dialog content in hit testing.
                    shadcn::DialogClose::new(dialog_open_for_close.clone())
                        .into_element(cx)
                        .test_id(TEST_ID_DIALOG_CLOSE),
                ])
                .into_element(cx)
                .test_id(TEST_ID_DIALOG_CONTENT)
            },
        );

        ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [dialog])
                .gap(Space::N6)
                .items_center()
                .justify_center()
                .size_full()
                .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N6)
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-overlay-basics")
        .window("cookbook-overlay-basics", (560.0, 420.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<OverlayBasicsProgram>()
        .map_err(anyhow::Error::from)
}
