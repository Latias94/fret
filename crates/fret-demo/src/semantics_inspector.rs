use fret_core::{AppWindowId, FrameId, SemanticsSnapshot};
use fret_ui_app::{
    App, EventCx, GenericWidget, LayoutCx, PaintCx, VirtualList, VirtualListDataSource,
    VirtualListRow,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct SemanticsInspectorEntry {
    pub revision: u64,
    pub frame_id: FrameId,
    pub snapshot: Arc<SemanticsSnapshot>,
}

#[derive(Default)]
pub struct SemanticsInspectorService {
    next_revision: u64,
    per_window: HashMap<AppWindowId, SemanticsInspectorEntry>,
    last_update_at: HashMap<AppWindowId, Instant>,
}

impl SemanticsInspectorService {
    pub fn should_sample(&self, window: AppWindowId, now: Instant) -> bool {
        const MIN_INTERVAL: Duration = Duration::from_millis(250);
        self.last_update_at
            .get(&window)
            .is_none_or(|last| now.duration_since(*last) >= MIN_INTERVAL)
    }

    pub fn set_snapshot(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        snapshot: Arc<SemanticsSnapshot>,
        now: Instant,
    ) {
        self.next_revision = self.next_revision.saturating_add(1);
        let revision = self.next_revision;
        self.last_update_at.insert(window, now);
        self.per_window.insert(
            window,
            SemanticsInspectorEntry {
                revision,
                frame_id,
                snapshot,
            },
        );
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.per_window.remove(&window);
        self.last_update_at.remove(&window);
    }

    pub fn snapshot(&self, window: AppWindowId) -> Option<&SemanticsInspectorEntry> {
        self.per_window.get(&window)
    }
}

#[derive(Debug, Clone)]
struct SemanticsListDataSource {
    frame_id: Option<FrameId>,
    snapshot: Option<Arc<SemanticsSnapshot>>,
}

impl SemanticsListDataSource {
    fn empty() -> Self {
        Self {
            frame_id: None,
            snapshot: None,
        }
    }

    fn from_entry(entry: &SemanticsInspectorEntry) -> Self {
        Self {
            frame_id: Some(entry.frame_id),
            snapshot: Some(entry.snapshot.clone()),
        }
    }
}

impl VirtualListDataSource for SemanticsListDataSource {
    type Key = usize;

    fn len(&self) -> usize {
        let Some(snap) = self.snapshot.as_ref() else {
            return 1;
        };
        // header + blank + roots + blank + nodes
        1 + 1 + snap.roots.len() + 1 + snap.nodes.len()
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        let Some(snap) = self.snapshot.as_ref() else {
            return VirtualListRow::new(
                "No semantics snapshot (open the Semantics panel in a window).",
            );
        };

        if index == 0 {
            return VirtualListRow::new(format!(
                "Semantics — frame={:?}  roots={}  nodes={}  barrier={:?}  focus={:?}  captured={:?}",
                self.frame_id.unwrap_or_default(),
                snap.roots.len(),
                snap.nodes.len(),
                snap.barrier_root,
                snap.focus,
                snap.captured
            ));
        }

        if index == 1 {
            return VirtualListRow::new("");
        }

        let roots_start = 2;
        let roots_end = roots_start + snap.roots.len();
        if index >= roots_start && index < roots_end {
            let r = &snap.roots[index - roots_start];
            return VirtualListRow::new(format!(
                "root z={} id={:?} visible={} hit_testable={} blocks_underlay_input={}",
                r.z_index, r.root, r.visible, r.hit_testable, r.blocks_underlay_input
            ));
        }

        let nodes_start = roots_end + 1;
        if index == nodes_start - 1 {
            return VirtualListRow::new("");
        }

        let node_index = index.saturating_sub(nodes_start);
        if node_index >= snap.nodes.len() {
            return VirtualListRow::new("");
        }
        let n = &snap.nodes[node_index];

        let mut flags: Vec<&'static str> = Vec::new();
        if n.flags.focused {
            flags.push("focused");
        }
        if n.flags.captured {
            flags.push("captured");
        }
        if n.flags.disabled {
            flags.push("disabled");
        }
        if n.flags.selected {
            flags.push("selected");
        }
        if n.flags.expanded {
            flags.push("expanded");
        }
        let flags = if flags.is_empty() {
            "-".to_string()
        } else {
            flags.join("|")
        };

        VirtualListRow::new(format!(
            "{:?} parent={:?} role={:?} flags={} bounds=({}, {}) + ({}, {})",
            n.id,
            n.parent,
            n.role,
            flags,
            n.bounds.origin.x.0,
            n.bounds.origin.y.0,
            n.bounds.size.width.0,
            n.bounds.size.height.0
        ))
    }

    fn index_of_key(&self, key: Self::Key) -> Option<usize> {
        (key < self.len()).then_some(key)
    }
}

pub struct SemanticsPanel {
    list: VirtualList<SemanticsListDataSource>,
    last_revision: Option<u64>,
}

impl SemanticsPanel {
    pub fn new() -> Self {
        Self {
            list: VirtualList::new(SemanticsListDataSource::empty()),
            last_revision: None,
        }
    }

    fn maybe_refresh(&mut self, app: &App, window: AppWindowId) -> bool {
        let Some(entry) = app
            .global::<SemanticsInspectorService>()
            .and_then(|s| s.snapshot(window))
        else {
            let changed = self.last_revision.is_some();
            self.last_revision = None;
            if changed {
                self.list.set_data(SemanticsListDataSource::empty());
            }
            return changed;
        };

        if self.last_revision == Some(entry.revision) {
            return false;
        }
        self.last_revision = Some(entry.revision);
        self.list
            .set_data(SemanticsListDataSource::from_entry(entry));
        true
    }
}

impl GenericWidget<App> for SemanticsPanel {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &fret_core::Event) {
        let Some(window) = cx.window else {
            return;
        };
        let _ = self.maybe_refresh(cx.app, window);
        self.list.event(cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> fret_core::Size {
        let Some(window) = cx.window else {
            return cx.available;
        };
        let _ = self.maybe_refresh(cx.app, window);
        self.list.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(window) = cx.window else {
            return;
        };
        let _ = self.maybe_refresh(cx.app, window);
        self.list.paint(cx);
    }
}
