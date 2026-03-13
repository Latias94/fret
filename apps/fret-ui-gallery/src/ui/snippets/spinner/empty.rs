pub const SOURCE: &str = include_str!("empty.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_media(|cx| ui::children![cx; shadcn::Spinner::new(),])
                        .variant(fret_ui_shadcn::empty::EmptyMediaVariant::Icon),
                    shadcn::empty_title("Processing your request"),
                    shadcn::empty_description(
                        "Please wait while we process your request. Do not refresh the page.",
                    ),
                ]
            }),
            shadcn::empty_content(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full()),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-spinner-empty")
}

// endregion: example
