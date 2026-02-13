use super::super::super::super::*;

#[derive(Default)]
struct UiKitListTortureModels {
    selection: Option<Model<Option<usize>>>,
}

pub(in crate::ui) fn preview_ui_kit_list_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let selection = cx.with_state(UiKitListTortureModels::default, |st| st.selection.clone());
    let selection = match selection {
        Some(selection) => selection,
        None => {
            let selection = cx.app.models_mut().insert(Option::<usize>::None);
            cx.with_state(UiKitListTortureModels::default, |st| {
                st.selection = Some(selection.clone());
            });
            selection
        }
    };

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text(
                    "Goal: validate fret-ui-kit list virtualization under view-cache + shell reuse (ADR 0177).",
                ),
                cx.text("Expect: scroll boundary shifts reconcile without scroll-window dirty views."),
            ]
        },
    );

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

    let root = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |_cx| vec![header, list],
    );

    let root = root.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-ui-kit-list-torture-root"),
    );

    vec![root]
}
