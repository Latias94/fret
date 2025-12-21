use std::path::PathBuf;

use fret_app::App;
use fret_core::{AppWindowId, PanelKey};
use fret_editor::{AssetGuid, ProjectEntryKind, ProjectService};

use crate::DemoDriver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetOpenDecision {
    Handled,
    Ignored,
}

pub struct AssetOpenMatchCx<'a> {
    pub kind: Option<ProjectEntryKind>,
    pub extension: Option<&'a str>,
}

pub struct AssetOpenApplyCx<'a> {
    pub app: &'a mut App,
    pub driver: &'a mut DemoDriver,
    pub window: AppWindowId,
    pub guid: AssetGuid,
    pub path: Option<PathBuf>,
}

pub type AssetOpenMatchFn = Box<dyn Fn(&AssetOpenMatchCx<'_>) -> bool>;
pub type AssetOpenApplyFn = Box<dyn Fn(&mut AssetOpenApplyCx<'_>) -> AssetOpenDecision>;

pub struct AssetOpenRule {
    pub matches: AssetOpenMatchFn,
    pub apply: AssetOpenApplyFn,
}

#[derive(Default)]
pub struct AssetOpenRegistry {
    rules: Vec<AssetOpenRule>,
}

impl AssetOpenRegistry {
    pub fn add_rule(&mut self, rule: AssetOpenRule) {
        self.rules.push(rule);
    }

    pub fn handle(
        &mut self,
        driver: &mut DemoDriver,
        app: &mut App,
        window: AppWindowId,
        guid: AssetGuid,
    ) -> AssetOpenDecision {
        let (kind, path, extension) = {
            let Some(project) = app.global::<ProjectService>() else {
                return AssetOpenDecision::Ignored;
            };
            let id = project.id_for_guid(guid);
            let kind = id.and_then(|id| project.kind_for_id(id));
            let path = id
                .and_then(|id| project.path_for_id(id))
                .map(|p| p.to_path_buf());
            let extension = path
                .as_ref()
                .and_then(|p| p.extension().and_then(|s| s.to_str()))
                .map(|s| s.to_ascii_lowercase());
            (kind, path, extension)
        };

        for rule in self.rules.iter().rev() {
            let matches = {
                let cx = AssetOpenMatchCx {
                    kind,
                    extension: extension.as_deref(),
                };
                (rule.matches)(&cx)
            };
            if !matches {
                continue;
            }

            let mut cx = AssetOpenApplyCx {
                app,
                driver,
                window,
                guid,
                path,
            };
            return (rule.apply)(&mut cx);
        }

        AssetOpenDecision::Ignored
    }
}

pub fn activate_panel_tab(
    app: &mut App,
    window: AppWindowId,
    panel: PanelKey,
) -> Option<fret_core::DockOp> {
    let dock = app.global::<fret_ui::DockManager>()?;
    let root = dock.graph.window_root(window)?;

    fn find(
        graph: &fret_core::DockGraph,
        node: fret_core::DockNodeId,
        panel: &PanelKey,
    ) -> Option<(fret_core::DockNodeId, usize)> {
        match graph.node(node)? {
            fret_core::DockNode::Tabs { tabs, .. } => {
                let i = tabs.iter().position(|p| p == panel)?;
                Some((node, i))
            }
            fret_core::DockNode::Split { children, .. } => {
                children.iter().copied().find_map(|c| find(graph, c, panel))
            }
        }
    }

    let (tabs, active) = find(&dock.graph, root, &panel)?;
    Some(fret_core::DockOp::SetActiveTab { tabs, active })
}
