use std::sync::Arc;

use fret::app::prelude::*;
use fret::{
    icons::IconId,
    style::{ChromeRefinement, ColorRef, Radius, Space, TextOverflow, TextWrap, Theme},
};
use fret_core::{Corners, FontWeight, TextAlign};
use fret_ui::element::TextProps;

mod act {
    fret::actions!([
        Inc = "cookbook.hello_counter.inc.v1",
        Dec = "cookbook.hello_counter.dec.v1",
        Reset = "cookbook.hello_counter.reset.v1",
        StepPreset1 = "cookbook.hello_counter.step_preset_1.v1",
        StepPreset5 = "cookbook.hello_counter.step_preset_5.v1",
        StepPreset10 = "cookbook.hello_counter.step_preset_10.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.hello_counter.root";
const TEST_ID_COUNT: &str = "hello-counter.count";
const TEST_ID_STEP_INPUT: &str = "hello-counter.step";
const TEST_ID_DEC: &str = "hello-counter.dec";
const TEST_ID_INC: &str = "hello-counter.inc";
const TEST_ID_RESET: &str = "hello-counter.reset";
const TEST_ID_STEP_1: &str = "hello-counter.step.1";
const TEST_ID_STEP_5: &str = "hello-counter.step.5";
const TEST_ID_STEP_10: &str = "hello-counter.step.10";

pub fn run() -> anyhow::Result<()> {
    FretApp::new("hello-counter-demo")
        .window("hello_counter_demo", (520.0, 420.0))
        .setup(fret_cookbook::install_cookbook_defaults)
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

        let count = cx.state().watch(&count_state).layout().value_or(0);
        let step_text = cx
            .state()
            .watch(&step_state)
            .layout()
            .value_or_else(String::new);
        let (effective_step, step_valid) = parse_step(&step_text);

        let count_color = if count > 0 {
            theme.color_token("primary")
        } else if count < 0 {
            theme.color_token("destructive")
        } else {
            theme.color_token("foreground")
        };

        cx.actions().locals::<act::Inc>({
            let count_state = count_state.clone();
            let step_state = step_state.clone();
            move |tx| {
                let step_text = tx.value_or_else(&step_state, || "1".to_string());
                let (step, _) = parse_step(&step_text);
                tx.update(&count_state, |value| *value = value.saturating_add(step))
            }
        });

        cx.actions().locals::<act::Dec>({
            let count_state = count_state.clone();
            let step_state = step_state.clone();
            move |tx| {
                let step_text = tx.value_or_else(&step_state, || "1".to_string());
                let (step, _) = parse_step(&step_text);
                tx.update(&count_state, |value| *value = value.saturating_sub(step))
            }
        });

        cx.actions().local_set::<act::Reset, i64>(&count_state, 0);
        cx.actions()
            .local_set::<act::StepPreset1, String>(&step_state, "1".to_string());
        cx.actions()
            .local_set::<act::StepPreset5, String>(&step_state, "5".to_string());
        cx.actions()
            .local_set::<act::StepPreset10, String>(&step_state, "10".to_string());

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
        .justify_center();

        let header = shadcn::card_header(|cx| {
            ui::children![cx;
                ui::v_flex(|cx| {
                    ui::children![
                        cx;
                        hero_icon,
                        shadcn::card_title("Hello Counter"),
                        shadcn::card_description(
                            "A minimal counter demo using `fret` + shadcn/ui (view runtime + typed actions).",
                        ),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
            ]
        });

        let count_text = cx
            .text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(count.to_string()),
                style: Some(fret_core::TextStyle {
                    size: Px(72.0),
                    weight: FontWeight::BOLD,
                    ..Default::default()
                }),
                color: Some(count_color),
                align: TextAlign::Center,
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
            align: TextAlign::Center,
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
            align: TextAlign::Center,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            ink_overflow: Default::default(),
        });

        let step_input = shadcn::Input::new(&step_state)
            .placeholder("Step (e.g. 1)")
            .submit_command(act::Inc.into())
            .a11y_role(SemanticsRole::TextField)
            .test_id(TEST_ID_STEP_INPUT);

        let presets = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("1")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::StepPreset1)
                    .test_id(TEST_ID_STEP_1),
                shadcn::Button::new("5")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::StepPreset5)
                    .test_id(TEST_ID_STEP_5),
                shadcn::Button::new("10")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::StepPreset10)
                    .test_id(TEST_ID_STEP_10),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let step_row = ui::v_flex(|cx| ui::children![cx; step_input, presets])
            .gap(Space::N2)
            .w_full()
            .items_center();

        let actions = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconLg)
                    .corner_radii_override(Corners::all(Px(9999.0)))
                    .action(act::Dec)
                    .children([icon::icon(cx, IconId::new("lucide.minus"))])
                    .a11y_role(SemanticsRole::Button)
                    .a11y_label("Decrement")
                    .test_id(TEST_ID_DEC),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Reset)
                    .children([icon::icon(cx, IconId::new("lucide.rotate-ccw"))])
                    .a11y_role(SemanticsRole::Button)
                    .test_id(TEST_ID_RESET),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Default)
                    .size(shadcn::ButtonSize::IconLg)
                    .corner_radii_override(Corners::all(Px(9999.0)))
                    .action(act::Inc)
                    .children([icon::icon(cx, IconId::new("lucide.plus"))])
                    .a11y_role(SemanticsRole::Button)
                    .a11y_label("Increment")
                    .test_id(TEST_ID_INC),
            ]
        })
        .gap(Space::N4)
        .items_center();

        let content_body = ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::v_flex(|cx| ui::children![cx; count_text, status_line, step_badge])
                    .gap(Space::N2)
                    .items_center(),
                ui::v_flex(|cx| ui::children![cx; step_row, step_help])
                    .gap(Space::N2)
                    .w_full()
                    .items_center(),
            ]
        })
        .gap(Space::N6)
        .items_center();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                header,
                shadcn::card_content(|cx| ui::children![cx; content_body]),
                shadcn::card_footer(|cx| ui::children![cx; actions]),
            ]
        })
        .refine_style(ChromeRefinement::default().shadow_lg())
        .ui()
        .w_full()
        .max_w(Px(480.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    run()
}
