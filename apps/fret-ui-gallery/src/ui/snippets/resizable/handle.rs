pub const SOURCE: &str = include_str!("handle.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Axis;
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
) -> impl IntoUiElement<H> + use<H> {
    let body = ui::v_flex(move |cx| vec![shadcn::raw::typography::small(label).into_element(cx)])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx);

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().p(Space::N6),
        LayoutRefinement::default().w_full().h_full(),
    );
    ui::container_props(props, move |_cx| [body])
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let fractions =
        cx.local_model_keyed("ui-gallery-resizable-handle-fractions", || vec![0.25, 0.75]);

    let group = shadcn::resizable_panel_group(cx, fractions, |cx| {
        [
            shadcn::ResizablePanel::new([panel(cx, "Sidebar").into_element(cx)]).into(),
            shadcn::ResizableHandle::new().with_handle(true).into(),
            shadcn::ResizablePanel::new([panel(cx, "Content").into_element(cx)]).into(),
        ]
    })
    .axis(Axis::Horizontal)
    .test_id_prefix("ui-gallery-resizable-handle")
    .into_element(cx);

    box_group(
        cx,
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(448.0))
            .h_px(Px(200.0)),
        group,
    )
    .into_element(cx)
    .test_id("ui-gallery-resizable-handle")
}
// endregion: example
