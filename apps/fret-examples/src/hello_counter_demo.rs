use std::sync::Arc;

use fret::prelude::*;

const TEST_ID_COUNT: &str = "hello-counter.count";
const TEST_ID_STEP_INPUT: &str = "hello-counter.step";
const TEST_ID_DEC: &str = "hello-counter.dec";
const TEST_ID_INC: &str = "hello-counter.inc";
const TEST_ID_RESET: &str = "hello-counter.reset";
const TEST_ID_STEP_1: &str = "hello-counter.step.1";
const TEST_ID_STEP_5: &str = "hello-counter.step.5";
const TEST_ID_STEP_10: &str = "hello-counter.step.10";

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<HelloCounterProgram>("hello-counter-demo")?
        .with_main_window("hello_counter_demo", (520.0, 420.0))
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Light,
            );
        })
        .run()?;
    Ok(())
}

struct HelloCounterState {
    count: Model<i64>,
    step: Model<String>,
}

#[derive(Debug, Clone)]
enum Msg {
    Inc,
    Dec,
    Reset,
    SetStepPreset(i64),
}

struct HelloCounterProgram;

impl MvuProgram for HelloCounterProgram {
    type State = HelloCounterState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        init_window(app, window)
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        update(app, state, message);
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, state, msg)
    }
}

fn init_window(app: &mut App, _window: AppWindowId) -> HelloCounterState {
    HelloCounterState {
        count: app.models_mut().insert(0i64),
        step: app.models_mut().insert("1".to_string()),
    }
}

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

fn update(app: &mut App, state: &mut HelloCounterState, msg: Msg) {
    match msg {
        Msg::Inc => {
            let step_text = app
                .models()
                .read(&state.step, Clone::clone)
                .ok()
                .unwrap_or_else(|| "1".to_string());
            let (step, _) = parse_step(&step_text);
            let _ = app
                .models_mut()
                .update(&state.count, |v| *v = v.saturating_add(step));
        }
        Msg::Dec => {
            let step_text = app
                .models()
                .read(&state.step, Clone::clone)
                .ok()
                .unwrap_or_else(|| "1".to_string());
            let (step, _) = parse_step(&step_text);
            let _ = app
                .models_mut()
                .update(&state.count, |v| *v = v.saturating_sub(step));
        }
        Msg::Reset => {
            let _ = app.models_mut().update(&state.count, |v| *v = 0);
        }
        Msg::SetStepPreset(preset) => {
            let preset = preset.max(1);
            let _ = app
                .models_mut()
                .update(&state.step, |v| *v = preset.to_string());
        }
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut HelloCounterState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();

    let count = cx.watch_model(&st.count).layout().copied_or(0);
    let step_text = cx
        .watch_model(&st.step)
        .layout()
        .cloned_or_else(String::new);
    let (effective_step, step_valid) = parse_step(&step_text);

    let inc_cmd = msg.cmd(Msg::Inc);
    let dec_cmd = msg.cmd(Msg::Dec);
    let reset_cmd = msg.cmd(Msg::Reset);
    let step_1_cmd = msg.cmd(Msg::SetStepPreset(1));
    let step_5_cmd = msg.cmd(Msg::SetStepPreset(5));
    let step_10_cmd = msg.cmd(Msg::SetStepPreset(10));

    let header = shadcn::CardHeader::new([
        shadcn::CardTitle::new("Hello Counter").into_element(cx),
        shadcn::CardDescription::new(
            "A minimal, product-ish demo using `fret` + shadcn/ui (Model + MVU messages).",
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let count_text = cx
        .text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(count.to_string()),
            style: Some(fret_core::TextStyle {
                size: Px(56.0),
                weight: fret_core::FontWeight::BOLD,
                ..Default::default()
            }),
            color: Some(theme.color_token("foreground")),
            align: fret_core::TextAlign::Start,
            wrap: fret_core::TextWrap::None,
            overflow: fret_core::TextOverflow::Clip,
        })
        .test_id(TEST_ID_COUNT);

    let step_badge = shadcn::Badge::new(format!("step: {effective_step}"))
        .variant(if step_valid {
            shadcn::BadgeVariant::Secondary
        } else {
            shadcn::BadgeVariant::Destructive
        })
        .into_element(cx);

    let step_help = cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::from(if step_valid {
            "Edit step, then press Enter to increment."
        } else {
            "Step must be a positive integer (using 1)."
        }),
        style: None,
        color: Some(theme.color_token("muted-foreground")),
        align: fret_core::TextAlign::Start,
        wrap: fret_core::TextWrap::Word,
        overflow: fret_core::TextOverflow::Clip,
    });

    let step_input = shadcn::Input::new(st.step.clone())
        .placeholder("Step (e.g. 1)")
        .submit_command(inc_cmd.clone())
        .into_element(cx)
        .a11y_role(SemanticsRole::TextField)
        .test_id(TEST_ID_STEP_INPUT);

    let presets = ui::h_flex(cx, |cx| {
        [
            shadcn::Button::new("1")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_click(step_1_cmd)
                .into_element(cx)
                .test_id(TEST_ID_STEP_1),
            shadcn::Button::new("5")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_click(step_5_cmd)
                .into_element(cx)
                .test_id(TEST_ID_STEP_5),
            shadcn::Button::new("10")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_click(step_10_cmd)
                .into_element(cx)
                .test_id(TEST_ID_STEP_10),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let step_row = ui::v_flex(cx, |_cx| [step_input, presets])
        .gap(Space::N2)
        .w_full()
        .into_element(cx);

    let actions = ui::h_flex(cx, |cx| {
        [
            shadcn::Button::new("−")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(dec_cmd)
                .into_element(cx)
                .a11y_role(SemanticsRole::Button)
                .test_id(TEST_ID_DEC),
            shadcn::Button::new("+")
                .variant(shadcn::ButtonVariant::Default)
                .on_click(inc_cmd)
                .into_element(cx)
                .a11y_role(SemanticsRole::Button)
                .test_id(TEST_ID_INC),
            shadcn::Button::new("Reset")
                .variant(shadcn::ButtonVariant::Ghost)
                .on_click(reset_cmd)
                .into_element(cx)
                .a11y_role(SemanticsRole::Button)
                .test_id(TEST_ID_RESET),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let content = shadcn::CardContent::new([
        ui::v_flex(cx, |_cx| [count_text, step_badge])
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        ui::v_flex(cx, |_cx| [step_row, step_help])
            .gap(Space::N2)
            .w_full()
            .into_element(cx),
        actions,
    ])
    .into_element(cx);

    let card = shadcn::Card::new([header, content])
        .ui()
        .w_full()
        .max_w(Px(420.0))
        .into_element(cx);

    ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [card])
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx)
    .into()
}
