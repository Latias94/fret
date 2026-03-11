pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    message_value: Option<Model<String>>,
    voice_enabled: Option<Model<bool>>,
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

    let voice_enabled = cx.with_state(Models::default, |st| st.voice_enabled.clone());
    let voice_enabled = match voice_enabled {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.voice_enabled = Some(model.clone()));
            model
        }
    };

    let voice_enabled_now = cx
        .get_model_cloned(&voice_enabled, Invalidation::Paint)
        .unwrap_or(false);
    let placeholder = if voice_enabled_now {
        "Record and send audio..."
    } else {
        "Send a message..."
    };

    let voice_button = {
        let mut button = shadcn::InputGroupButton::new("")
            .a11y_label("Voice Mode")
            .size(shadcn::InputGroupButtonSize::IconXs)
            .test_id("ui-gallery-button-group-input-group-voice-button")
            .icon(IconId::new_static("lucide.audio-lines"))
            .toggle_model(voice_enabled.clone());

        if voice_enabled_now {
            let theme = Theme::global(&*cx.app).snapshot();
            button = button.refine_style(
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_token("accent")))
                    .text_color(ColorRef::Color(theme.color_token("accent.foreground"))),
            );
        }

        button.into_element(cx)
    };

    let voice_tooltip = shadcn::Tooltip::new(
        voice_button,
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
            .into_element(cx),
    )
    .arrow(true)
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let group = shadcn::InputGroup::new(message_value)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::InputGroupPart::input(
                    shadcn::InputGroupInput::new()
                        .a11y_label("Message")
                        .placeholder(placeholder)
                        .test_id("ui-gallery-button-group-input-group-control")
                        .disabled(voice_enabled_now),
                ),
                shadcn::InputGroupPart::addon(
                    shadcn::InputGroupAddon::new([voice_tooltip])
                        .align(shadcn::InputGroupAddonAlign::InlineEnd)
                        .has_button(true),
                ),
            ]
        });

    let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
        .a11y_label("Add")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .test_id("ui-gallery-button-group-input-group-add-button")
        .icon(IconId::new_static("lucide.plus"))
        .into()]);

    let message = shadcn::ButtonGroup::new([group.into()])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0());

    shadcn::ButtonGroup::new([plus.into(), message.into()])
        .radius_override(Radius::Full)
        .refine_layout(LayoutRefinement::default().w_px(Px(420.0)).min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-button-group-input-group")
}

// endregion: example
