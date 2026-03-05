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
    shadcn::shadcn_themes::apply_shadcn_new_york(
        app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        match scheme {
            SCHEME_DARK => shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            _ => shadcn::shadcn_themes::ShadcnColorScheme::Light,
        },
    );
}

struct ThemeSwitchingBasicsView {
    window: AppWindowId,
    scheme: Model<Option<Arc<str>>>,
    applied_scheme: Option<Arc<str>>,
}

impl View for ThemeSwitchingBasicsView {
    fn init(app: &mut App, window: AppWindowId) -> Self {
        apply_scheme(app, SCHEME_LIGHT);

        Self {
            window,
            scheme: app.models_mut().insert(Some(Arc::from(SCHEME_LIGHT))),
            applied_scheme: Some(Arc::from(SCHEME_LIGHT)),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let scheme = cx
            .watch_model(&self.scheme)
            .layout()
            .cloned_or_default()
            .unwrap_or_else(|| Arc::from(SCHEME_LIGHT));

        let applied_mismatch = match self.applied_scheme.as_ref() {
            Some(applied) => applied.as_ref() != scheme.as_ref(),
            None => true,
        };
        if applied_mismatch {
            apply_scheme(&mut *cx.app, scheme.as_ref());
            self.applied_scheme = Some(scheme.clone());
            cx.app.request_redraw(self.window);
            cx.app
                .push_effect(Effect::RequestAnimationFrame(self.window));
        }

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

        let scheme_row = ui::h_flex(|cx| {
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

        let scheme_toggle = shadcn::ToggleGroup::single(self.scheme.clone())
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
            shadcn::CardContent::new([ui::h_flex(|cx| {
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

        let content_body = ui::v_flex(|_cx| [scheme_row, toggle_row, sample])
            .gap(Space::N5)
            .w_full()
            .into_element(cx);
        let content = shadcn::CardContent::new([content_body]).into_element(cx);

        let card = shadcn::Card::new([header, content])
            .ui()
            .w_full()
            .max_w(Px(560.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-theme-switching-basics")
        .window("cookbook-theme-switching-basics", (720.0, 520.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<ThemeSwitchingBasicsView>()
        .map_err(anyhow::Error::from)
}
