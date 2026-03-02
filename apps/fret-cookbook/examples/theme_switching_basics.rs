use fret::prelude::*;
use std::sync::Arc;

const TEST_ID_ROOT: &str = "cookbook.theme_switching_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.theme_switching_basics.toggle";
const TEST_ID_TOGGLE_LIGHT: &str = "cookbook.theme_switching_basics.toggle.light";
const TEST_ID_TOGGLE_DARK: &str = "cookbook.theme_switching_basics.toggle.dark";
const TEST_ID_SCHEME: &str = "cookbook.theme_switching_basics.scheme";
const TEST_ID_SAMPLE_CARD: &str = "cookbook.theme_switching_basics.sample_card";

const SCHEME_LIGHT: &str = "light";
const SCHEME_DARK: &str = "dark";

fn apply_scheme(app: &mut App, scheme: &str) {
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        match scheme {
            SCHEME_DARK => shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            _ => shadcn::shadcn_themes::ShadcnColorScheme::Light,
        },
    );
}

struct ThemeSwitchingBasicsState {
    window: AppWindowId,
    scheme: Model<Option<Arc<str>>>,
    applied_scheme: Option<Arc<str>>,
}

struct ThemeSwitchingBasicsProgram;

impl MvuProgram for ThemeSwitchingBasicsProgram {
    type State = ThemeSwitchingBasicsState;
    type Message = ();

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        apply_scheme(app, SCHEME_LIGHT);

        Self::State {
            window,
            scheme: app.models_mut().insert(Some(Arc::from(SCHEME_LIGHT))),
            applied_scheme: Some(Arc::from(SCHEME_LIGHT)),
        }
    }

    fn update(_app: &mut App, _state: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let scheme = state
            .scheme
            .read(&mut *cx.app, |_host, v| v.clone())
            .ok()
            .flatten()
            .unwrap_or_else(|| Arc::from(SCHEME_LIGHT));

        let applied_mismatch = match state.applied_scheme.as_ref() {
            Some(applied) => applied.as_ref() != scheme.as_ref(),
            None => true,
        };
        if applied_mismatch {
            apply_scheme(&mut *cx.app, scheme.as_ref());
            state.applied_scheme = Some(scheme.clone());
            cx.app.request_redraw(state.window);
            cx.app
                .push_effect(Effect::RequestAnimationFrame(state.window));
        }

        let theme = Theme::global(&*cx.app).snapshot();

        let scheme_label = match scheme.as_ref() {
            SCHEME_DARK => "Dark",
            _ => "Light",
        };

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

        let scheme_toggle = shadcn::ToggleGroup::single(state.scheme.clone())
            .items([
                shadcn::ToggleGroupItem::new(SCHEME_LIGHT, [cx.text("Light")])
                    .a11y_label("Light")
                    .test_id(TEST_ID_TOGGLE_LIGHT),
                shadcn::ToggleGroupItem::new(SCHEME_DARK, [cx.text("Dark")])
                    .a11y_label("Dark")
                    .test_id(TEST_ID_TOGGLE_DARK),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
            .into_element(cx)
            .test_id(TEST_ID_TOGGLE);

        // Avoid `ui::h_flex` here: its internal flex sizing is always `width: fill`, which can
        // cause children to get a much larger hit box than intended.
        let toggle_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .justify_center()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| [scheme_toggle],
        );

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
            .gap(Space::N5)
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
        .bg(ColorRef::Color(theme.color_token("background")))
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
