use std::path::PathBuf;

use fret_app::App;
use fret_core::{AppWindowId, PanelKey, RenderTargetId};
use fret_editor::{AssetGuid, ProjectEntryKind, ProjectService};

use crate::DemoDriver;

#[derive(Debug, Clone, PartialEq)]
pub enum AssetDropTarget {
    SceneViewport {
        panel: PanelKey,
        target: RenderTargetId,
        uv: (f32, f32),
    },
    Hierarchy {
        parent: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetDropRequest {
    pub window: AppWindowId,
    pub guid: AssetGuid,
    pub target: AssetDropTarget,
}

#[derive(Default)]
pub struct AssetDropService {
    queue: Vec<AssetDropRequest>,
}

impl AssetDropService {
    pub fn push(&mut self, req: AssetDropRequest) {
        self.queue.push(req);
    }

    pub fn drain(&mut self) -> Vec<AssetDropRequest> {
        std::mem::take(&mut self.queue)
    }
}

#[derive(Debug, Clone)]
pub struct CurrentSceneService {
    guid: Option<AssetGuid>,
    revision: u64,
}

impl Default for CurrentSceneService {
    fn default() -> Self {
        Self {
            guid: None,
            revision: 0,
        }
    }
}

impl CurrentSceneService {
    pub fn guid(&self) -> Option<AssetGuid> {
        self.guid
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn set_guid(&mut self, guid: Option<AssetGuid>) {
        if self.guid == guid {
            return;
        }
        self.guid = guid;
        self.revision = self.revision.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetDropDecision {
    Handled,
    Ignored,
}

pub struct AssetDropMatchCx<'a> {
    pub kind: Option<ProjectEntryKind>,
    pub extension: Option<&'a str>,
    pub target: &'a AssetDropTarget,
}

pub struct AssetDropApplyCx<'a> {
    pub app: &'a mut App,
    pub driver: &'a mut DemoDriver,
    pub window: AppWindowId,
    pub guid: AssetGuid,
    pub path: Option<PathBuf>,
    pub target: AssetDropTarget,
}

pub type AssetDropMatchFn = Box<dyn Fn(&AssetDropMatchCx<'_>) -> bool>;
pub type AssetDropApplyFn = Box<dyn Fn(&mut AssetDropApplyCx<'_>) -> AssetDropDecision>;

pub struct AssetDropRule {
    pub matches: AssetDropMatchFn,
    pub apply: AssetDropApplyFn,
}

#[derive(Default)]
pub struct AssetDropRegistry {
    rules: Vec<AssetDropRule>,
}

impl AssetDropRegistry {
    pub fn add_rule(&mut self, rule: AssetDropRule) {
        self.rules.push(rule);
    }

    pub fn handle(
        &mut self,
        driver: &mut DemoDriver,
        app: &mut App,
        req: AssetDropRequest,
    ) -> AssetDropDecision {
        let (kind, path, extension) = {
            let Some(project) = app.global::<ProjectService>() else {
                return AssetDropDecision::Ignored;
            };
            let id = project.id_for_guid(req.guid);
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

        // Prefer last-registered rules (allows "override" semantics like Godot/GPUIs plugin stacks).
        for rule in self.rules.iter().rev() {
            let matches = {
                let match_cx = AssetDropMatchCx {
                    kind,
                    extension: extension.as_deref(),
                    target: &req.target,
                };
                (rule.matches)(&match_cx)
            };
            if !matches {
                continue;
            }

            let mut apply_cx = AssetDropApplyCx {
                app,
                driver,
                window: req.window,
                guid: req.guid,
                path,
                target: req.target,
            };
            return (rule.apply)(&mut apply_cx);
        }

        AssetDropDecision::Ignored
    }
}
