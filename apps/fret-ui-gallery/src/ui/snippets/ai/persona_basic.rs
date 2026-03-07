pub const SOURCE: &str = include_str!("persona_basic.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(move |cx| {
        vec![ui_ai::Persona::new(ui_ai::PersonaState::Listening)
            .variant(ui_ai::PersonaVariant::Opal)
            .into_element(cx)]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
