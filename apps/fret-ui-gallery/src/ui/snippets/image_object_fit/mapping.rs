pub const SOURCE: &str = include_str!("mapping.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::ImageId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let square_image = super::square_image(cx);
    let wide_image = super::wide_image(cx);
    let tall_image = super::tall_image(cx);

    let image_cell = |cx: &mut UiCx<'_>,
                      label: &'static str,
                      source: Option<ImageId>,
                      fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::maybe(source)
            .fit(fit)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
            .into_element(cx)
            .test_id(format!("ui-gallery-image-object-fit-cell-{:?}", fit).to_lowercase());

        ui::v_stack(|_cx| vec![label, image])
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default())
            .into_element(cx)
    };

    let row = |cx: &mut UiCx<'_>, title: &'static str, image: Option<ImageId>| -> AnyElement {
        let stretch = image_cell(
            cx,
            "Stretch",
            image.clone(),
            fret_core::ViewportFit::Stretch,
        );
        let contain = image_cell(
            cx,
            "Contain",
            image.clone(),
            fret_core::ViewportFit::Contain,
        );
        let cover = image_cell(cx, "Cover", image, fret_core::ViewportFit::Cover);

        let grid = fret_ui_kit::ui::h_flex(|_cx| vec![stretch, contain, cover])
            .gap(Space::N4)
            .wrap()
            .w_full()
            .items_start()
            .into_element(cx);

        ui::v_flex(move |cx| vec![cx.text(title), grid])
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
    };

    ui::v_flex(|cx| {
        vec![
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
