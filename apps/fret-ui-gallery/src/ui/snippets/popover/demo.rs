pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn centered<B>(body: B) -> impl UiChild + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| ui::children![cx; body])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
}

fn row(
    label: &'static str,
    model: Model<String>,
    input_test_id: &'static str,
) -> impl UiChild + use<> {
    let label_cell = ui::h_row(move |cx| ui::children![cx; ui::label(label)])
        .layout(LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0())
        .justify_end()
        .items_center();

    let input = shadcn::Input::new(model)
        .size(fret_ui_kit::Size::Small)
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .test_id(input_test_id);

    ui::h_flex(move |cx| ui::children![cx; label_cell, input])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N4)
        .items_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let width = cx.local_model_keyed("width", || String::from("100%"));
    let max_width = cx.local_model_keyed("max_width", || String::from("300px"));
    let height = cx.local_model_keyed("height", || String::from("25px"));
    let max_height = cx.local_model_keyed("max_height", || String::from("none"));
    let content = shadcn::PopoverContent::build(cx, move |cx| {
        let header = shadcn::PopoverHeader::new(ui::children![
            cx;
            shadcn::PopoverTitle::new("Dimensions")
                .into_element(cx)
                .test_id("ui-gallery-popover-demo-title"),
            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                .into_element(cx)
                .test_id("ui-gallery-popover-demo-description")
        ])
        .into_element(cx)
        .test_id("ui-gallery-popover-demo-header");

        let fields = ui::v_flex(move |cx| {
            ui::children![
                cx;
                row("Width", width.clone(), "ui-gallery-popover-demo-width-input"),
                row("Max. width", max_width.clone(), "ui-gallery-popover-demo-max-width-input"),
                row("Height", height.clone(), "ui-gallery-popover-demo-height-input"),
                row("Max. height", max_height.clone(), "ui-gallery-popover-demo-max-height-input")
            ]
        })
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full());

        ui::children![cx; header, fields]
    })
    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
    .test_id("ui-gallery-popover-demo-panel");

    let popover = shadcn::Popover::new(
        cx,
        shadcn::PopoverTrigger::build(
            shadcn::Button::new("Open popover")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-popover-demo-trigger"),
        ),
        content,
    )
    .into_element(cx)
    .test_id("ui-gallery-popover-demo-popover");

    centered(popover)
        .into_element(cx)
        .test_id("ui-gallery-popover-demo")
}
// endregion: example
