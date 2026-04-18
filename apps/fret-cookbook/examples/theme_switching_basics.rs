use std::sync::Arc;

use fret::app::prelude::*;
use fret::style::{LayoutRefinement, Space};
use fret_app::Effect;

const TEST_ID_ROOT: &str = "cookbook.theme_switching_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.theme_switching_basics.toggle";
const TEST_ID_TOGGLE_LIGHT: &str = "cookbook.theme_switching_basics.toggle.light";
const TEST_ID_TOGGLE_DARK: &str = "cookbook.theme_switching_basics.toggle.dark";
const TEST_ID_SCHEME: &str = "cookbook.theme_switching_basics.scheme";
const TEST_ID_SAMPLE_CARD: &str = "cookbook.theme_switching_basics.sample_card";

const SCHEME_LIGHT: &str = "light";
const SCHEME_DARK: &str = "dark";

fn apply_scheme(app: &mut App, scheme: &str) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        match scheme {
            SCHEME_DARK => shadcn::themes::ShadcnColorScheme::Dark,
            _ => shadcn::themes::ShadcnColorScheme::Light,
        },
    );
}

struct ThemeSwitchingBasicsView {
    window: WindowId,
    applied_scheme: Option<Arc<str>>,
}

impl View for ThemeSwitchingBasicsView {
    fn init(app: &mut App, window: WindowId) -> Self {
        apply_scheme(app, SCHEME_LIGHT);

        Self {
            window,
            applied_scheme: Some(Arc::from(SCHEME_LIGHT)),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let scheme_state = cx
            .state()
            .local_init(|| Some::<Arc<str>>(Arc::from(SCHEME_LIGHT)));
        let scheme: Arc<str> = cx
            .state()
            .watch(&scheme_state)
            .layout()
            .value_or_default()
            .unwrap_or_else(|| Arc::from(SCHEME_LIGHT));

        let applied_mismatch = match self.applied_scheme.as_ref() {
            Some(applied) => applied.as_ref() != scheme.as_ref(),
            None => true,
        };
        if applied_mismatch {
            apply_scheme(cx.app_mut(), scheme.as_ref());
            self.applied_scheme = Some(scheme.clone());
            let app = cx.app_mut();
            app.request_redraw(self.window);
            app.push_effect(Effect::RequestAnimationFrame(self.window));
        }

        let scheme_label = match scheme.as_ref() {
            SCHEME_DARK => "Dark",
            _ => "Light",
        };

        let header = shadcn::card_header(|cx| {
            ui::children![
                cx;
                shadcn::card_title("Theme switching basics"),
                shadcn::card_description(
                    "A minimal example that toggles between shadcn New York v4 Light/Dark.",
                ),
            ]
        });

        let scheme_row = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("Active scheme:"),
                shadcn::Badge::new(scheme_label).test_id(TEST_ID_SCHEME),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let scheme_toggle = shadcn::ToggleGroup::single(&scheme_state)
            .items([
                shadcn::ToggleGroupItem::new(SCHEME_LIGHT, [ui::text("Light").into_element_in(cx)])
                    .a11y_label("Light")
                    .test_id(TEST_ID_TOGGLE_LIGHT),
                shadcn::ToggleGroupItem::new(SCHEME_DARK, [ui::text("Dark").into_element_in(cx)])
                    .a11y_label("Dark")
                    .test_id(TEST_ID_TOGGLE_DARK),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
            .test_id(TEST_ID_TOGGLE);

        // Avoid `ui::h_flex` here: its internal flex sizing forces `width: fill` by default, which
        // can cause children to get a much larger hit box than intended.
        let toggle_row = ui::h_row(|_cx| [scheme_toggle]).justify_center().w_full();

        let sample = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Sample surface"),
                        shadcn::card_description(
                            "Buttons + tokens should match the active scheme.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::h_flex(|cx| {
                            ui::children![
                                cx;
                                shadcn::Button::new("Default"),
                                shadcn::Button::new("Outline")
                                    .variant(shadcn::ButtonVariant::Outline),
                                shadcn::Button::new("Secondary")
                                    .variant(shadcn::ButtonVariant::Secondary),
                            ]
                        })
                        .gap(Space::N2),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_SAMPLE_CARD);

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                header,
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| ui::children![cx; scheme_row, toggle_row, sample])
                            .gap(Space::N5)
                            .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(560.0));

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-theme-switching-basics")
        .window("cookbook-theme-switching-basics", (720.0, 520.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<ThemeSwitchingBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
