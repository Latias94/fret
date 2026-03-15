pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let icon = icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));

    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_media(|cx| ui::children![cx; icon])
                        .variant(shadcn::EmptyMediaVariant::Icon),
                    shadcn::empty_title("No Projects Yet").test_id("ui-gallery-empty-demo-title"),
                    shadcn::empty_description(
                        "You haven't created any projects yet. Get started by creating your first project.",
                    ),
                ]
            })
            .test_id("ui-gallery-empty-demo-header"),
            shadcn::empty_content(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Create Project"),
                    shadcn::Button::new("Import Project").variant(shadcn::ButtonVariant::Outline),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full()),
            shadcn::Button::new("Learn more")
                .variant(shadcn::ButtonVariant::Link)
                .size(shadcn::ButtonSize::Sm),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-demo")
}
// endregion: example
