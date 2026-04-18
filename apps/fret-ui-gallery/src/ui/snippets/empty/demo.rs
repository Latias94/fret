pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_icons::IconId;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let icon = icon::icon(cx, IconId::new_static("lucide.folder-code"));
    let actions = ui::h_row(|cx| {
        vec![
            shadcn::Button::new("Create Project")
                .test_id("ui-gallery-empty-demo-create-project")
                .into_element(cx),
            shadcn::Button::new("Import Project")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-empty-demo-import-project")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-empty-demo-actions");

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
            shadcn::empty_content(|cx| ui::children![cx; actions])
                .refine_layout(LayoutRefinement::default().w_full()),
            shadcn::Button::new("Learn More")
                .variant(shadcn::ButtonVariant::Link)
                .size(shadcn::ButtonSize::Sm)
                .render(shadcn::ButtonRender::Link {
                    href: Arc::from("https://example.com/projects"),
                    target: None,
                    rel: None,
                })
                .trailing_icon(IconId::new_static("lucide.arrow-up-right"))
                .on_activate(Arc::new(|_host, _acx, _reason| {}))
                .test_id("ui-gallery-empty-demo-learn-more"),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-demo")
}
// endregion: example
