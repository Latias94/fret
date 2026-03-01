# `fret-node` Declarative-First Fearless Refactor (v1) — TODO

Keep this list concrete. Prefer tasks that can land independently with a single gate.

## Cross-cutting: contract gap log (must stay current)

- [ ] Maintain a short “contract gap log” section in this file:
  - what is missing,
  - which milestone is blocked,
  - can it be solved in ecosystem (preferred) or does it require a runtime ADR.

## Cross-cutting: evidence discipline

- [ ] For every milestone task that changes behavior/perf, add:
  - [ ] 1–3 evidence anchors (file + function/module name),
  - [ ] 1 regression artifact (test and/or diag script and/or perf counter gate).

## M0 — Baseline + gates

- [ ] Identify the smallest runnable node-graph demo target (native-first).
- [ ] Add one scripted regression artifact:
  - [ ] `fretboard diag` script for pan/zoom + pointer capture cancellation, or
  - [ ] integration test for input mapping invariants.
- [ ] Document current retained-only hot spots and why they exist (perf vs missing contracts).

Suggested starting artifacts (already present in this worktree):

- Gate (Rust): `ecosystem/fret-node/src/ui/canvas/widget/tests/escape_cancel_releases_pointer_capture_conformance.rs`
- Repro (diag script): `tools/diag-scripts/node-graph/node-graph-pan-middle-escape-cancel.json` (asserts `panning true` → `panning false` via viewport semantics `value`)

Suggested run commands (Windows-friendly):

- Run the demo (auto-enables the feature gate):
  - `cargo run -p fretboard -- dev native --bin node_graph_demo`
- Run the repro script and capture a diagnostics bundle:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-pan-middle-escape-cancel.json --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`

## M1 — Declarative surface skeleton

- [ ] Define the declarative “surface” API (public) that does not expose retained types.
- [ ] Implement `Canvas` paint pass for:
  - [ ] grid/background
  - [ ] edges (initial)
  - [ ] node chrome (initial)
- [ ] Introduce externalized render-data caches:
  - [ ] stable key strategy (node/edge ids + style keys)
  - [ ] cache invalidation by revision + viewport + scale factor
- [ ] Add cache observability counters (prepares/hits/evictions) for tuning.
- [ ] Add one “steady-state” gate:
  - [ ] fixed viewport + idle frames do not rebuild heavy render data.

## M2 — Interaction + portals

- [ ] Migrate selection + marquee to declarative input wiring + model reducers.
- [ ] Port node “portal” host to declarative elements for the visible subset.
- [ ] Move overlays (menus/rename/toolbars) to ecosystem overlay policy surfaces.
- [ ] Add one cancellation gate:
  - [ ] PointerCancel during drag clears pressed/drag state and releases capture.

## M3 — Defaults and compatibility

- [ ] Remove `unstable-retained-bridge` from `fret-node` default features.
- [ ] Keep retained implementation behind an explicit `compat-retained-canvas` (or similar) feature.
- [ ] Add a short “Ecosystem authoring guide” section describing:
  - [ ] when to use declarative canvas composition,
  - [ ] when a compatibility path is justified.

## Contract gap log (living)

- (none yet)
