pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    ui::h_flex(move |cx| ui::children![cx; body])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
}

fn row<H: UiHost>(label: &'static str, model: Model<String>) -> impl IntoUiElement<H> + use<H> {
    let label_cell = ui::h_row(move |cx| ui::children![cx; ui::label(label)])
        .layout(LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0())
        .justify_end()
        .items_center();

    let input = shadcn::Input::new(model)
        .size(fret_ui_kit::Size::Small)
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0());

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

    let header = shadcn::PopoverHeader::new(ui::children![
        cx;
        shadcn::PopoverTitle::new("Dimensions"),
        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
    ]);

    let fields = ui::v_flex(move |cx| {
        ui::children![
            cx;
            row("Width", width.clone()),
            row("Max. width", max_width.clone()),
            row("Height", height.clone()),
            row("Max. height", max_height.clone())
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full());

    let content = shadcn::PopoverContent::new(ui::children![cx; header, fields])
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
