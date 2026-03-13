pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let pressed = cx.local_model_keyed("pressed", || false);
    let pressed_now = cx
        .get_model_cloned(&pressed, Invalidation::Paint)
        .unwrap_or(false);
    let control_id = ControlId::from("ui-gallery-toggle-label");

    ui::v_flex(|cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Bookmark")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-toggle-label-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Click the label to toggle the control.")
                        .for_control(control_id.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::toggle(cx, pressed.clone(), |cx| {
                    ui::children![
                        cx;
                        shadcn::raw::icon::icon(cx, IconId::new_static("lucide.bookmark")),
                        ui::text("Bookmark")
                    ]
                })
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Sm)
                .control_id(control_id)
                .a11y_label("Toggle bookmark label association")
                .into_element(cx)
                .test_id("ui-gallery-toggle-label-control"),
            ])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx),
            shadcn::raw::typography::muted(format!("Pressed: {pressed_now}"))
                .into_element(cx)
                .test_id("ui-gallery-toggle-label-state"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-toggle-label")
}
// endregion: example
