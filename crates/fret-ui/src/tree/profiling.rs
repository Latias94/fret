use super::*;

const SCROLL_LAYOUT_KIND_PROFILE_MAX: usize = 12;

#[derive(Debug, Clone, Copy)]
pub(super) struct LayoutNodeProfileEntry {
    pub(super) node: NodeId,
    pub(super) pass_kind: crate::layout_pass::LayoutPassKind,
    pub(super) bounds: Rect,
    pub(super) elapsed_total: Duration,
    pub(super) elapsed_self: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LayoutNodeProfileConfig {
    top_n: usize,
    min_elapsed: Duration,
}

impl LayoutNodeProfileConfig {
    pub(super) fn from_env() -> Option<Self> {
        let cfg = crate::runtime_config::ui_runtime_config().layout_node_profile?;
        Some(Self {
            top_n: cfg.top_n,
            min_elapsed: cfg.min_elapsed,
        })
    }
}

#[derive(Debug)]
pub(super) struct LayoutNodeProfileState {
    config: LayoutNodeProfileConfig,
    pub(super) frame_id: FrameId,
    pub(super) entries: Vec<LayoutNodeProfileEntry>,
    stack: Vec<LayoutNodeProfileStackEntry>,
    pub(super) total_self_time: Duration,
    pub(super) nodes_profiled: u64,
}

impl LayoutNodeProfileState {
    pub(super) fn new(config: LayoutNodeProfileConfig, frame_id: FrameId) -> Self {
        Self {
            config,
            frame_id,
            entries: Vec::new(),
            stack: Vec::new(),
            total_self_time: Duration::default(),
            nodes_profiled: 0,
        }
    }

    pub(super) fn enter(
        &mut self,
        node: NodeId,
        pass_kind: crate::layout_pass::LayoutPassKind,
        bounds: Rect,
    ) {
        self.stack.push(LayoutNodeProfileStackEntry {
            node,
            pass_kind,
            bounds,
            started: fret_core::time::Instant::now(),
            child_time: Duration::default(),
        });
    }

    pub(super) fn exit(&mut self, node: NodeId) {
        let Some(entry) = self.stack.pop() else {
            return;
        };
        if entry.node != node {
            // Best-effort: avoid poisoning the layout pass if the stack gets out of sync.
            self.stack.clear();
            return;
        }

        let elapsed_total = entry.started.elapsed();
        let elapsed_self = elapsed_total.saturating_sub(entry.child_time);
        self.total_self_time = self.total_self_time.saturating_add(elapsed_self);
        self.nodes_profiled = self.nodes_profiled.saturating_add(1);

        if let Some(parent) = self.stack.last_mut() {
            parent.child_time = parent.child_time.saturating_add(elapsed_total);
        }

        self.record(LayoutNodeProfileEntry {
            node: entry.node,
            pass_kind: entry.pass_kind,
            bounds: entry.bounds,
            elapsed_total,
            elapsed_self,
        });
    }

    fn record(&mut self, entry: LayoutNodeProfileEntry) {
        if entry.elapsed_self < self.config.min_elapsed {
            return;
        }

        // Keep a stable, small "top N" list; N is tiny (default 16), so O(N) insertion is fine.
        let mut inserted = false;
        for i in 0..self.entries.len() {
            if entry.elapsed_self > self.entries[i].elapsed_self {
                self.entries.insert(i, entry);
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.entries.push(entry);
        }
        if self.entries.len() > self.config.top_n {
            self.entries.truncate(self.config.top_n);
        }
    }
}

#[derive(Debug)]
struct LayoutNodeProfileStackEntry {
    node: NodeId,
    pass_kind: crate::layout_pass::LayoutPassKind,
    bounds: Rect,
    started: fret_core::time::Instant,
    child_time: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MeasureNodeProfileEntry {
    pub(super) node: NodeId,
    pub(super) constraints: crate::layout_constraints::LayoutConstraints,
    pub(super) elapsed_total: Duration,
    pub(super) elapsed_self: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MeasureNodeProfileConfig {
    top_n: usize,
    min_elapsed: Duration,
}

impl MeasureNodeProfileConfig {
    pub(super) fn from_env() -> Option<Self> {
        let cfg = crate::runtime_config::ui_runtime_config().measure_node_profile?;
        Some(Self {
            top_n: cfg.top_n,
            min_elapsed: cfg.min_elapsed,
        })
    }
}

#[derive(Debug)]
pub(super) struct MeasureNodeProfileState {
    config: MeasureNodeProfileConfig,
    pub(super) frame_id: FrameId,
    pub(super) entries: Vec<MeasureNodeProfileEntry>,
    stack: Vec<MeasureNodeProfileStackEntry>,
    pub(super) total_self_time: Duration,
    pub(super) nodes_profiled: u64,
}

impl MeasureNodeProfileState {
    pub(super) fn new(config: MeasureNodeProfileConfig, frame_id: FrameId) -> Self {
        Self {
            config,
            frame_id,
            entries: Vec::new(),
            stack: Vec::new(),
            total_self_time: Duration::default(),
            nodes_profiled: 0,
        }
    }

    pub(super) fn enter(
        &mut self,
        node: NodeId,
        constraints: crate::layout_constraints::LayoutConstraints,
    ) {
        self.stack.push(MeasureNodeProfileStackEntry {
            node,
            constraints,
            started: fret_core::time::Instant::now(),
            child_time: Duration::default(),
        });
    }

    pub(super) fn exit(&mut self, node: NodeId) {
        let Some(entry) = self.stack.pop() else {
            return;
        };
        if entry.node != node {
            self.stack.clear();
            return;
        }

        let elapsed_total = entry.started.elapsed();
        let elapsed_self = elapsed_total.saturating_sub(entry.child_time);
        self.total_self_time = self.total_self_time.saturating_add(elapsed_self);
        self.nodes_profiled = self.nodes_profiled.saturating_add(1);

        if let Some(parent) = self.stack.last_mut() {
            parent.child_time = parent.child_time.saturating_add(elapsed_total);
        }

        self.record(MeasureNodeProfileEntry {
            node: entry.node,
            constraints: entry.constraints,
            elapsed_total,
            elapsed_self,
        });
    }

    fn record(&mut self, entry: MeasureNodeProfileEntry) {
        if entry.elapsed_total < self.config.min_elapsed {
            return;
        }

        let mut inserted = false;
        for i in 0..self.entries.len() {
            if entry.elapsed_total > self.entries[i].elapsed_total {
                self.entries.insert(i, entry);
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.entries.push(entry);
        }
        if self.entries.len() > self.config.top_n {
            self.entries.truncate(self.config.top_n);
        }
    }
}

#[derive(Debug)]
struct MeasureNodeProfileStackEntry {
    node: NodeId,
    constraints: crate::layout_constraints::LayoutConstraints,
    started: fret_core::time::Instant,
    child_time: Duration,
}

#[derive(Debug, Default)]
pub(super) struct ScrollLayoutKindProfileScope {
    entries: Vec<ScrollLayoutKindProfileEntry>,
}

impl ScrollLayoutKindProfileScope {
    pub(super) fn record(
        &mut self,
        kind: &'static str,
        elapsed_self: Duration,
        elapsed_total: Duration,
    ) {
        let self_us = elapsed_self.as_micros().min(u64::MAX as u128) as u64;
        let total_us = elapsed_total.as_micros().min(u64::MAX as u128) as u64;
        if self_us == 0 && total_us == 0 {
            return;
        }
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.kind == kind) {
            entry.nodes = entry.nodes.saturating_add(1);
            entry.self_us = entry.self_us.saturating_add(self_us);
            entry.total_us = entry.total_us.saturating_add(total_us);
            entry.max_self_us = entry.max_self_us.max(self_us);
            entry.max_total_us = entry.max_total_us.max(total_us);
        } else {
            self.entries.push(ScrollLayoutKindProfileEntry {
                kind,
                nodes: 1,
                self_us,
                total_us,
                max_self_us: self_us,
                max_total_us: total_us,
            });
        }
    }

    pub(super) fn into_debug_profiles(
        mut self,
    ) -> Vec<crate::tree::UiDebugScrollLayoutKindProfile> {
        self.entries
            .sort_by(|a, b| b.self_us.cmp(&a.self_us).then_with(|| a.kind.cmp(b.kind)));
        self.entries.truncate(SCROLL_LAYOUT_KIND_PROFILE_MAX);
        self.entries
            .into_iter()
            .map(|entry| crate::tree::UiDebugScrollLayoutKindProfile {
                kind: entry.kind,
                nodes: entry.nodes,
                self_us: entry.self_us,
                total_us: entry.total_us,
                max_self_us: entry.max_self_us,
                max_total_us: entry.max_total_us,
            })
            .collect()
    }

    pub(super) fn absorb_debug_profiles(
        &mut self,
        profiles: &[crate::tree::UiDebugScrollLayoutKindProfile],
    ) {
        for profile in profiles {
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|entry| entry.kind == profile.kind)
            {
                entry.nodes = entry.nodes.saturating_add(profile.nodes);
                entry.self_us = entry.self_us.saturating_add(profile.self_us);
                entry.total_us = entry.total_us.saturating_add(profile.total_us);
                entry.max_self_us = entry.max_self_us.max(profile.max_self_us);
                entry.max_total_us = entry.max_total_us.max(profile.max_total_us);
            } else {
                self.entries.push(ScrollLayoutKindProfileEntry {
                    kind: profile.kind,
                    nodes: profile.nodes,
                    self_us: profile.self_us,
                    total_us: profile.total_us,
                    max_self_us: profile.max_self_us,
                    max_total_us: profile.max_total_us,
                });
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ScrollLayoutKindProfileEntry {
    kind: &'static str,
    nodes: u32,
    self_us: u64,
    total_us: u64,
    max_self_us: u64,
    max_total_us: u64,
}
