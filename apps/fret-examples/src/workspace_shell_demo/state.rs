use std::collections::HashSet;
use std::sync::Arc;

use fret_ui::VirtualListScrollHandle;
use fret_ui_kit::{TreeItem, TreeState};
use fret_workspace::{WorkspaceWorkbench, layout::WorkspaceWindowLayout};

pub(crate) mod act {
    fret::actions!([
        SetActiveDirty = "workspace.shell_demo.set_active_dirty",
        ClearActiveDirty = "workspace.shell_demo.clear_active_dirty",
        SetPaneBActiveDirty = "workspace.shell_demo.set_pane_b_active_dirty",
        DebugCloseActivePaneA = "workspace.shell_demo.debug_close_active_in_pane_a",
        CloseWindow = "window.close",
        ToggleTabstripTwoRowPinned = "workspace.shell_demo.toggle_tabstrip_two_row_pinned",
    ]);
}

pub(crate) fn build_file_tree_items() -> (Vec<TreeItem>, TreeState) {
    let root_count = 80u64;
    let folders_per_root = 6u64;
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

            let mut leaves: Vec<TreeItem> = Vec::with_capacity(leaves_per_folder as usize);
            for l in 0..leaves_per_folder {
                let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                let label: Arc<str> = Arc::from(format!("leaf_{r}_{f}_{l}"));
                leaves.push(TreeItem::new(leaf_id, label).disabled(leaf_id % 97 == 0));
            }

            folders.push(
                TreeItem::new(folder_id, Arc::<str>::from(format!("dir_{r}_{f}"))).children(leaves),
            );
        }

        roots.push(TreeItem::new(root_id, Arc::<str>::from(format!("root_{r}"))).children(folders));
    }

    (
        roots,
        TreeState {
            selected: None,
            expanded,
        },
    )
}

pub struct WorkspaceShellWindowState {
    pub(crate) view_cache_shell: bool,
    pub(crate) window_layout: fret_app::Model<WorkspaceWindowLayout>,
    pub(crate) workbench: WorkspaceWorkbench,
    pub(crate) tabstrip_two_row_pinned: fret_app::Model<bool>,
    pub(crate) file_tree_items: fret_app::Model<Vec<TreeItem>>,
    pub(crate) file_tree_state: fret_app::Model<TreeState>,
    pub(crate) file_tree_scroll: VirtualListScrollHandle,
    pub(crate) frame_stage_frame: Option<u64>,
    pub(crate) frame_stages: Vec<fret::app::UiAppFrameStage>,
    pub(crate) completed_frame_stages: Vec<fret::app::UiAppFrameStage>,
}

impl WorkspaceShellWindowState {
    pub(crate) fn completed_frame_stage_label(&self) -> Arc<str> {
        if self.completed_frame_stages.is_empty() {
            return Arc::from("Frame pipeline pending");
        }

        Arc::from(
            self.completed_frame_stages
                .iter()
                .map(|stage| format!("{stage:?}"))
                .collect::<Vec<_>>()
                .join(" > "),
        )
    }
}

impl fret::app::UiAppFrameStageSink for WorkspaceShellWindowState {
    fn record_frame_stage(&mut self, observation: fret::app::UiAppFrameObservation) {
        let frame = observation.frame_id.0;
        if self.frame_stage_frame != Some(frame) {
            self.frame_stage_frame = Some(frame);
            self.frame_stages.clear();
        }
        self.frame_stages.push(observation.stage);
        if observation.stage == fret::app::UiAppFrameStage::End {
            self.completed_frame_stages = std::mem::take(&mut self.frame_stages);
        }
    }
}
