pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Axis;
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn box_group<H: UiHost, B>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    body: B,
) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Lg),
        layout,
    );
    ui::container_props(props, move |cx| [body.into_element(cx)])
}

fn panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    height: Option<Px>,
) -> impl IntoUiElement<H> + use<H> {
    let layout = match height {
        Some(h) => LayoutRefinement::default().w_full().h_px(h),
        None => LayoutRefinement::default().w_full().h_full(),
    };

    let body = ui::v_flex(move |cx| vec![shadcn::raw::typography::small(label).into_element(cx)])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx);

    let props =
        decl_style::container_props(cx.theme(), ChromeRefinement::default().p(Space::N6), layout);
    ui::container_props(props, move |_cx| [body])
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let h_fractions = cx.local_model_keyed("ui-gallery-resizable-demo-h-fractions", || vec![0.5, 0.5]);
    let v_fractions =
        cx.local_model_keyed("ui-gallery-resizable-demo-v-fractions", || vec![0.25, 0.75]);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let nested_vertical = shadcn::resizable_panel_group(cx, v_fractions, |cx| {
        [
            shadcn::ResizablePanel::new([panel(cx, "Two", None).into_element(cx)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([panel(cx, "Three", None).into_element(cx)]).into(),
        ]
    })
    .axis(Axis::Vertical)
    .test_id_prefix("ui-gallery-resizable-demo.nested-vertical")
    .into_element(cx);

    let group = shadcn::resizable_panel_group(cx, h_fractions, |cx| {
        [
            shadcn::ResizablePanel::new([panel(cx, "One", Some(Px(200.0))).into_element(cx)])
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([nested_vertical]).into(),
        ]
    })
    .axis(Axis::Horizontal)
    .test_id_prefix("ui-gallery-resizable-demo")
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:resizable-panels")
            .test_id("ui-gallery-resizable-panels"),
    );

    box_group(
        cx,
        max_w_sm
            .clone()
            .merge(LayoutRefinement::default().h_px(Px(320.0))),
        group,
    )
    .into_element(cx)
    .test_id("ui-gallery-resizable-demo")
}
// endregion: example
