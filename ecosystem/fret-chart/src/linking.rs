use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use delinea::engine::model::ChartModel;
use delinea::engine::window::DataWindow;
use delinea::ids::{AxisId, DatasetId, FieldId};
use delinea::spec::AxisKind;
use delinea::{ChartSpec, LinkEvent};
use fret_runtime::Model;
use fret_ui::UiHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkAxisKey {
    pub kind: AxisKind,
    pub dataset: DatasetId,
    pub field: FieldId,
}

impl Ord for LinkAxisKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let rank = |kind: AxisKind| match kind {
            AxisKind::X => 0u8,
            AxisKind::Y => 1u8,
        };

        (rank(self.kind), self.dataset, self.field).cmp(&(
            rank(other.kind),
            other.dataset,
            other.field,
        ))
    }
}

impl PartialOrd for LinkAxisKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisPointerLinkAnchor {
    pub axis: LinkAxisKey,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushSelectionLink2D {
    pub x_axis: LinkAxisKey,
    pub y_axis: LinkAxisKey,
    pub x: DataWindow,
    pub y: DataWindow,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ChartLinkPolicy {
    pub brush: bool,
    pub axis_pointer: bool,
    pub domain_windows: bool,
}

#[derive(Debug, Clone)]
pub struct ChartLinkRouter {
    axis_to_key: BTreeMap<AxisId, LinkAxisKey>,
    key_to_axis: BTreeMap<LinkAxisKey, AxisId>,
}

impl ChartLinkRouter {
    pub fn with_explicit_axis_map(mut self, map: BTreeMap<AxisId, LinkAxisKey>) -> Self {
        self.apply_explicit_axis_map(map);
        self
    }

    fn apply_explicit_axis_map(&mut self, map: BTreeMap<AxisId, LinkAxisKey>) {
        let mut explicit_key_to_axis: BTreeMap<LinkAxisKey, Option<AxisId>> = BTreeMap::new();

        for (axis, key) in map {
            self.axis_to_key.insert(axis, key);

            match explicit_key_to_axis.get(&key).copied().flatten() {
                None => {
                    explicit_key_to_axis.insert(key, Some(axis));
                }
                Some(existing) if existing == axis => {}
                Some(_) => {
                    explicit_key_to_axis.insert(key, None);
                }
            }
        }

        for (key, axis) in explicit_key_to_axis {
            match axis {
                Some(axis) => {
                    self.key_to_axis.insert(key, axis);
                }
                None => {
                    self.key_to_axis.remove(&key);
                }
            }
        }
    }

    pub fn from_spec(spec: &ChartSpec) -> Self {
        let mut axis_kind_by_id: BTreeMap<AxisId, AxisKind> = BTreeMap::new();
        for axis in &spec.axes {
            axis_kind_by_id.insert(axis.id, axis.kind);
        }

        let mut axis_to_pairs: BTreeMap<AxisId, BTreeSet<(DatasetId, FieldId)>> = BTreeMap::new();
        for s in &spec.series {
            axis_to_pairs
                .entry(s.x_axis)
                .or_default()
                .insert((s.dataset, s.encode.x));
            axis_to_pairs
                .entry(s.y_axis)
                .or_default()
                .insert((s.dataset, s.encode.y));
        }

        let mut axis_to_key: BTreeMap<AxisId, LinkAxisKey> = BTreeMap::new();
        let mut key_to_axis: BTreeMap<LinkAxisKey, AxisId> = BTreeMap::new();
        for (axis, pairs) in axis_to_pairs {
            let Some(kind) = axis_kind_by_id.get(&axis).copied() else {
                continue;
            };
            if pairs.len() != 1 {
                continue;
            }
            let (dataset, field) = pairs.into_iter().next().unwrap();
            let key = LinkAxisKey {
                kind,
                dataset,
                field,
            };
            axis_to_key.insert(axis, key);

            match key_to_axis.get(&key).copied() {
                None => {
                    key_to_axis.insert(key, axis);
                }
                Some(_) => {
                    // Not unique: drop the reverse mapping to avoid ambiguity.
                    key_to_axis.remove(&key);
                }
            }
        }

        Self {
            axis_to_key,
            key_to_axis,
        }
    }

    pub fn from_model(model: &ChartModel) -> Self {
        let mut axis_to_pairs: BTreeMap<AxisId, BTreeSet<(DatasetId, FieldId)>> = BTreeMap::new();
        for s in model.series.values() {
            axis_to_pairs
                .entry(s.x_axis)
                .or_default()
                .insert((s.dataset, s.encode.x));
            axis_to_pairs
                .entry(s.y_axis)
                .or_default()
                .insert((s.dataset, s.encode.y));
        }

        let mut axis_to_key: BTreeMap<AxisId, LinkAxisKey> = BTreeMap::new();
        let mut key_to_axis: BTreeMap<LinkAxisKey, AxisId> = BTreeMap::new();
        for (axis, pairs) in axis_to_pairs {
            let Some(kind) = model.axes.get(&axis).map(|a| a.kind) else {
                continue;
            };
            if pairs.len() != 1 {
                continue;
            }
            let (dataset, field) = pairs.into_iter().next().unwrap();
            let key = LinkAxisKey {
                kind,
                dataset,
                field,
            };
            axis_to_key.insert(axis, key);

            match key_to_axis.get(&key).copied() {
                None => {
                    key_to_axis.insert(key, axis);
                }
                Some(_) => {
                    key_to_axis.remove(&key);
                }
            }
        }

        Self {
            axis_to_key,
            key_to_axis,
        }
    }

    pub fn axis_key(&self, axis: AxisId) -> Option<LinkAxisKey> {
        self.axis_to_key.get(&axis).copied()
    }

    pub fn axis_for_key(&self, key: LinkAxisKey) -> Option<AxisId> {
        self.key_to_axis.get(&key).copied()
    }
}

#[derive(Debug, Clone)]
pub struct LinkedChartMember {
    pub router: ChartLinkRouter,
    pub output: Model<crate::retained::ChartCanvasOutput>,
}

#[derive(Debug, Clone, Copy)]
struct LinkedChartMemberMemory {
    last_output_revision: u64,
    ignore_next_output_revision: bool,
}

#[derive(Debug)]
pub struct LinkedChartGroup {
    policy: ChartLinkPolicy,
    members: Vec<LinkedChartMember>,
    memory: Vec<LinkedChartMemberMemory>,
    brush: Model<Option<BrushSelectionLink2D>>,
    axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
    domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
}

impl LinkedChartGroup {
    pub fn new(
        policy: ChartLinkPolicy,
        brush: Model<Option<BrushSelectionLink2D>>,
        axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
        domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
    ) -> Self {
        Self {
            policy,
            members: Vec::new(),
            memory: Vec::new(),
            brush,
            axis_pointer,
            domain_windows,
        }
    }

    pub fn push(&mut self, member: LinkedChartMember) -> &mut Self {
        self.members.push(member);
        self.memory.push(LinkedChartMemberMemory {
            last_output_revision: 0,
            ignore_next_output_revision: false,
        });
        self
    }

    pub fn tick<H: UiHost>(&mut self, app: &mut H) -> bool {
        if self.members.len() <= 1 {
            return false;
        }

        let mut outputs: Vec<Option<crate::retained::ChartCanvasOutput>> =
            Vec::with_capacity(self.members.len());
        for m in &self.members {
            let out = m.output.read(app, |_app, o| o.clone()).ok();
            outputs.push(out);
        }

        // First pass: clear ignore markers when the expected output change happened.
        for i in 0..self.members.len() {
            let Some(out) = outputs.get(i).and_then(|o| o.clone()) else {
                continue;
            };
            let mem = &mut self.memory[i];
            if mem.ignore_next_output_revision && out.revision != mem.last_output_revision {
                mem.last_output_revision = out.revision;
                mem.ignore_next_output_revision = false;
            }
        }

        // Second pass: pick a source member that changed (and is not suppressed).
        //
        // Heuristic: prefer a member that currently has an active axisPointer anchor, useful when
        // the pointer moves between charts.
        let mut source_index: Option<usize> = None;
        let mut source_events: Option<Vec<LinkEvent>> = None;
        let mut source_score: i32 = -1;

        for i in 0..self.members.len() {
            let Some(out) = outputs.get(i).and_then(|o| o.clone()) else {
                continue;
            };
            let mem = &self.memory[i];
            if out.revision == mem.last_output_revision || mem.ignore_next_output_revision {
                continue;
            }

            let events = out.snapshot.link_events.clone();
            let mut score = 0i32;
            if events
                .iter()
                .any(|e| matches!(e, LinkEvent::AxisPointerChanged { anchor: Some(_) }))
            {
                score += 10;
            }
            if events
                .iter()
                .any(|e| matches!(e, LinkEvent::DomainWindowChanged { .. }))
            {
                score += 5;
            }
            if events
                .iter()
                .any(|e| matches!(e, LinkEvent::BrushSelectionChanged { .. }))
            {
                score += 3;
            }

            if score > source_score {
                source_score = score;
                source_index = Some(i);
                source_events = Some(events);
            }
        }

        let Some(source_index) = source_index else {
            return false;
        };
        let source_events = source_events.unwrap_or_default();

        // Update last seen revision for the source immediately.
        if let Some(out) = outputs.get(source_index).and_then(|o| o.clone()) {
            self.memory[source_index].last_output_revision = out.revision;
        }

        let source_router = &self.members[source_index].router;

        let mut changed = false;

        if self.policy.axis_pointer {
            if let Some(anchor) = source_events.iter().rev().find_map(|e| match e {
                LinkEvent::AxisPointerChanged { anchor } => anchor.as_ref(),
                _ => None,
            }) {
                let next = source_router
                    .axis_key(anchor.axis)
                    .map(|axis| AxisPointerLinkAnchor {
                        axis,
                        value: anchor.value,
                    });

                let Ok(current) = self.axis_pointer.read(app, |_app, a| a.clone()) else {
                    return false;
                };
                if current != next {
                    let _ = self.axis_pointer.update(app, |a, _cx| {
                        *a = next;
                    });
                    changed = true;
                }
            } else if source_events
                .iter()
                .any(|e| matches!(e, LinkEvent::AxisPointerChanged { anchor: None }))
            {
                let Ok(current) = self.axis_pointer.read(app, |_app, a| a.clone()) else {
                    return false;
                };
                if current.is_some() {
                    let _ = self.axis_pointer.update(app, |a, _cx| {
                        *a = None;
                    });
                    changed = true;
                }
            }
        }

        if self.policy.domain_windows {
            let updates: Vec<(LinkAxisKey, Option<DataWindow>)> = source_events
                .iter()
                .filter_map(|e| match e {
                    LinkEvent::DomainWindowChanged { axis, window } => {
                        source_router.axis_key(*axis).map(|key| (key, *window))
                    }
                    _ => None,
                })
                .collect();

            if !updates.is_empty() {
                let Ok(current) = self.domain_windows.read(app, |_app, w| w.clone()) else {
                    return false;
                };
                let mut next = current.clone();
                for (key, window) in updates {
                    next.insert(key, window);
                }
                if next != current {
                    let _ = self.domain_windows.update(app, |w, _cx| {
                        *w = next;
                    });
                    changed = true;
                }
            }
        }

        if self.policy.brush {
            if let Some(selection) = source_events.iter().rev().find_map(|e| match e {
                LinkEvent::BrushSelectionChanged { selection } => Some(*selection),
                _ => None,
            }) {
                let next = selection.and_then(|sel| {
                    let x_key = source_router.axis_key(sel.x_axis)?;
                    let y_key = source_router.axis_key(sel.y_axis)?;
                    Some(BrushSelectionLink2D {
                        x_axis: x_key,
                        y_axis: y_key,
                        x: sel.x,
                        y: sel.y,
                    })
                });

                let Ok(current) = self.brush.read(app, |_app, s| *s) else {
                    return false;
                };
                if current != next {
                    let _ = self.brush.update(app, |s, _cx| {
                        *s = next;
                    });
                    changed = true;
                }
            }
        }

        if !changed {
            return false;
        }

        // Suppress the next output revision for all non-source members to avoid ping-pong.
        for i in 0..self.members.len() {
            if i == source_index {
                continue;
            }
            self.memory[i].ignore_next_output_revision = true;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use delinea::spec::{
        AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
    };

    fn spec_with_ambiguous_x_key() -> (ChartSpec, LinkAxisKey, AxisId, AxisId) {
        let chart_id = delinea::ids::ChartId::new(1);
        let dataset = DatasetId::new(1);
        let grid = delinea::ids::GridId::new(1);
        let x1 = AxisId::new(1);
        let x2 = AxisId::new(2);
        let y = AxisId::new(3);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);

        let key = LinkAxisKey {
            kind: AxisKind::X,
            dataset,
            field: x_field,
        };

        let spec = ChartSpec {
            id: chart_id,
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_field,
                        column: 1,
                    },
                ],

                from: None,
                transforms: Vec::new(),
            }],
            grids: vec![GridSpec { id: grid }],
            axes: vec![
                AxisSpec {
                    id: x1,
                    name: None,
                    kind: AxisKind::X,
                    grid,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                AxisSpec {
                    id: x2,
                    name: None,
                    kind: AxisKind::X,
                    grid,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                AxisSpec {
                    id: y,
                    name: None,
                    kind: AxisKind::Y,
                    grid,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(1),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis: x1,
                    y_axis: y,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(2),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis: x2,
                    y_axis: y,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        (spec, key, x1, x2)
    }

    #[test]
    fn explicit_axis_map_can_restore_unique_reverse_mapping() {
        let (spec, key, x1, _x2) = spec_with_ambiguous_x_key();
        let router = ChartLinkRouter::from_spec(&spec);
        assert_eq!(router.axis_for_key(key), None);

        let mut explicit = BTreeMap::new();
        explicit.insert(x1, key);
        let router = ChartLinkRouter::from_spec(&spec).with_explicit_axis_map(explicit);
        assert_eq!(router.axis_for_key(key), Some(x1));
    }

    #[test]
    fn duplicate_explicit_key_assignment_disables_reverse_mapping() {
        let (spec, key, x1, x2) = spec_with_ambiguous_x_key();
        let mut explicit = BTreeMap::new();
        explicit.insert(x1, key);
        explicit.insert(x2, key);

        let router = ChartLinkRouter::from_spec(&spec).with_explicit_axis_map(explicit);
        assert_eq!(router.axis_for_key(key), None);
    }
}
