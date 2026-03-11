pub const SOURCE: &str = include_str!("nested.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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
            .test_id("ui-gallery-button-group-nested-voice-button")
            .icon(icon_id("lucide.audio-lines"))
            .into_element(cx),
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
            .into_element(cx),
    )
    .arrow(true)
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let input_group = shadcn::InputGroup::new(message_value).into_element_parts(cx, |_cx| {
        vec![
            shadcn::InputGroupPart::input(
                shadcn::InputGroupInput::new()
                    .a11y_label("Message")
                    .placeholder("Send a message...")
                    .test_id("ui-gallery-button-group-nested-control"),
            ),
            shadcn::InputGroupPart::addon(
                shadcn::InputGroupAddon::new([voice_tooltip])
                    .align(shadcn::InputGroupAddonAlign::InlineEnd)
                    .has_button(true),
            ),
        ]
    });

    let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .test_id("ui-gallery-button-group-nested-add-button")
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
