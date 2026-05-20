pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        let bookmark_children = vec![
            shadcn::raw::icon::icon(cx, IconId::new_static("lucide.bookmark")),
            decl_text::text_button_label(cx, "Bookmark"),
        ];
        let underline_children = vec![
            shadcn::raw::icon::icon(cx, IconId::new_static("lucide.underline")),
            decl_text::text_button_label(cx, "Underline"),
        ];

        vec![
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Sm)
                .a11y_label("Toggle bookmark children")
                .children(bookmark_children)
                .into_element(cx)
                .test_id("ui-gallery-toggle-children-bookmark"),
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Sm)
                .a11y_label("Toggle underline children")
                .children(underline_children)
                .into_element(cx)
                .test_id("ui-gallery-toggle-children-underline"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-children")
}
// endregion: example
