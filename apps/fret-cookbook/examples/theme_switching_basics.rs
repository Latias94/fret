use fret::prelude::*;

const TEST_ID_ROOT: &str = "cookbook.theme_switching_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.theme_switching_basics.toggle";
const TEST_ID_SCHEME: &str = "cookbook.theme_switching_basics.scheme";
const TEST_ID_SAMPLE_CARD: &str = "cookbook.theme_switching_basics.sample_card";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scheme {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
enum Msg {
    ToggleScheme,
}

struct ThemeSwitchingBasicsState {
    window: AppWindowId,
    scheme: Model<Scheme>,
}

struct ThemeSwitchingBasicsProgram;

impl MvuProgram for ThemeSwitchingBasicsProgram {
    type State = ThemeSwitchingBasicsState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        Self::State {
            window,
            scheme: app.models_mut().insert(Scheme::Light),
        }
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        match message {
            Msg::ToggleScheme => {
                let scheme = state
                    .scheme
                    .read(app, |_host, v| *v)
                    .unwrap_or(Scheme::Light);

                let next = match scheme {
                    Scheme::Light => Scheme::Dark,
                    Scheme::Dark => Scheme::Light,
                };

                let _ = state.scheme.update(app, |v, _cx| *v = next);

                shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                    app,
                    shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                    match next {
                        Scheme::Light => shadcn::shadcn_themes::ShadcnColorScheme::Light,
                        Scheme::Dark => shadcn::shadcn_themes::ShadcnColorScheme::Dark,
                    },
                );

                app.request_redraw(state.window);
                app.push_effect(Effect::RequestAnimationFrame(state.window));
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let scheme = state
            .scheme
            .read(&mut *cx.app, |_host, v| *v)
            .unwrap_or(Scheme::Light);
        let scheme_label = match scheme {
            Scheme::Light => "Light",
            Scheme::Dark => "Dark",
        };

        let toggle_cmd = msg.cmd(Msg::ToggleScheme);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Theme switching basics").into_element(cx),
            shadcn::CardDescription::new(
                "A minimal example that toggles between shadcn New York v4 Light/Dark.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let scheme_row = ui::h_flex(cx, |cx| {
            [
                shadcn::Label::new("Active scheme:").into_element(cx),
                shadcn::Badge::new(scheme_label)
                    .into_element(cx)
                    .test_id(TEST_ID_SCHEME),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let toggle_button = shadcn::Button::new("Toggle Light/Dark")
            .variant(shadcn::ButtonVariant::Outline)
            .on_click(toggle_cmd)
            .into_element(cx)
            .a11y_role(SemanticsRole::Button)
            .test_id(TEST_ID_TOGGLE);

        // Keep the button's hit box tight even when parent stacks default to stretch sizing.
        let toggle_row = ui::h_flex(cx, |_cx| [toggle_button])
            .items_center()
            .into_element(cx);

        let sample = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Sample surface").into_element(cx),
                shadcn::CardDescription::new("Buttons + tokens should match the active scheme.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([ui::h_flex(cx, |cx| {
                [
                    shadcn::Button::new("Default").into_element(cx),
                    shadcn::Button::new("Outline")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                    shadcn::Button::new("Secondary")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx)])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .into_element(cx)
        .test_id(TEST_ID_SAMPLE_CARD);

        let content_body = ui::v_flex(cx, |_cx| [scheme_row, toggle_row, sample])
            .gap(Space::N4)
            .w_full()
            .into_element(cx);
        let content = shadcn::CardContent::new([content_body]).into_element(cx);

        let card = shadcn::Card::new([header, content])
            .ui()
            .w_full()
            .max_w(Px(560.0))
            .into_element(cx);

        ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [card])
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
    FretApp::new("cookbook-theme-switching-basics")
        .window("cookbook-theme-switching-basics", (720.0, 520.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<ThemeSwitchingBasicsProgram>()
        .map_err(anyhow::Error::from)
}
