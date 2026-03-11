pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Axis;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    fractions: Option<Model<Vec<f32>>>,
}

fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    ui::v_flex(move |cx| vec![shadcn::raw::typography::small(cx, label)])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let fractions = state.fractions.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(vec![0.5, 0.5]);
        cx.with_state(Models::default, |st| st.fractions = Some(model.clone()));
        model
    });

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
            shadcn::ResizablePanel::new([panel(cx, "One")]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([panel(cx, "Two")]).into(),
        ])
        .into_element(cx)
}
// endregion: example
