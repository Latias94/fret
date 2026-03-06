pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    pressed: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let pressed = cx.with_state(Models::default, |st| st.pressed.clone());
    let pressed = pressed.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.pressed = Some(model.clone()));
        model
    });
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
                shadcn::Toggle::new(pressed.clone())
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Sm)
                    .control_id(control_id)
                    .a11y_label("Toggle bookmark label association")
                    .leading_icon(IconId::new_static("lucide.bookmark"))
                    .label("Bookmark")
                    .into_element(cx)
                    .test_id("ui-gallery-toggle-label-control"),
            ])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx),
            shadcn::typography::muted(cx, format!("Pressed: {pressed_now}"))
                .test_id("ui-gallery-toggle-label-state"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-toggle-label")
}
// endregion: example
