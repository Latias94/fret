use fret_runtime::Model;
use fret_ui::UiHost;

use crate::retained::{PlotOutput, PlotOutputSnapshot, PlotState};

#[derive(Debug, Clone, Copy)]
pub struct PlotLinkPolicy {
    pub view_bounds: bool,
    pub query: bool,
    pub cursor_x: bool,
}

impl Default for PlotLinkPolicy {
    fn default() -> Self {
        Self {
            view_bounds: true,
            query: true,
            cursor_x: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkedPlotMember {
    pub state: Model<PlotState>,
    pub output: Model<PlotOutput>,
}

#[derive(Debug, Clone, Copy)]
struct LinkedPlotMemberMemory {
    last_output_revision: u64,
    ignore_next_output_revision: bool,
}

/// A simple coordinator that links interaction state across multiple plots.
///
/// This is intended to support common ImPlot-style workflows:
/// - Linked view bounds (pan/zoom) across multiple plots.
/// - Shared query selections across multiple plots.
/// - Multi-axis linking: when present, Y2/Y3/Y4 view bounds propagate alongside the primary view.
///
/// Design notes:
/// - The coordinator uses `PlotOutput.revision` as a change detector.
/// - When it applies state to other plots, their next output update is suppressed once to avoid
///   feedback loops (ping-pong).
/// - This assumes plots in the group share compatible domains (or at least can tolerate adopting
///   each other's view bounds).
#[derive(Debug)]
pub struct LinkedPlotGroup {
    policy: PlotLinkPolicy,
    members: Vec<LinkedPlotMember>,
    memory: Vec<LinkedPlotMemberMemory>,
}

impl LinkedPlotGroup {
    pub fn new(policy: PlotLinkPolicy) -> Self {
        Self {
            policy,
            members: Vec::new(),
            memory: Vec::new(),
        }
    }

    pub fn push(&mut self, member: LinkedPlotMember) -> &mut Self {
        self.members.push(member);
        self.memory.push(LinkedPlotMemberMemory {
            last_output_revision: 0,
            ignore_next_output_revision: false,
        });
        self
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn clear(&mut self) {
        self.members.clear();
        self.memory.clear();
    }

    /// Propagate interaction state if any member output changed since the previous tick.
    ///
    /// Recommended usage:
    /// - Call this after dispatching input events (e.g. after `ui.dispatch_event(...)`).
    /// - Optionally call this once per frame; it is cheap when nothing changes.
    pub fn tick<H: UiHost>(&mut self, app: &mut H) {
        if self.members.len() <= 1 {
            return;
        }

        let mut outputs: Vec<Option<PlotOutput>> = Vec::with_capacity(self.members.len());
        for m in &self.members {
            let out = m.output.read(app, |_app, o| *o).ok();
            outputs.push(out);
        }

        // First pass: clear "ignore next" markers if the expected output change happened.
        for i in 0..self.members.len() {
            let Some(out) = outputs.get(i).and_then(|o| *o) else {
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
        // Heuristic: prefer a member that currently has a cursor (useful when the pointer moves
        // between plots, where one plot clears cursor while another gains it in the same tick).
        let mut source_index: Option<usize> = None;
        let mut source_snapshot: Option<PlotOutputSnapshot> = None;
        let mut source_score: i32 = -1;
        for i in 0..self.members.len() {
            let Some(out) = outputs.get(i).and_then(|o| *o) else {
                continue;
            };
            let mem = &mut self.memory[i];
            if out.revision != mem.last_output_revision {
                mem.last_output_revision = out.revision;
                if !mem.ignore_next_output_revision {
                    let score = if out.snapshot.cursor.is_some() { 1 } else { 0 };
                    if score > source_score {
                        source_index = Some(i);
                        source_snapshot = Some(out.snapshot);
                        source_score = score;
                    }
                }
            }
        }

        let Some(source_index) = source_index else {
            return;
        };
        let Some(source_snapshot) = source_snapshot else {
            return;
        };

        for i in 0..self.members.len() {
            if i == source_index {
                continue;
            }
            let Some(out) = outputs.get(i).and_then(|o| *o) else {
                continue;
            };

            let state = &self.members[i].state;
            let policy = self.policy;
            use std::cell::Cell;
            let did_change = Cell::new(false);
            let should_ignore = Cell::new(false);
            let ok = state
                .update(app, |s, _cx| {
                    let r = apply_snapshot_to_plot_state(s, source_snapshot, policy);
                    did_change.set(r.changed);
                    should_ignore.set(r.should_ignore_output_once);
                })
                .is_ok();

            if ok && did_change.get() {
                let mem = &mut self.memory[i];
                mem.last_output_revision = out.revision;
                mem.ignore_next_output_revision = should_ignore.get();
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ApplyResult {
    changed: bool,
    should_ignore_output_once: bool,
}

fn apply_snapshot_to_plot_state(
    state: &mut PlotState,
    snapshot: PlotOutputSnapshot,
    policy: PlotLinkPolicy,
) -> ApplyResult {
    let mut changed = false;
    let mut should_ignore = false;

    if policy.view_bounds {
        if state.view_is_auto || state.view_bounds != Some(snapshot.view_bounds) {
            state.view_is_auto = false;
            state.view_bounds = Some(snapshot.view_bounds);
            changed = true;
            should_ignore = true;
        }

        // Only propagate optional axis bounds when the source plot provides them. This prevents a
        // plot that doesn't have a given axis enabled from clearing other members' axis state.
        if let Some(vb) = snapshot.view_bounds_y2
            && (state.view_y2_is_auto || state.view_bounds_y2 != Some(vb)) {
                state.view_y2_is_auto = false;
                state.view_bounds_y2 = Some(vb);
                changed = true;
                should_ignore = true;
            }
        if let Some(vb) = snapshot.view_bounds_y3
            && (state.view_y3_is_auto || state.view_bounds_y3 != Some(vb)) {
                state.view_y3_is_auto = false;
                state.view_bounds_y3 = Some(vb);
                changed = true;
                should_ignore = true;
            }
        if let Some(vb) = snapshot.view_bounds_y4
            && (state.view_y4_is_auto || state.view_bounds_y4 != Some(vb)) {
                state.view_y4_is_auto = false;
                state.view_bounds_y4 = Some(vb);
                changed = true;
                should_ignore = true;
            }
    }
    if policy.query
        && state.query != snapshot.query {
            state.query = snapshot.query;
            changed = true;
            should_ignore = true;
        }
    if policy.cursor_x {
        let next = snapshot.cursor.map(|c| c.x);
        if state.linked_cursor_x != next {
            state.linked_cursor_x = next;
            changed = true;
        }
    }

    ApplyResult {
        changed,
        should_ignore_output_once: should_ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct SimMember {
        mem: LinkedPlotMemberMemory,
        rev: u64,
    }

    fn tick_pick_source(members: &mut [SimMember]) -> Option<usize> {
        // Clear ignore markers.
        for m in members.iter_mut() {
            if m.mem.ignore_next_output_revision && m.rev != m.mem.last_output_revision {
                m.mem.last_output_revision = m.rev;
                m.mem.ignore_next_output_revision = false;
            }
        }

        // Pick the first changed member as the source (unless suppressed).
        for (i, m) in members.iter_mut().enumerate() {
            if m.rev != m.mem.last_output_revision {
                m.mem.last_output_revision = m.rev;
                if !m.mem.ignore_next_output_revision {
                    return Some(i);
                }
            }
        }
        None
    }

    #[test]
    fn ignores_next_output_after_propagation_to_prevent_ping_pong() {
        let mut members = vec![
            SimMember {
                mem: LinkedPlotMemberMemory {
                    last_output_revision: 0,
                    ignore_next_output_revision: false,
                },
                rev: 0,
            },
            SimMember {
                mem: LinkedPlotMemberMemory {
                    last_output_revision: 0,
                    ignore_next_output_revision: false,
                },
                rev: 0,
            },
        ];

        // User interacts with plot A.
        members[0].rev = 1;
        assert_eq!(tick_pick_source(&mut members), Some(0));

        // Propagation would mark plot B as suppressed for the next output bump.
        members[1].mem.ignore_next_output_revision = true;

        // Plot B output bumps due to the propagated state change; do not treat it as a source.
        members[1].rev = 1;
        assert_eq!(tick_pick_source(&mut members), None);
        assert_eq!(members[1].mem.ignore_next_output_revision, false);

        // Now a real user interaction on plot B should be observed.
        members[1].rev = 2;
        assert_eq!(tick_pick_source(&mut members), Some(1));
    }

    #[test]
    fn apply_snapshot_propagates_optional_axis_bounds_when_present() {
        let mut state = PlotState::default();
        assert!(state.view_is_auto);
        assert!(state.view_y2_is_auto);
        assert!(state.view_y3_is_auto);
        assert!(state.view_y4_is_auto);

        let snapshot = PlotOutputSnapshot {
            view_bounds: crate::cartesian::DataRect {
                x_min: 0.0,
                x_max: 10.0,
                y_min: -1.0,
                y_max: 1.0,
            },
            view_bounds_y2: Some(crate::cartesian::DataRect {
                x_min: 0.0,
                x_max: 10.0,
                y_min: 0.0,
                y_max: 100.0,
            }),
            view_bounds_y3: Some(crate::cartesian::DataRect {
                x_min: 0.0,
                x_max: 10.0,
                y_min: 500.0,
                y_max: 750.0,
            }),
            view_bounds_y4: Some(crate::cartesian::DataRect {
                x_min: 0.0,
                x_max: 10.0,
                y_min: 10_000.0,
                y_max: 12_000.0,
            }),
            cursor: None,
            hover: None,
            query: None,
            drag: None,
        };

        let r = apply_snapshot_to_plot_state(&mut state, snapshot, PlotLinkPolicy::default());
        assert!(r.changed);
        assert!(r.should_ignore_output_once);
        assert!(!state.view_is_auto);
        assert!(!state.view_y2_is_auto);
        assert!(!state.view_y3_is_auto);
        assert!(!state.view_y4_is_auto);
        assert_eq!(state.view_bounds, Some(snapshot.view_bounds));
        assert_eq!(state.view_bounds_y2, snapshot.view_bounds_y2);
        assert_eq!(state.view_bounds_y3, snapshot.view_bounds_y3);
        assert_eq!(state.view_bounds_y4, snapshot.view_bounds_y4);
    }

    #[test]
    fn apply_snapshot_does_not_clear_optional_axis_bounds_on_none() {
        let mut state = PlotState::default();
        state.view_y2_is_auto = false;
        state.view_bounds_y2 = Some(crate::cartesian::DataRect {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 10.0,
            y_max: 20.0,
        });

        let snapshot = PlotOutputSnapshot {
            view_bounds: crate::cartesian::DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            },
            view_bounds_y2: None,
            view_bounds_y3: None,
            view_bounds_y4: None,
            cursor: None,
            hover: None,
            query: None,
            drag: None,
        };

        let _ = apply_snapshot_to_plot_state(&mut state, snapshot, PlotLinkPolicy::default());
        assert_eq!(
            state.view_bounds_y2,
            Some(crate::cartesian::DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 10.0,
                y_max: 20.0,
            })
        );
    }
}
