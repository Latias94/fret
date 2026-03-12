pub const SOURCE: &str = include_str!("vertical.rs");

// region: example
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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let fractions = cx.local_model_keyed("fractions", || vec![0.25, 0.75]);

    let group = shadcn::ResizablePanelGroup::new(fractions)
        .axis(Axis::Vertical)
        .test_id_prefix("ui-gallery-resizable-vertical")
        .entries([
            shadcn::ResizablePanel::new([panel(cx, "Header").into_element(cx)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([panel(cx, "Content").into_element(cx)]).into(),
        ])
        .into_element(cx);

    box_group(
        cx,
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(384.0))
            .h_px(Px(200.0)),
        group,
    )
    .into_element(cx)
    .test_id("ui-gallery-resizable-vertical")
}
// endregion: example
