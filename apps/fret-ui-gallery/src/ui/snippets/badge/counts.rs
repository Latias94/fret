pub const SOURCE: &str = include_str!("counts.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    row(|cx| {
        vec![
            shadcn::Badge::new("8")
                .label_font_monospace()
                .label_tabular_nums()
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .px(Space::N1),
                )
                .refine_layout(LayoutRefinement::default().h_px(Px(20.0)).min_w(Px(20.0)))
                .test_id("ui-gallery-badge-demo-count")
                .into_element(cx),
            shadcn::Badge::new("99")
                .variant(shadcn::BadgeVariant::Destructive)
                .label_font_monospace()
                .label_tabular_nums()
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .px(Space::N1),
                )
                .refine_layout(LayoutRefinement::default().h_px(Px(20.0)).min_w(Px(20.0)))
                .test_id("ui-gallery-badge-demo-count-destructive")
                .into_element(cx),
            shadcn::Badge::new("20+")
                .variant(shadcn::BadgeVariant::Outline)
                .label_font_monospace()
                .label_tabular_nums()
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .px(Space::N1),
                )
                .refine_layout(LayoutRefinement::default().h_px(Px(20.0)).min_w(Px(20.0)))
                .test_id("ui-gallery-badge-demo-count-outline")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-badge-counts")
}
// endregion: example
