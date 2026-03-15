pub const SOURCE: &str = include_str!("persona_custom_visual.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(move |cx| {
        vec![
            ui_ai::Persona::new(ui_ai::PersonaState::Speaking)
                .variant(ui_ai::PersonaVariant::Command)
                .size(Px(112.0))
                .show_label(true)
                .into_element_with_children(cx, |cx, controller| {
                    vec![
                        ui::v_flex(move |cx| {
                            vec![
                                decl_icon::icon(cx, IconId::new_static("lucide.bot")),
                                cx.text(controller.variant.label()),
                            ]
                        })
                        .gap(Space::N1)
                        .items_center()
                        .into_element(cx),
                    ]
                }),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
