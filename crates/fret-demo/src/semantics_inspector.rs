use fret_core::{AppWindowId, FrameId, KeyCode, Px, SemanticsSnapshot, TextWrap};
use fret_ui_app::VirtualListRowHeight;
use fret_ui_app::{
    App, EventCx, GenericWidget, LayoutCx, PaintCx, VirtualList, VirtualListDataSource,
    VirtualListRow,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
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
    pending_requests: HashSet<AppWindowId>,
}

impl SemanticsInspectorService {
    pub fn request(&mut self, window: AppWindowId) {
        self.pending_requests.insert(window);
    }

    pub fn should_sample(&self, window: AppWindowId, _now: std::time::Instant) -> bool {
        self.pending_requests.contains(&window)
    }

    pub fn set_snapshot(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        snapshot: Arc<SemanticsSnapshot>,
        _now: std::time::Instant,
    ) {
        self.next_revision = self.next_revision.saturating_add(1);
        let revision = self.next_revision;
        self.pending_requests.remove(&window);
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
        self.pending_requests.remove(&window);
        self.per_window.remove(&window);
    }

    pub fn snapshot(&self, window: AppWindowId) -> Option<&SemanticsInspectorEntry> {
        self.per_window.get(&window)
    }
}

#[derive(Debug, Clone)]
struct SemanticsListLine {
    text: String,
    indent_x: Px,
}

#[derive(Debug, Clone)]
struct SemanticsListDataSource {
    lines: Vec<SemanticsListLine>,
}

impl SemanticsListDataSource {
    fn empty() -> Self {
        Self { lines: Vec::new() }
    }

    fn from_entry(entry: &SemanticsInspectorEntry) -> Self {
        let snap = &entry.snapshot;

        let mut depth_by_id: HashMap<fret_core::NodeId, usize> = HashMap::new();
        let mut parent_by_id: HashMap<fret_core::NodeId, Option<fret_core::NodeId>> =
            HashMap::new();
        for n in &snap.nodes {
            parent_by_id.insert(n.id, n.parent);
        }

        fn depth_for(
            id: fret_core::NodeId,
            parent_by_id: &HashMap<fret_core::NodeId, Option<fret_core::NodeId>>,
            depth_by_id: &mut HashMap<fret_core::NodeId, usize>,
        ) -> usize {
            if let Some(d) = depth_by_id.get(&id) {
                return *d;
            }
            let mut depth = 0usize;
            let mut current = id;
            let mut guard = 0usize;
            while let Some(Some(parent)) = parent_by_id.get(&current) {
                depth = depth.saturating_add(1);
                current = *parent;
                guard = guard.saturating_add(1);
                if guard > 128 {
                    break;
                }
            }
            depth_by_id.insert(id, depth);
            depth
        }

        let mut lines: Vec<SemanticsListLine> =
            Vec::with_capacity(4 + snap.roots.len() + snap.nodes.len());

        lines.push(SemanticsListLine {
            indent_x: Px(0.0),
            text: format!(
                "Semantics frame={:?} roots={} nodes={} barrier={:?} focus={:?} captured={:?}",
                entry.frame_id,
                snap.roots.len(),
                snap.nodes.len(),
                snap.barrier_root,
                snap.focus,
                snap.captured
            ),
        });
        lines.push(SemanticsListLine {
            indent_x: Px(0.0),
            text: String::new(),
        });

        lines.push(SemanticsListLine {
            indent_x: Px(0.0),
            text: "Roots".to_string(),
        });
        for r in &snap.roots {
            lines.push(SemanticsListLine {
                indent_x: Px(0.0),
                text: format!(
                    "z={} id={:?} visible={} hit_testable={} blocks_underlay_input={}",
                    r.z_index, r.root, r.visible, r.hit_testable, r.blocks_underlay_input
                ),
            });
        }

        lines.push(SemanticsListLine {
            indent_x: Px(0.0),
            text: String::new(),
        });

        lines.push(SemanticsListLine {
            indent_x: Px(0.0),
            text: "Nodes".to_string(),
        });
        for n in &snap.nodes {
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
            let flags_text = if flags.is_empty() {
                "-".to_string()
            } else {
                flags.join("|")
            };

            let depth = depth_for(n.id, &parent_by_id, &mut depth_by_id);
            let indent_x = Px(depth as f32 * 12.0);

            lines.push(SemanticsListLine {
                indent_x,
                text: format!(
                    "id={:?} role={:?} flags={} parent={:?} bounds=({:.1},{:.1})+({:.1},{:.1})",
                    n.id,
                    n.role,
                    flags_text,
                    n.parent,
                    n.bounds.origin.x.0,
                    n.bounds.origin.y.0,
                    n.bounds.size.width.0,
                    n.bounds.size.height.0
                ),
            });
        }

        Self { lines }
    }
}

impl VirtualListDataSource for SemanticsListDataSource {
    type Key = usize;

    fn len(&self) -> usize {
        self.lines.len().max(1)
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        if self.lines.is_empty() {
            return VirtualListRow::new("No semantics snapshot (press R to refresh).");
        }

        let idx = index.min(self.lines.len().saturating_sub(1));
        let line = &self.lines[idx];
        VirtualListRow {
            text: line.text.as_str().into(),
            indent_x: line.indent_x,
        }
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
            list: VirtualList::new(SemanticsListDataSource::empty())
                .with_row_height(VirtualListRowHeight::Fixed(Px(20.0)))
                .with_wrap(TextWrap::None),
            last_revision: None,
        }
    }

    fn request_snapshot(&mut self, app: &mut App, window: AppWindowId) {
        app.global_mut::<SemanticsInspectorService>()
            .map(|s| s.request(window));
    }

    fn maybe_refresh(&mut self, app: &mut App, window: AppWindowId) {
        let Some(entry) = app
            .global::<SemanticsInspectorService>()
            .and_then(|s| s.snapshot(window))
        else {
            self.request_snapshot(app, window);
            if self.last_revision.is_some() {
                self.last_revision = None;
                self.list.set_data(SemanticsListDataSource::empty());
            }
            return;
        };

        if self.last_revision == Some(entry.revision) {
            return;
        }
        self.last_revision = Some(entry.revision);
        self.list.set_data(SemanticsListDataSource::from_entry(entry));
    }
}

impl GenericWidget<App> for SemanticsPanel {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &fret_core::Event) {
        let Some(window) = cx.window else {
            return;
        };

        if let fret_core::Event::KeyDown { key, modifiers, .. } = event {
            if *key == KeyCode::KeyR
                && !modifiers.shift
                && !modifiers.ctrl
                && !modifiers.alt
                && !modifiers.meta
                && !modifiers.alt_gr
            {
                self.request_snapshot(cx.app, window);
            }
        }

        self.maybe_refresh(cx.app, window);
        self.list.event(cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> fret_core::Size {
        let Some(window) = cx.window else {
            return cx.available;
        };
        self.maybe_refresh(cx.app, window);
        self.list.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(window) = cx.window else {
            return;
        };
        self.maybe_refresh(cx.app, window);
        self.list.paint(cx);
    }
}
