use super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_file_tree_torture(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let _ = theme;
    use std::collections::HashSet;

    let row_height = Px(26.0);
    let overscan = 12;

    let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(460.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    use fret_ui_kit::{TreeItem, TreeItemId, TreeState};

    let items = cx.local_model_keyed("items", || {
        let root_count: u64 = std::env::var("FRET_UI_GALLERY_FILE_TREE_ROOTS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(200);
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
                    leaves.push(TreeItem::new(
                        leaf_id,
                        Arc::<str>::from(format!("file_{r}_{f}_{l}.rs")),
                    ));
                }

                folders.push(
                    TreeItem::new(folder_id, Arc::<str>::from(format!("dir_{r}_{f}")))
                        .children(leaves),
                );
            }

            roots.push(
                TreeItem::new(root_id, Arc::<str>::from(format!("root_{r}"))).children(folders),
            );
        }

        roots
    });
    let state = cx.local_model_keyed("state", || {
        let root_count: u64 = std::env::var("FRET_UI_GALLERY_FILE_TREE_ROOTS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(200);
        let folders_per_root = 10u64;

        let mut expanded: HashSet<TreeItemId> = HashSet::new();
        for r in 0..root_count {
            let root_id = r;
            expanded.insert(root_id);
            for f in 0..folders_per_root {
                let folder_id = 1_000_000 + r * 100 + f;
                expanded.insert(folder_id);
            }
        }

        TreeState {
            expanded,
            selected: None,
        }
    });

    let mut props = fret_ui_kit::declarative::file_tree::FileTreeViewProps::default();
    props.layout = list_layout;
    props.row_height = row_height;
    props.overscan = overscan;
    props.debug_root_test_id = Some(Arc::<str>::from("ui-gallery-file-tree-root"));
    props.debug_row_test_id_prefix = Some(Arc::<str>::from("ui-gallery-file-tree-node"));

    vec![
        fret_ui_kit::declarative::file_tree::file_tree_view_retained_v0(
            cx,
            items,
            state,
            &scroll_handle,
            props,
        ),
    ]
}
