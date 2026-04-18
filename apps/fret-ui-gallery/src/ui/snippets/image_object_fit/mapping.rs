pub const SOURCE: &str = include_str!("mapping.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{ImageId, ViewportFit};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn image_cell(
    cx: &mut AppComponentCx<'_>,
    label: &'static str,
    source: Option<ImageId>,
    fit: ViewportFit,
) -> impl UiChild + use<> {
    let test_id = format!("ui-gallery-image-object-fit-cell-{:?}", fit).to_lowercase();
    let image = shadcn::MediaImage::maybe(source)
        .fit(fit)
        .loading(true)
        .refine_style(ChromeRefinement::default().rounded(Radius::Md))
        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
        .into_element(cx)
        .test_id(test_id);

    ui::v_stack(move |cx| {
        ui::children![
            cx;
            cx.text(label),
            image,
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default())
}

fn row(
    cx: &mut AppComponentCx<'_>,
    title: &'static str,
    image: Option<ImageId>,
) -> impl UiChild + use<> {
    let stretch = image_cell(cx, "Stretch", image.clone(), ViewportFit::Stretch);
    let contain = image_cell(cx, "Contain", image.clone(), ViewportFit::Contain);
    let cover = image_cell(cx, "Cover", image, ViewportFit::Cover);
    let grid = fret_ui_kit::ui::h_flex(move |cx| {
        ui::children![
            cx;
            stretch,
            contain,
            cover,
        ]
    })
    .gap(Space::N4)
    .wrap()
    .w_full()
    .items_start();

    ui::v_flex(move |cx| {
        ui::children![
            cx;
            cx.text(title),
            grid,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let square_image = super::square_image(cx);
    let wide_image = super::wide_image(cx);
    let tall_image = super::tall_image(cx);

    ui::v_flex(|cx| {
        ui::children![
            cx;
            row(cx, "Wide source -> fixed 160x96", wide_image.clone()),
            row(cx, "Tall source -> fixed 160x96", tall_image.clone()),
            row(cx, "Square source -> fixed 160x96", square_image.clone()),
        ]
    })
    .gap(Space::N6)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-image-object-fit-mapping")
}
// endregion: example
