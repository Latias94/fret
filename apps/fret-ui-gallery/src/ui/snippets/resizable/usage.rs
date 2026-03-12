pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Axis;
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn panel<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    label: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    ui::v_flex(move |cx| vec![shadcn::raw::typography::small(label).into_element(cx)])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let fractions = cx.local_model_keyed("fractions", || vec![0.5, 0.5]);

    shadcn::ResizablePanelGroup::new(fractions)
        .axis(Axis::Horizontal)
        .test_id_prefix("ui-gallery-resizable-usage")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .h_px(Px(120.0)),
        )
        .entries([
            shadcn::ResizablePanel::new([panel(cx, "One").into_element(cx)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([panel(cx, "Two").into_element(cx)]).into(),
        ])
        .into_element(cx)
}
// endregion: example
