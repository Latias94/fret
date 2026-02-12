use super::super::super::super::*;

pub(in crate::ui) fn preview_tree_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::collections::HashSet;

    use fret_ui_kit::TreeItem;
    use fret_ui_kit::TreeState;

    let variable_height = std::env::var_os("FRET_UI_GALLERY_TREE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();

    #[derive(Default)]
    struct TreeTortureModels {
        items: Option<Model<Vec<TreeItem>>>,
        state: Option<Model<TreeState>>,
    }

    let (items, state) = cx.with_state(TreeTortureModels::default, |st| {
        (st.items.clone(), st.state.clone())
    });
    let (items, state) = match (items, state) {
        (Some(items), Some(state)) => (items, state),
        _ => {
            let (items_value, state_value) = {
                let root_count = 200u64;
                let folders_per_root = 10u64;
                let leaves_per_folder = 25u64;

                let mut expanded: HashSet<u64> = HashSet::new();
                let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

                for r in 0..root_count {
                    let root_id = r;
                    expanded.insert(root_id);

                    let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
                    for f in 0..folders_per_root {
                        let folder_id = 1_000_000 + r * 100 + f;
                        expanded.insert(folder_id);

                        let mut leaves: Vec<TreeItem> =
                            Vec::with_capacity(leaves_per_folder as usize);
                        for l in 0..leaves_per_folder {
                            let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                            let label = if variable_height && leaf_id % 15 == 0 {
                                format!(
                                    "Leaf {r}/{f}/{l} (id={leaf_id})\nDetails: id={} seed={}",
                                    leaf_id,
                                    leaf_id.wrapping_mul(2654435761)
                                )
                            } else {
                                format!("Leaf {r}/{f}/{l} (id={leaf_id})")
                            };
                            leaves.push(TreeItem::new(leaf_id, label).disabled(leaf_id % 97 == 0));
                        }

                        folders.push(
                            TreeItem::new(folder_id, format!("Folder {r}/{f}")).children(leaves),
                        );
                    }

                    roots.push(TreeItem::new(root_id, format!("Root {r}")).children(folders));
                }

                (
                    roots,
                    TreeState {
                        selected: None,
                        expanded,
                    },
                )
            };

            let items = cx.app.models_mut().insert(items_value);
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(TreeTortureModels::default, |st| {
                st.items = Some(items.clone());
                st.state = Some(state.clone());
            });
            (items, state)
        }
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized tree (expand/collapse + selection + scroll)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
            ]
        },
    );

    let tree = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let retained = std::env::var_os("FRET_UI_GALLERY_TREE_RETAINED")
            .filter(|v| !v.is_empty())
            .is_some();

        let tree = if retained {
            if variable_height {
                fret_ui_kit::declarative::tree::tree_view_retained_with_measure_mode(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                    fret_ui::element::VirtualListMeasureMode::Measured,
                    Some(Arc::<str>::from("ui-gallery-tree-row")),
                )
            } else {
                fret_ui_kit::declarative::tree::tree_view_retained(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                    Some(Arc::<str>::from("ui-gallery-tree-row")),
                )
            }
        } else {
            fret_ui_kit::declarative::tree::tree_view(cx, items, state, fret_ui_kit::Size::Medium)
        };

        vec![
            tree.attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-tree-torture-root"),
            ),
        ]
    });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![tree])]
}
