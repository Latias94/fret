pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    width: Option<Model<String>>,
    max_width: Option<Model<String>>,
    height: Option<Model<String>>,
    max_height: Option<Model<String>>,
}

fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement {
    ui::h_flex(move |_cx| [body])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
        .into_element(cx)
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

    let row = |cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<String>| {
        ui::h_flex(move |cx| {
            let label_cell = ui::h_row(move |cx| vec![ui::label(label).into_element(cx)])
                .layout(LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0())
                .justify_end()
                .items_center()
                .into_element(cx);

            let input = shadcn::Input::new(model)
                .size(fret_ui_kit::Size::Small)
                .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                .into_element(cx);

            vec![label_cell, input]
        })
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N4)
        .items_center()
        .into_element(cx)
    };

    let header = shadcn::PopoverHeader::new([
        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
        shadcn::PopoverDescription::new("Set the dimensions for the layer.").into_element(cx),
    ])
    .into_element(cx);

    let fields = ui::v_flex(move |cx| {
        vec![
            row(cx, "Width", width.clone()),
            row(cx, "Max. width", max_width.clone()),
            row(cx, "Height", height.clone()),
            row(cx, "Max. height", max_height.clone()),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx);

    let content = shadcn::PopoverContent::new([header, fields])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .test_id("ui-gallery-popover-demo-panel");

    let popover = shadcn::Popover::new_controllable(cx, None, false)
        .build(
            cx,
            shadcn::PopoverTrigger::build(
                shadcn::Button::new("Open popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-popover-demo-trigger"),
            ),
            content,
        )
        .test_id("ui-gallery-popover-demo-popover");

    centered(cx, popover).test_id("ui-gallery-popover-demo")
}
// endregion: example
