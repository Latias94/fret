// region: example
use fret_core::Axis;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    fractions: Option<Model<Vec<f32>>>,
}

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
    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().h_full())
            .items_center()
            .justify_center(),
        move |cx| vec![shadcn::typography::small(cx, label)],
    );

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().p(Space::N6),
        LayoutRefinement::default().w_full().h_full(),
    );
    cx.container(props, move |_cx| [body])
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let fractions = state.fractions.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
        cx.with_state(Models::default, |st| st.fractions = Some(model.clone()));
        model
    });

    let group = shadcn::ResizablePanelGroup::new(fractions)
        .axis(Axis::Vertical)
        .test_id_prefix("ui-gallery-resizable-vertical")
        .entries([
            shadcn::ResizablePanel::new([panel(cx, "Header")]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([panel(cx, "Content")]).into(),
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
    .test_id("ui-gallery-resizable-vertical")
}
// endregion: example

