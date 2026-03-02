// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    width: Option<Model<String>>,
    height: Option<Model<String>>,
}

fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_center(),
        move |_cx| [body],
    )
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let width = match state.width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(Models::default, |st| st.width = Some(model.clone()));
            model
        }
    };
    let height = match state.height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(Models::default, |st| st.height = Some(model.clone()));
            model
        }
    };

    let popover = shadcn::Popover::new_controllable(cx, None, false)
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            |cx| {
                let trigger = shadcn::Button::new("Open Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
                    .test_id("ui-gallery-popover-with-form-trigger");
                shadcn::PopoverTrigger::new(trigger).into_element(cx)
            },
            |cx| {
                shadcn::PopoverContent::new([
                    shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::FieldGroup::new([
                        shadcn::Field::new([
                            shadcn::FieldLabel::new("Width")
                                .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                .into_element(cx),
                            shadcn::Input::new(width.clone())
                                .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                .into_element(cx),
                        ])
                        .orientation(shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                        shadcn::Field::new([
                            shadcn::FieldLabel::new("Height")
                                .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                .into_element(cx),
                            shadcn::Input::new(height.clone())
                                .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                .into_element(cx),
                        ])
                        .orientation(shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                    ])
                    .gap(Space::N4)
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_px(Px(256.0)))
                .into_element(cx)
                .test_id("ui-gallery-popover-with-form-panel")
            },
        )
        .test_id("ui-gallery-popover-with-form-popover");

    centered(cx, popover).test_id("ui-gallery-popover-with-form")
}
// endregion: example
