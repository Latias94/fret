pub const SOURCE: &str = include_str!("persona_variants.rs");

// region: example
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let items = ui::h_flex(move |cx| {
        vec![
            ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                .variant(ui_ai::PersonaVariant::Obsidian)
                .show_label(true)
                .test_id("ui-ai-persona-variant-obsidian")
                .into_element(cx),
            ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                .variant(ui_ai::PersonaVariant::Mana)
                .show_label(true)
                .test_id("ui-ai-persona-variant-mana")
                .into_element(cx),
            ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                .variant(ui_ai::PersonaVariant::Opal)
                .show_label(true)
                .test_id("ui-ai-persona-variant-opal")
                .into_element(cx),
            ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                .variant(ui_ai::PersonaVariant::Halo)
                .show_label(true)
                .test_id("ui-ai-persona-variant-halo")
                .into_element(cx),
            ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                .variant(ui_ai::PersonaVariant::Glint)
                .show_label(true)
                .test_id("ui-ai-persona-variant-glint")
                .into_element(cx),
            ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                .variant(ui_ai::PersonaVariant::Command)
                .show_label(true)
                .test_id("ui-ai-persona-variant-command")
                .into_element(cx),
        ]
    })
    .gap(Space::N6)
    .wrap()
    .w_full()
    .items_center()
    .justify_center()
    .into_element(cx);

    let shell_props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .p(Space::N6),
            LayoutRefinement::default()
                .w_full()
                .max_w(MetricRef::Px(Px(900.0)))
                .min_w_0(),
        )
    });

    cx.container(shell_props, move |_cx| [items])
}
// endregion: example
