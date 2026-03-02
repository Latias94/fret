pub const SOURCE: &str = include_str!("label_card.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    description: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let description = cx.with_state(Models::default, |st| st.description.clone());
    let description = match description {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.description = Some(model.clone()));
            model
        }
    };

    let blue = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0x3B_82_F6));
    let style = shadcn::switch::SwitchStyle::default().track_background(
        fret_ui_kit::WidgetStateProperty::new(None)
            .when(fret_ui_kit::WidgetStates::SELECTED, Some(blue)),
    );

    shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldTitle::new("Share across devices").into_element(cx),
            shadcn::FieldDescription::new(
                "Focus is shared across devices, and turns off when you leave the app.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(description)
            .a11y_label("Share across devices")
            .style(style)
            .test_id("ui-gallery-switch-label-card-toggle")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_style(
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Lg)
            .p(Space::N4),
    )
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-label-card")
}

// endregion: example
