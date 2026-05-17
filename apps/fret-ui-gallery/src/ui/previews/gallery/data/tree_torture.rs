use super::super::super::super::*;
use fret::{AppComponentCx, app::AppRenderActionsExt};

const DYNAMIC_DISABLED_TREE_ROW_ID: u64 = 2_000_000;

fn set_tree_item_disabled(
    items: &mut [fret_ui_kit::TreeItem],
    target_id: u64,
    disabled: bool,
) -> bool {
    for item in items {
        if item.id == target_id {
            item.disabled = disabled;
            return true;
        }
        if set_tree_item_disabled(&mut item.children, target_id, disabled) {
            return true;
        }
    }
    false
}

fn tree_item_disabled(items: &[fret_ui_kit::TreeItem], target_id: u64) -> Option<bool> {
    for item in items {
        if item.id == target_id {
            return Some(item.disabled);
        }
        if let Some(disabled) = tree_item_disabled(&item.children, target_id) {
            return Some(disabled);
        }
    }
    None
}

pub(in crate::ui) fn preview_tree_torture(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::collections::HashSet;

    use fret_ui_kit::TreeItem;
    use fret_ui_kit::TreeState;

    let variable_height = std::env::var_os("FRET_UI_GALLERY_TREE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();

    let items = cx.local_model_keyed("items", || {
        let root_count = 200u64;
        let folders_per_root = 10u64;
        let leaves_per_folder = 25u64;

        let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

        for r in 0..root_count {
            let root_id = r;

            let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
            for f in 0..folders_per_root {
                let folder_id = 1_000_000 + r * 100 + f;

                let mut leaves: Vec<TreeItem> = Vec::with_capacity(leaves_per_folder as usize);
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

                folders.push(TreeItem::new(folder_id, format!("Folder {r}/{f}")).children(leaves));
            }

            roots.push(TreeItem::new(root_id, format!("Root {r}")).children(folders));
        }

        roots
    });
    let target_disabled = cx
        .app
        .models()
        .read(&items, |items| {
            tree_item_disabled(items, DYNAMIC_DISABLED_TREE_ROW_ID)
        })
        .ok()
        .flatten()
        .unwrap_or(false);
    let state = cx.local_model_keyed("state", || {
        let root_count = 200u64;
        let folders_per_root = 10u64;
        let mut expanded: HashSet<u64> = HashSet::new();
        for r in 0..root_count {
            let root_id = r;
            expanded.insert(root_id);
            for f in 0..folders_per_root {
                let folder_id = 1_000_000 + r * 100 + f;
                expanded.insert(folder_id);
            }
        }
        TreeState {
            selected: None,
            expanded,
        }
    });

    let header = ui::v_flex(|cx| {
            vec![
                doc_layout::paragraph_text(cx, "Goal: baseline perf harness for a virtualized tree (expand/collapse + selection + scroll)."),
                doc_layout::paragraph_text(cx, "Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
            ]
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2).into_element(cx);

    let controls = {
        let items_for_toggle = items.clone();
        let next_disabled = !target_disabled;
        let label = if target_disabled {
            "Enable dynamic target"
        } else {
            "Disable dynamic target"
        };
        let status = Arc::<str>::from(format!(
            "Dynamic target {DYNAMIC_DISABLED_TREE_ROW_ID}: disabled={target_disabled}"
        ));

        ui::h_flex(move |cx| {
            vec![
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(cx.actions().listen({
                        let items_for_toggle = items_for_toggle.clone();
                        move |host, _action_cx| {
                            let _ = host.models_mut().update(&items_for_toggle, |items| {
                                set_tree_item_disabled(
                                    items,
                                    DYNAMIC_DISABLED_TREE_ROW_ID,
                                    next_disabled,
                                );
                            });
                        }
                    }))
                    .test_id("ui-gallery-tree-toggle-target-disabled")
                    .into_element(cx),
                doc_layout::control_readout_text(cx, status.clone())
                    .test_id("ui-gallery-tree-target-disabled-status"),
            ]
        })
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N2)
        .items_center()
        .into_element(cx)
    };

    let tree = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        |cx| {
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
                fret_ui_kit::declarative::tree::tree_view(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                )
            };

            vec![
                tree.attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-tree-torture-root"),
                ),
            ]
        },
    );

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![
        header,
        controls,
        cx.container(container_props, |_cx| vec![tree]),
    ]
}
