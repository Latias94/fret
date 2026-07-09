use std::collections::HashSet;
use std::sync::Arc;

use fret_app::CommandId;
use fret_ui::VirtualListScrollHandle;
use fret_ui::elements::GlobalElementId;
use fret_ui_kit::{TreeItem, TreeState};
use fret_workspace::close_policy::{
    WorkspaceCloseReason, WorkspaceDirtyCloseDecision, WorkspaceDirtyClosePolicy,
    WorkspaceDirtyCloseRequest,
};
use fret_workspace::layout::WorkspaceWindowLayout;

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
    pub(crate) dirty_close_prompt_open: fret_app::Model<bool>,
    pub(crate) dirty_close_prompt: fret_app::Model<Option<WorkspaceShellDirtyClosePrompt>>,
    pub(crate) tabstrip_two_row_pinned: fret_app::Model<bool>,
    pub(crate) file_tree_items: fret_app::Model<Vec<TreeItem>>,
    pub(crate) file_tree_state: fret_app::Model<TreeState>,
    pub(crate) file_tree_scroll: VirtualListScrollHandle,
}

pub(crate) const CMD_WORKSPACE_SHELL_DEMO_SET_ACTIVE_DIRTY: &str =
    "workspace.shell_demo.set_active_dirty";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_CLEAR_ACTIVE_DIRTY: &str =
    "workspace.shell_demo.clear_active_dirty";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_SET_PANE_B_ACTIVE_DIRTY: &str =
    "workspace.shell_demo.set_pane_b_active_dirty";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_DEBUG_CLOSE_ACTIVE_PANE_A: &str =
    "workspace.shell_demo.debug_close_active_in_pane_a";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_WINDOW_CLOSE: &str = "window.close";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_CANCEL: &str =
    "workspace.shell_demo.dirty_close.cancel";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_DISCARD: &str =
    "workspace.shell_demo.dirty_close.discard";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_SAVE_AND_CLOSE: &str =
    "workspace.shell_demo.dirty_close.save_and_close";
pub(crate) const CMD_WORKSPACE_SHELL_DEMO_TOGGLE_TABSTRIP_TWO_ROW_PINNED: &str =
    "workspace.shell_demo.toggle_tabstrip_two_row_pinned";

pub(crate) const DIRTY_CLOSE_PROMPT_OVERLAY_ID: GlobalElementId =
    GlobalElementId(0x6a4e_5c1f_8f3b_1c20);

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceShellDirtyClosePrompt {
    pub(crate) pane_id: Arc<str>,
    pub(crate) command: CommandId,
    pub(crate) request: WorkspaceDirtyCloseRequest,
}

impl WorkspaceShellDirtyClosePrompt {
    pub(crate) fn window_close(request: WorkspaceDirtyCloseRequest) -> Self {
        Self {
            pane_id: Arc::from("<window>"),
            command: CommandId::new(Arc::<str>::from(CMD_WORKSPACE_SHELL_DEMO_WINDOW_CLOSE)),
            request,
        }
    }

    pub(crate) fn tab_command(
        pane_id: Arc<str>,
        command: CommandId,
        request: WorkspaceDirtyCloseRequest,
    ) -> Self {
        Self {
            pane_id,
            command,
            request,
        }
    }

    pub(crate) fn is_window_close(&self) -> bool {
        self.request.reason == WorkspaceCloseReason::CloseWindow
    }
}

pub(crate) struct WorkspaceShellDemoDirtyClosePolicy {
    pub(crate) block: bool,
}

impl WorkspaceDirtyClosePolicy for WorkspaceShellDemoDirtyClosePolicy {
    fn decide_dirty_close(
        &mut self,
        _request: &WorkspaceDirtyCloseRequest,
    ) -> WorkspaceDirtyCloseDecision {
        if self.block {
            WorkspaceDirtyCloseDecision::Block
        } else {
            WorkspaceDirtyCloseDecision::Allow
        }
    }
}
