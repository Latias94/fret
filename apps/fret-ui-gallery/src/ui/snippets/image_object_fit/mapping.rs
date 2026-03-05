pub const SOURCE: &str = include_str!("mapping.rs");

// region: example
use fret_core::ImageId;
use fret_ui::Theme;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
) -> AnyElement {
    let image_cell = |cx: &mut ElementContext<'_, H>,
                      label: &'static str,
                      source: Model<Option<ImageId>>,
                      fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::model(source)
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

    let row = |cx: &mut ElementContext<'_, H>,
               title: &'static str,
               image: Model<Option<ImageId>>|
     -> AnyElement {
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
            row(cx, "Wide source → fixed 160×96", wide_image.clone()),
            row(cx, "Tall source → fixed 160×96", tall_image.clone()),
            row(cx, "Square source → fixed 160×96", square_image.clone()),
        ]
    })
    .gap(Space::N6)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-image-object-fit-mapping")
}
// endregion: example
