use super::super::super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(in crate::ui) fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct ResizableModels {
        vertical_fractions: Option<Model<Vec<f32>>>,
        handle_fractions: Option<Model<Vec<f32>>>,
        rtl_h_fractions: Option<Model<Vec<f32>>>,
        rtl_v_fractions: Option<Model<Vec<f32>>>,
    }

    let box_group =
        |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default().border_1().rounded(Radius::Lg),
                    layout,
                ),
                move |_cx| [body],
            )
        };

    let panel = |cx: &mut ElementContext<'_, App>, label: &'static str, height: Option<Px>| {
        let layout = match height {
            Some(h) => LayoutRefinement::default().w_full().h_px(h),
            None => LayoutRefinement::default().w_full().h_full(),
        };

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .items_center()
                .justify_center(),
            move |cx| vec![shadcn::typography::small(cx, label)],
        );

        cx.container(
            decl_style::container_props(theme, ChromeRefinement::default().p(Space::N6), layout),
            move |_cx| [body],
        )
    };

    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(448.0));

    let state = cx.with_state(ResizableModels::default, |st| st.clone());
    let vertical_fractions = match state.vertical_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.vertical_fractions = Some(model.clone())
            });
            model
        }
    };
    let handle_fractions = match state.handle_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.handle_fractions = Some(model.clone())
            });
            model
        }
    };
    let rtl_h_fractions = match state.rtl_h_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.5, 0.5]);
            cx.with_state(ResizableModels::default, |st| {
                st.rtl_h_fractions = Some(model.clone())
            });
            model
        }
    };
    let rtl_v_fractions = match state.rtl_v_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.rtl_v_fractions = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
            .axis(fret_core::Axis::Vertical)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Two", None)]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([panel(cx, "Three", None)]).into(),
            ])
            .into_element(cx);

        let group = shadcn::ResizablePanelGroup::new(h_fractions)
            .axis(fret_core::Axis::Horizontal)
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
    };

    let handle = {
        let group = shadcn::ResizablePanelGroup::new(handle_fractions)
            .axis(fret_core::Axis::Horizontal)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Sidebar", None)]).into(),
                shadcn::ResizableHandle::new().with_handle(true).into(),
                shadcn::ResizablePanel::new([panel(cx, "Content", None)]).into(),
            ])
            .into_element(cx);

        box_group(
            cx,
            max_w_md
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(200.0))),
            group,
        )
        .test_id("ui-gallery-resizable-handle")
    };

    let vertical = {
        let group = shadcn::ResizablePanelGroup::new(vertical_fractions)
            .axis(fret_core::Axis::Vertical)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Header", None)]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([panel(cx, "Content", None)]).into(),
            ])
            .into_element(cx);

        box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(200.0))),
            group,
        )
        .test_id("ui-gallery-resizable-vertical")
    };

    let rtl = {
        let group = doc_layout::rtl(cx, |cx| {
            let nested_vertical = shadcn::ResizablePanelGroup::new(rtl_v_fractions.clone())
                .axis(fret_core::Axis::Vertical)
                .entries([
                    shadcn::ResizablePanel::new([panel(cx, "اثنان", None)]).into(),
                    shadcn::ResizableHandle::new().with_handle(true).into(),
                    shadcn::ResizablePanel::new([panel(cx, "ثلاثة", None)]).into(),
                ])
                .into_element(cx);

            shadcn::ResizablePanelGroup::new(rtl_h_fractions.clone())
                .axis(fret_core::Axis::Horizontal)
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
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows `resizable-demo.tsx` (new-york-v4): nested panels, with-handle, and vertical orientation.",
            "Resizable groups expose their own semantics; keep an eye on focus order and hit-testing near handles.",
            "API reference: `ecosystem/fret-ui-shadcn/src/resizable.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Drag the handles to resize panels."),
        vec![
            DocSection::new("Demo", demo)
                .description("Nested vertical panels inside a horizontal group.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::ResizablePanelGroup::new(h_fractions)
    .axis(Axis::Horizontal)
    .entries([
        ResizablePanel::new([panel_one]).into(),
        ResizableHandle::new().into(),
        ResizablePanel::new([nested_vertical]).into(),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Handle", handle)
                .description("A handle with a visual grabber (`withHandle`).")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::ResizableHandle::new().with_handle(true)"#,
                ),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::ResizablePanelGroup::new(fractions)
    .axis(Axis::Vertical)"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Direction provider coverage for hit-testing and handle affordances.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::ResizablePanelGroup::new(fractions).into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes).description("Parity notes and references."),
        ],
    );

    vec![body.test_id("ui-gallery-resizable")]
}
