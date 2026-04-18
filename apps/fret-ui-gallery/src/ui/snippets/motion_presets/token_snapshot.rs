pub const SOURCE: &str = include_str!("token_snapshot.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Theme;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn fmt_bezier(b: fret_ui::theme::CubicBezier) -> String {
    format!("{:.2}, {:.2}, {:.2}, {:.2}", b.x1, b.y1, b.x2, b.y2)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let shell_layout = LayoutRefinement::default()
        .w_full()
        .max_w(Px(760.0))
        .min_w_0();

    let theme = Theme::global(&*cx.app);
    let rows = [
        (
            "duration.shadcn.motion.100",
            theme
                .duration_ms_token("duration.shadcn.motion.100")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.200",
            theme
                .duration_ms_token("duration.shadcn.motion.200")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.300",
            theme
                .duration_ms_token("duration.shadcn.motion.300")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.500",
            theme
                .duration_ms_token("duration.shadcn.motion.500")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.overlay.open",
            theme
                .duration_ms_token("duration.shadcn.motion.overlay.open")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.overlay.close",
            theme
                .duration_ms_token("duration.shadcn.motion.overlay.close")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.collapsible.toggle",
            theme
                .duration_ms_token("duration.shadcn.motion.collapsible.toggle")
                .to_string(),
        ),
        (
            "easing.shadcn.motion.overlay",
            fmt_bezier(theme.easing_token("easing.shadcn.motion.overlay")),
        ),
        (
            "easing.shadcn.motion.collapsible.toggle",
            fmt_bezier(theme.easing_token("easing.shadcn.motion.collapsible.toggle")),
        ),
        (
            "duration.shadcn.motion.toast.enter",
            theme
                .duration_ms_token("duration.shadcn.motion.toast.enter")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.toast.exit",
            theme
                .duration_ms_token("duration.shadcn.motion.toast.exit")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.toast.stack.shift",
            theme
                .duration_ms_token("duration.shadcn.motion.toast.stack.shift")
                .to_string(),
        ),
        (
            "duration.shadcn.motion.toast.stack.shift.stagger",
            theme
                .duration_ms_token("duration.shadcn.motion.toast.stack.shift.stagger")
                .to_string(),
        ),
        (
            "easing.shadcn.motion.toast.stack.shift",
            fmt_bezier(theme.easing_token("easing.shadcn.motion.toast.stack.shift")),
        ),
        (
            "duration.shadcn.motion.spring.drawer.settle",
            theme
                .duration_ms_token("duration.shadcn.motion.spring.drawer.settle")
                .to_string(),
        ),
        (
            "number.shadcn.motion.spring.drawer.settle.bounce",
            format!(
                "{:.2}",
                theme.number_token("number.shadcn.motion.spring.drawer.settle.bounce")
            ),
        ),
        (
            "duration.motion.presence.enter",
            theme
                .duration_ms_token("duration.motion.presence.enter")
                .to_string(),
        ),
        (
            "duration.motion.collapsible.toggle",
            theme
                .duration_ms_token("duration.motion.collapsible.toggle")
                .to_string(),
        ),
        (
            "easing.motion.collapsible.toggle",
            fmt_bezier(theme.easing_token("easing.motion.collapsible.toggle")),
        ),
        (
            "duration.motion.layout.expand",
            theme
                .duration_ms_token("duration.motion.layout.expand")
                .to_string(),
        ),
        (
            "easing.motion.layout.expand",
            fmt_bezier(theme.easing_token("easing.motion.layout.expand")),
        ),
        (
            "duration.motion.stack.shift",
            theme
                .duration_ms_token("duration.motion.stack.shift")
                .to_string(),
        ),
        (
            "duration.motion.stack.shift.stagger",
            theme
                .duration_ms_token("duration.motion.stack.shift.stagger")
                .to_string(),
        ),
        (
            "easing.motion.stack.shift",
            fmt_bezier(theme.easing_token("easing.motion.stack.shift")),
        ),
        (
            "easing.motion.standard",
            fmt_bezier(theme.easing_token("easing.motion.standard")),
        ),
    ];

    let content = ui::v_flex(move |cx| {
        rows.into_iter()
            .map(|(key, value)| {
                ui::h_flex(move |cx| {
                    vec![
                        cx.text(key),
                        shadcn::Badge::new(value)
                            .variant(shadcn::BadgeVariant::Outline)
                            .into_element(cx),
                    ]
                })
                .layout(LayoutRefinement::default().w_full())
                .justify_between()
                .items_center()
                .gap(Space::N4)
                .into_element(cx)
            })
            .collect::<Vec<_>>()
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N3)
    .items_start()
    .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Token snapshot"),
                    shadcn::card_description(
                        "Current effective values for a small, shared set of motion tokens.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; content]),
        ]
    })
    .refine_layout(shell_layout)
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-token-snapshot")
}
// endregion: example
