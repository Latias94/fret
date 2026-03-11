pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Axis;
use fret_ui::element::SemanticsDecoration;
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

fn panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    height: Option<Px>,
) -> AnyElement {
    let layout = match height {
        Some(h) => LayoutRefinement::default().w_full().h_px(h),
        None => LayoutRefinement::default().w_full().h_full(),
    };

    let body = ui::v_flex(move |cx| vec![shadcn::raw::typography::small(cx, label)])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx);

    let props =
        decl_style::container_props(cx.theme(), ChromeRefinement::default().p(Space::N6), layout);
    cx.container(props, move |_cx| [body])
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> AnyElement {
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
        .axis(Axis::Vertical)
        .test_id_prefix("ui-gallery-resizable-demo.nested-vertical")
        .entries([
            shadcn::ResizablePanel::new([panel(cx, "Two", None)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([panel(cx, "Three", None)]).into(),
        ])
        .into_element(cx);

    let group = shadcn::ResizablePanelGroup::new(h_fractions)
        .axis(Axis::Horizontal)
        .test_id_prefix("ui-gallery-resizable-demo")
        .entries([
            shadcn::ResizablePanel::new([panel(cx, "One", Some(Px(200.0)))]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([nested_vertical]).into(),
        ])
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
    .test_id("ui-gallery-resizable-demo")
}
// endregion: example
