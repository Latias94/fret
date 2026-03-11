pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Axis;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    h_fractions: Option<Model<Vec<f32>>>,
    v_fractions: Option<Model<Vec<f32>>>,
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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let state = cx.with_state(Models::default, |st| st.clone());
    let h_fractions = state.h_fractions.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(vec![0.5, 0.5]);
        cx.with_state(Models::default, |st| st.h_fractions = Some(model.clone()));
        model
    });
    let v_fractions = state.v_fractions.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
        cx.with_state(Models::default, |st| st.v_fractions = Some(model.clone()));
        model
    });

    let group = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions.clone())
            .axis(Axis::Vertical)
            .test_id_prefix("ui-gallery-resizable-rtl.nested-vertical")
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "اثنان", None)]).into(),
                shadcn::ResizableHandle::new().with_handle(true).into(),
                shadcn::ResizablePanel::new([panel(cx, "ثلاثة", None)]).into(),
            ])
            .into_element(cx);

        shadcn::ResizablePanelGroup::new(h_fractions.clone())
            .axis(Axis::Horizontal)
            .test_id_prefix("ui-gallery-resizable-rtl")
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "واحد", Some(Px(200.0)))]).into(),
                shadcn::ResizableHandle::new().with_handle(true).into(),
                shadcn::ResizablePanel::new([nested_vertical]).into(),
            ])
            .into_element(cx)
    });

    box_group(
        cx,
        max_w_sm
            .clone()
            .merge(LayoutRefinement::default().h_px(Px(320.0))),
        group,
    )
    .test_id("ui-gallery-resizable-rtl")
}
// endregion: example
