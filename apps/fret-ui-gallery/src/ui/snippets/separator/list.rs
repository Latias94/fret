pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    value: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    ui::h_flex(move |cx| {
        vec![
            shadcn::raw::typography::small(label).into_element(cx),
            shadcn::raw::typography::muted(value).into_element(cx),
        ]
    })
    .justify_between()
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            row(cx, "Item 1", "Value 1").into_element(cx),
            shadcn::Separator::new().into_element(cx),
            row(cx, "Item 2", "Value 2").into_element(cx),
            shadcn::Separator::new().into_element(cx),
            row(cx, "Item 3", "Value 3").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(384.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-separator-list")
}
// endregion: example
