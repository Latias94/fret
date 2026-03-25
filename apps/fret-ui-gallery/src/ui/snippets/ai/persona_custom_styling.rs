pub const SOURCE: &str = include_str!("persona_custom_styling.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(move |cx| {
        vec![
            ui_ai::Persona::new(ui_ai::PersonaState::Thinking)
                .variant(ui_ai::PersonaVariant::Halo)
                .size(Px(256.0))
                .refine_layout(LayoutRefinement::default().min_w_0())
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .border_1()
                        .border_color(ColorRef::Token {
                            key: "border",
                            fallback: ColorFallback::ThemePanelBorder,
                        }),
                )
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
