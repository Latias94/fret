pub const SOURCE: &str = include_str!("shimmer_typography_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{FontId, FontWeight, Px, TextStyle};
use fret_ui_ai as ui_ai;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};

fn item<B>(label: &'static str, el: B) -> impl UiChild + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::v_stack(move |cx| {
        let theme = fret_ui::Theme::global(&*cx.app).snapshot();
        let muted = ColorRef::Color(typography::muted_foreground_color(&theme));
        vec![
            ui::text(label).text_sm().text_color(muted).into_element(cx),
            el.into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_center()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let theme = fret_ui::Theme::global(&*cx.app).snapshot();

    let explicit_style = TextStyle {
        font: FontId::ui(),
        size: Px(18.0),
        weight: FontWeight::SEMIBOLD,
        line_height: Some(Px(24.0)),
        ..Default::default()
    };

    let explicit = ui_ai::Shimmer::new("Explicit typography override")
        .text_style(explicit_style)
        .into_element(cx);
    let inherited = typography::scope_text_style_with_color(
        ui_ai::Shimmer::new("Inherited shimmer from subtree typography")
            .use_resolved_passive_text()
            .into_element(cx),
        typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        ),
        typography::muted_foreground_color(&theme),
    );

    ui::v_flex(move |cx| {
        ui::children![
            cx;
            item("Explicit `TextStyle` override", explicit),
            item("Inherited subtree typography", inherited),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N6)
    .into_element(cx)
    .test_id("ui-ai-shimmer-typography-root")
}
// endregion: example
