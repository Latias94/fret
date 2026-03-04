pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    width: Option<Model<String>>,
    max_width: Option<Model<String>>,
    height: Option<Model<String>>,
    max_height: Option<Model<String>>,
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
    let max_width = match state.max_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("300px"));
            cx.with_state(Models::default, |st| st.max_width = Some(model.clone()));
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
    let max_height = match state.max_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("none"));
            cx.with_state(Models::default, |st| st.max_height = Some(model.clone()));
            model
        }
    };

    let popover = shadcn::Popover::new_controllable(cx, None, false)
        .into_element(
            cx,
            |cx| {
                let trigger = shadcn::Button::new("Open popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
                    .test_id("ui-gallery-popover-demo-trigger");
                shadcn::PopoverTrigger::new(trigger).into_element(cx)
            },
            |cx| {
                let row = |cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<_>| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4)
                            .items_center(),
                        move |cx| {
                            vec![
                                stack::hstack(
                                    cx,
                                    stack::HStackProps::default()
                                        .layout(
                                            LayoutRefinement::default()
                                                .w_px(Px(96.0))
                                                .flex_shrink_0(),
                                        )
                                        .justify_end()
                                        .items_center(),
                                    move |cx| vec![ui::label(cx, label).into_element(cx)],
                                ),
                                shadcn::Input::new(model)
                                    .size(fret_ui_kit::Size::Small)
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let header = shadcn::PopoverHeader::new([
                    shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                    shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                        .into_element(cx),
                ])
                .into_element(cx);

                let fields = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    move |cx| {
                        vec![
                            row(cx, "Width", width.clone()),
                            row(cx, "Max. width", max_width.clone()),
                            row(cx, "Height", height.clone()),
                            row(cx, "Max. height", max_height.clone()),
                        ]
                    },
                );

                shadcn::PopoverContent::new([header, fields])
                    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                    .into_element(cx)
                    .test_id("ui-gallery-popover-demo-panel")
            },
        )
        .test_id("ui-gallery-popover-demo-popover");

    centered(cx, popover).test_id("ui-gallery-popover-demo")
}
// endregion: example
