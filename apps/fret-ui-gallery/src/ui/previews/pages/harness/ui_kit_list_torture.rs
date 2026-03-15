use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use fret::UiCx;
use fret_ui::element::ContainerProps;

pub(in crate::ui) fn preview_ui_kit_list_torture(
    cx: &mut UiCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let selection = cx.local_model_keyed("selection", || Option::<usize>::None);

    let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());

    let header = ui::v_flex(|cx| {
            vec![
                cx.text(
                    "Goal: validate fret-ui-kit list virtualization under view-cache + shell reuse (ADR 0177).",
                ),
                cx.text("Expect: scroll boundary shifts reconcile without scroll-window dirty views."),
            ]
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2).into_element(cx);

    let len: usize = std::env::var("FRET_UI_GALLERY_UI_KIT_LIST_LEN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000)
        .clamp(16, 200_000);
    let overscan: usize = 6;

    let list = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        vec![
            fret_ui_kit::declarative::list::list_virtualized_copyable_retained_v0(
                cx,
                selection,
                fret_ui_kit::Size::Medium,
                None,
                len,
                overscan,
                &scroll_handle,
                0,
                |i| i as u64,
                Arc::new(|_models, i| Some(format!("Item {i}"))),
                |_i| None,
                |cx, i| {
                    let mut out = Vec::new();
                    let label = cx.text(format!("Item {i}"));
                    let label = if i == 0 {
                        label.attach_semantics(
                            SemanticsDecoration::default()
                                .test_id("ui-gallery-ui-kit-list-row-0-label"),
                        )
                    } else {
                        label
                    };
                    out.push(label);
                    out.push(cx.spacer(fret_ui::element::SpacerProps {
                        min: Px(0.0),
                        ..Default::default()
                    }));
                    out
                },
            ),
        ]
    });

    let list = list.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::List)
            .test_id("ui-gallery-ui-kit-list-root"),
    );

    // Bound the list height so virtualization exercises a realistic viewport window.
    let list = cx
        .container(
            ContainerProps {
                layout: decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .h_px(MetricRef::Px(Px(520.0))),
                ),
                ..Default::default()
            },
            |_cx| vec![list],
        )
        .test_id("ui-gallery-ui-kit-list-viewport");

    let root = ui::v_flex(|_cx| vec![header, list])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N3)
        .into_element(cx);

    let root = root.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-ui-kit-list-torture-root"),
    );

    let harness = DocSection::build(cx, "Harness", root)
        .description("Expect: scroll boundary shifts reconcile without scroll-window dirty views.")
        .no_shell()
        .max_w(Px(980.0));

    let page = doc_layout::render_doc_page(
        cx,
        Some("Validate fret-ui-kit list virtualization under view-cache + shell reuse (ADR 0177)."),
        vec![harness],
    );

    vec![page.into_element(cx)]
}
