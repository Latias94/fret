pub const SOURCE: &str = include_str!("custom_input.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui_kit::ColorRef;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn custom_textarea_control(cx: &mut UiCx<'_>, value: Model<String>) -> AnyElement {
    shadcn::Textarea::new(value)
        .a11y_label("Autoresize textarea")
        .placeholder("Autoresize textarea...")
        .test_id("ui-gallery-input-group-custom-input-control")
        .min_height(Px(64.0))
        .resizable(false)
        .stable_line_boxes(false)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .refine_style(
            ChromeRefinement::default()
                .border_width(Px(0.0))
                .shadow_none()
                .bg(ColorRef::Color(Color::TRANSPARENT))
                .px_3()
                .py_2p5(),
        )
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let control = custom_textarea_control(cx, value.clone());

    shadcn::InputGroup::new(value)
        .custom_textarea(control)
        .block_end([shadcn::InputGroupButton::new("Submit")
            .variant(shadcn::ButtonVariant::Default)
            .size(shadcn::InputGroupButtonSize::Sm)
            .refine_layout(LayoutRefinement::default().ml_auto())
            .into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-custom-input")
        .into_element(cx)
}
// endregion: example
