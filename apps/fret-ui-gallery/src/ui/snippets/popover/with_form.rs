pub const SOURCE: &str = include_str!("with_form.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    ui::h_flex(move |cx| ui::children![cx; body])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = cx.local_model_keyed("width", || String::from("100%"));
    let height = cx.local_model_keyed("height", || String::from("25px"));

    let content = shadcn::PopoverContent::new(ui::children![
        cx;
        shadcn::PopoverHeader::new(ui::children![
            cx;
            shadcn::PopoverTitle::new("Dimensions"),
            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
        ]),
        shadcn::FieldGroup::new(ui::children![
            cx;
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("Width")
                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0))),
                shadcn::Input::new(width.clone())
                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            ])
            .orientation(shadcn::FieldOrientation::Horizontal),
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("Height")
                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0))),
                shadcn::Input::new(height.clone())
                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
        ])
        .gap(Space::N4)
    ])
    .refine_layout(LayoutRefinement::default().w_px(Px(256.0)))
    .test_id("ui-gallery-popover-with-form-panel");

    let popover = shadcn::Popover::new(
        cx,
        shadcn::PopoverTrigger::build(
            shadcn::Button::new("Open Popover")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-popover-with-form-trigger"),
        ),
        content,
    )
    .align(shadcn::PopoverAlign::Start)
    .into_element(cx)
    .test_id("ui-gallery-popover-with-form-popover");

    centered(popover)
        .into_element(cx)
        .test_id("ui-gallery-popover-with-form")
}
// endregion: example
