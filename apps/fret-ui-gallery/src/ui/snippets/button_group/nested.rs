pub const SOURCE: &str = include_str!("nested.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    message_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let message_value = cx.with_state(Models::default, |st| st.message_value.clone());
    let message_value = match message_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.message_value = Some(model.clone()));
            model
        }
    };

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    let voice_tooltip = shadcn::Tooltip::new(
        shadcn::InputGroupButton::new("")
            .a11y_label("Voice Mode")
            .size(shadcn::InputGroupButtonSize::IconSm)
            .icon(icon_id("lucide.audio-lines"))
            .into_element(cx),
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
            .into_element(cx),
    )
    .arrow(true)
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let input_group = shadcn::InputGroup::new(message_value)
        .a11y_label("Message")
        .trailing([voice_tooltip])
        .trailing_has_button(true);

    let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .icon(icon_id("lucide.plus"))
        .into()]);

    let message = shadcn::ButtonGroup::new([input_group.into()])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0());

    shadcn::ButtonGroup::new([plus.into(), message.into()])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(560.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-button-group-nested")
}

// endregion: example
