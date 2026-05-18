pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::toggle_group_single_uncontrolled(cx, Some("list"), |cx| {
        [
            shadcn::ToggleGroupItem::new(
                "list",
                [
                    shadcn::raw::icon::icon(cx, IconId::new_static("lucide.list")),
                    decl_text::text_button_label(cx, "List"),
                ],
            )
            .a11y_label("Toggle list"),
            shadcn::ToggleGroupItem::new(
                "grid",
                [
                    shadcn::raw::icon::icon(cx, IconId::new_static("lucide.grid-2x2")),
                    decl_text::text_button_label(cx, "Grid"),
                ],
            )
            .a11y_label("Toggle grid"),
            shadcn::ToggleGroupItem::new(
                "cards",
                [
                    shadcn::raw::icon::icon(cx, IconId::new_static("lucide.layout-panel-top")),
                    decl_text::text_button_label(cx, "Cards"),
                ],
            )
            .a11y_label("Toggle cards"),
        ]
    })
    .variant(shadcn::ToggleVariant::Outline)
    .into_element(cx)
    .test_id("ui-gallery-toggle-group-children")
}
// endregion: example
