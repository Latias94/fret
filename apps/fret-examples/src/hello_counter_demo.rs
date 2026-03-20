use std::sync::Arc;

use fret::app::prelude::*;
use fret::{
    FretApp,
    actions::CommandId,
    icons::{IconId, icon},
    semantics::SemanticsRole,
    shadcn,
    style::{
        ChromeRefinement, ColorRef, Radius, Space, TextOverflow, TextWrap, Theme, ThemeSnapshot,
    },
};
use fret_core::Corners;
use fret_ui::element::TextProps;

mod act {
    fret::actions!([
        Inc = "hello_counter_demo.inc.v1",
        Dec = "hello_counter_demo.dec.v1",
        Reset = "hello_counter_demo.reset.v1",
        SetStep1 = "hello_counter_demo.step.1.v1",
        SetStep5 = "hello_counter_demo.step.5.v1",
        SetStep10 = "hello_counter_demo.step.10.v1"
    ]);
}

const TEST_ID_ROOT: &str = "hello-counter.root";
const TEST_ID_COUNT: &str = "hello-counter.count";
const TEST_ID_STEP_INPUT: &str = "hello-counter.step";
const TEST_ID_DEC: &str = "hello-counter.dec";
const TEST_ID_INC: &str = "hello-counter.inc";
const TEST_ID_RESET: &str = "hello-counter.reset";
const TEST_ID_STEP_1: &str = "hello-counter.step.1";
const TEST_ID_STEP_5: &str = "hello-counter.step.5";
const TEST_ID_STEP_10: &str = "hello-counter.step.10";

fn install_demo_theme(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("hello-counter-demo")
        .window("hello-counter-demo", (520.0, 420.0))
        .config_files(false)
        .setup(install_demo_theme)
        .view::<HelloCounterView>()?
        .run()
        .map_err(anyhow::Error::from)
}

struct HelloCounterView;

fn parse_step(step_text: &str) -> (i64, bool) {
    let raw = step_text.trim();
    let Ok(step) = raw.parse::<i64>() else {
        return (1, false);
    };
    if step <= 0 {
        return (1, false);
    }
    (step, true)
}

impl View for HelloCounterView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();

        let count_state = cx.state().local_init(|| 0i64);
        let step_state = cx.state().local_init(|| "1".to_string());

        let count = count_state.layout_value(cx);
        let (effective_step, step_valid) = cx
            .data()
            .selector_layout(&step_state, |step_text| parse_step(step_text.as_str()));

        cx.actions()
            .locals_with((&count_state, &step_state))
            .on::<act::Inc>(|tx, (count_state, step_state)| {
                let step_text = tx.value(&step_state);
                let (step, _) = parse_step(&step_text);
                tx.update(&count_state, |value| *value = value.saturating_add(step))
            });

        cx.actions()
            .locals_with((&count_state, &step_state))
            .on::<act::Dec>(|tx, (count_state, step_state)| {
                let step_text = tx.value(&step_state);
                let (step, _) = parse_step(&step_text);
                tx.update(&count_state, |value| *value = value.saturating_sub(step))
            });

        cx.actions().local(&count_state).set::<act::Reset>(0);
        cx.actions()
            .local(&step_state)
            .set::<act::SetStep1>("1".to_string());
        cx.actions()
            .local(&step_state)
            .set::<act::SetStep5>("5".to_string());
        cx.actions()
            .local(&step_state)
            .set::<act::SetStep10>("10".to_string());

        let count_color = if count > 0 {
            theme.color_token("primary")
        } else if count < 0 {
            theme.color_token("destructive")
        } else {
            theme.color_token("foreground")
        };

        let inc_cmd: CommandId = act::Inc.into();

        let hero_icon = ui::h_flex(|cx| {
            [icon::icon_with(
                cx,
                IconId::new("lucide.party-popper"),
                Some(Px(22.0)),
                Some(ColorRef::Color(theme.color_token("foreground"))),
            )]
        })
        .bg(ColorRef::Color(theme.color_token("secondary")))
        .rounded(Radius::Full)
        .w_px(Px(48.0))
        .h_px(Px(48.0))
        .items_center()
        .justify_center()
        .into_element(cx);

        let header_inner = ui::v_flex(|cx| {
            ui::children![
                cx;
                hero_icon,
                shadcn::CardTitle::new("Hello Counter"),
                shadcn::CardDescription::new(
                    "A minimal counter demo using `fret` + shadcn/ui (View runtime + typed actions).",
                ),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let header = shadcn::CardHeader::new([header_inner]);

        let count_text = cx
            .text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(count.to_string()),
                style: Some(fret_core::TextStyle {
                    size: Px(72.0),
                    weight: fret_core::FontWeight::BOLD,
                    ..Default::default()
                }),
                color: Some(count_color),
                align: fret_core::TextAlign::Center,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                ink_overflow: Default::default(),
            })
            .test_id(TEST_ID_COUNT);

        let status_text: Arc<str> = Arc::from(if count == 0 {
            "Status: idle"
        } else if count > 0 {
            "Status: increasing"
        } else {
            "Status: decreasing"
        });
        let status_line = cx.text_props(TextProps {
            layout: Default::default(),
            text: status_text,
            style: None,
            color: Some(theme.color_token("muted-foreground")),
            align: fret_core::TextAlign::Center,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            ink_overflow: Default::default(),
        });

        let step_badge =
            shadcn::Badge::new(format!("Step: {effective_step}")).variant(if step_valid {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Destructive
            });

        let step_help = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(if step_valid {
                "Edit step, then press Enter to increment."
            } else {
                "Step must be a positive integer (using 1)."
            }),
            style: None,
            color: Some(theme.color_token("muted-foreground")),
            align: fret_core::TextAlign::Center,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            ink_overflow: Default::default(),
        });

        let step_input = shadcn::Input::new(&step_state)
            .placeholder("Step (e.g. 1)")
            .submit_action(inc_cmd)
            .into_element(cx)
            .role(SemanticsRole::TextField)
            .test_id(TEST_ID_STEP_INPUT);

        let presets = ui::h_flex(|_cx| {
            [
                shadcn::Button::new("1")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::SetStep1)
                    .test_id(TEST_ID_STEP_1),
                shadcn::Button::new("5")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::SetStep5)
                    .test_id(TEST_ID_STEP_5),
                shadcn::Button::new("10")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::SetStep10)
                    .test_id(TEST_ID_STEP_10),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let step_row = ui::v_flex(|_cx| [step_input, presets])
            .gap(Space::N2)
            .w_full()
            .items_center()
            .into_element(cx);

        let actions = ui::h_flex(|cx| {
            [
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconLg)
                    .corner_radii_override(Corners::all(Px(9999.0)))
                    .action(act::Dec)
                    .children([icon::icon(cx, IconId::new("lucide.minus"))])
                    .into_element(cx)
                    .role(SemanticsRole::Button)
                    .a11y_label("Decrement")
                    .test_id(TEST_ID_DEC),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Reset)
                    .children([icon::icon(cx, IconId::new("lucide.rotate-ccw"))])
                    .into_element(cx)
                    .role(SemanticsRole::Button)
                    .test_id(TEST_ID_RESET),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Default)
                    .size(shadcn::ButtonSize::IconLg)
                    .corner_radii_override(Corners::all(Px(9999.0)))
                    .action(act::Inc)
                    .children([icon::icon(cx, IconId::new("lucide.plus"))])
                    .into_element(cx)
                    .role(SemanticsRole::Button)
                    .a11y_label("Increment")
                    .test_id(TEST_ID_INC),
            ]
        })
        .gap(Space::N4)
        .items_center()
        .into_element(cx);

        let content_body = ui::v_flex(|cx| {
            [
                ui::v_flex(|cx| ui::children![cx; count_text, status_line, step_badge])
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                ui::v_flex(|_cx| [step_row, step_help])
                    .gap(Space::N2)
                    .w_full()
                    .items_center()
                    .into_element(cx),
            ]
        })
        .gap(Space::N6)
        .items_center()
        .into_element(cx);

        let content = shadcn::CardContent::new([content_body]);
        let footer = shadcn::CardFooter::new([actions]);

        let card = shadcn::Card::new(ui::children![cx; header, content, footer])
            .refine_style(ChromeRefinement::default().shadow_lg())
            .ui()
            .w_full()
            .max_w(Px(480.0))
            .into_element(cx);

        ui::single(cx, hello_counter_page(theme, card))
    }
}

fn hello_counter_page(theme: ThemeSnapshot, card: impl UiChild) -> impl UiChild {
    ui::container(|cx| {
        ui::single(
            cx,
            ui::v_flex(|_cx| [card])
                .items_center()
                .justify_center()
                .gap(Space::N6)
                .size_full(),
        )
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .size_full()
    .test_id(TEST_ID_ROOT)
}
