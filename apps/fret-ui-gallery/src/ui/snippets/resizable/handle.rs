pub const SOURCE: &str = include_str!("handle.rs");

// region: example
use fret_core::Axis;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn box_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    body: AnyElement,
) -> AnyElement {
    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Lg),
        layout,
    );
    cx.container(props, move |_cx| [body])
}

fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    let body = ui::v_flex(move |cx| vec![shadcn::raw::typography::small(cx, label)])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx);

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().p(Space::N6),
        LayoutRefinement::default().w_full().h_full(),
    );
    cx.container(props, move |_cx| [body])
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let fractions = cx.local_model_keyed("fractions", || vec![0.25, 0.75]);

    let group = shadcn::ResizablePanelGroup::new(fractions)
        .axis(Axis::Horizontal)
        .test_id_prefix("ui-gallery-resizable-handle")
        .entries([
            shadcn::ResizablePanel::new([panel(cx, "Sidebar")]).into(),
            shadcn::ResizableHandle::new().with_handle(true).into(),
            shadcn::ResizablePanel::new([panel(cx, "Content")]).into(),
        ])
        .into_element(cx);

    box_group(
        cx,
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(448.0))
            .h_px(Px(200.0)),
        group,
    )
    .test_id("ui-gallery-resizable-handle")
}
// endregion: example
