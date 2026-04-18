pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn icon_item(value: &'static str, label: &'static str) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::icon(
        value,
        IconId::new_static(match value {
            "bold" => "lucide.bold",
            "italic" => "lucide.italic",
            _ => "lucide.underline",
        }),
    )
    .a11y_label(label)
}

fn group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: shadcn::ToggleSize,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    match size {
        shadcn::ToggleSize::Sm => {
            shadcn::ToggleGroup::single_uncontrolled(Option::<&'static str>::None)
                .size(shadcn::ToggleSize::Sm)
                .items([
                    icon_item("bold", "Toggle bold"),
                    icon_item("italic", "Toggle italic"),
                    icon_item("strikethrough", "Toggle strikethrough"),
                ])
        }
        shadcn::ToggleSize::Lg => {
            shadcn::ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())
                .size(shadcn::ToggleSize::Lg)
                .items([
                    icon_item("bold", "Toggle bold"),
                    icon_item("italic", "Toggle italic"),
                    icon_item("strikethrough", "Toggle strikethrough"),
                ])
        }
        _ => shadcn::ToggleGroup::single_uncontrolled(Option::<&'static str>::None)
            .size(size)
            .items([
                icon_item("bold", "Toggle bold"),
                icon_item("italic", "Toggle italic"),
                icon_item("strikethrough", "Toggle strikethrough"),
            ]),
    }
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let small = group(cx, shadcn::ToggleSize::Sm).into_element(cx);
    let large = group(cx, shadcn::ToggleSize::Lg).into_element(cx);

    ui::v_stack(move |_cx| vec![small, large])
        .gap(Space::N4)
        .items_start()
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-size")
}
// endregion: example
