# Docking N-ary Split Graph — Workstream (v1)

Status: Draft (workstream document; normative contracts live in ADRs)

This workstream upgrades Fret's docking graph semantics from a “binary-split only” behavior to a
**stable N-ary split model** (shares-based), inspired by `egui_tiles` and the “no deep split trees”
property of editor-grade dockers.

The goal is to reduce long-term refactor risk by removing the current pressure to introduce
ad-hoc UI-side stabilization for deeply nested split trees.

## Why now

Historically, edge-docking a panel wrapped the target in a new 50/50 binary split (creating a new
`DockNode::Split { children: [new, old] }`). Repeating the gesture produced progressively deeper
trees, which:

- increases the number of nodes touched by layout/hit-test/paint,
- increases the complexity of splitter drags (nested same-axis stabilization),
- increases merge conflicts (large interaction core + many implicit invariants),
- and makes future “editor constraints” (min sizes, locked groups, drop masks) harder to layer in
  cleanly.

Fret already has the right layering (ADR 0075), and the persisted schema (`DockLayout`) already
supports `children: Vec<_>` and `fractions: Vec<_>`. The missing part is **how ops are applied** and
**how the runtime tree is simplified**.

## Implementation status (core)

The core graph now enforces a canonicalized form after operations and upgrades edge docking
semantics to prefer insertion into an existing same-axis split when possible:

- Canonicalization lives in `crates/fret-core/src/dock/mutate.rs` (`simplify_window_forest`).
- Edge docking “insert instead of wrap” lives in `crates/fret-core/src/dock/mutate.rs`
  (`insert_edge_child_prefer_same_axis_split`).

This workstream remains active for the docking UI layer (preview geometry, splitter drags, and
reducing transitional stabilization in `ecosystem/fret-docking`).

## Goals

- Preserve “ImGui/Unity/Unreal-class” hand-feel as the baseline interaction vocabulary.
- Make edge-docking prefer **inserting into an existing same-axis split** rather than wrapping.
- Keep split trees shallow by enforcing a canonical simplified form (no nested same-axis splits).
- Introduce a place to add editor-grade constraints (min sizes, drop masks, locked groups) without
  polluting `fret-ui` (policy stays in `ecosystem/fret-docking`).
- Build regressions into gates using `fretboard diag` scripts and perf probes.

## Non-goals (v1)

- A “dockview-style” DOM layout manager: we are not adopting their implementation style.
- Replacing the docking UI authoring approach (retained bridge vs declarative) in this workstream.
- Adding a first-class grid docking layout to `fret-core` (optional follow-up).

## Existing contracts and layering (must remain true)

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Docking layering (B route): `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
- Docking arbitration: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window degradation policy: `docs/adr/0083-multi-window-degradation-policy.md`
- Viewport embedding and overlays stay app-owned:
  - `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
  - `ecosystem/fret-docking/src/dock/mod.rs` (`DockViewportOverlayHooks`)

## Reference anchors (non-normative)

These are useful for aligning design intent, not for copying code:

- `egui_tiles` (N-ary linear containers + shares + simplification rules):
  - Upstream: https://github.com/rerun-io/egui_tiles
  - Local (repo-ref checkout): `F:\SourceCodes\Rust\fret\repo-ref\egui_tiles\`
  - `repo-ref/egui_tiles/src/container/linear.rs` (`Linear`, `Shares`)
  - `repo-ref/egui_tiles/src/lib.rs` (`SimplificationOptions`, “two-pass layout + ui” note)
  - `repo-ref/egui_tiles/src/behavior.rs` (`Behavior` trait: overrideable policy surface)
- `dockview` (layout tree + panel state map separation; float/popout state organization):
  - Upstream: https://github.com/mathuo/dockview
  - Local (repo-ref checkout): `F:\SourceCodes\Rust\fret\repo-ref\dockview\`
  - `repo-ref/dockview/packages/dockview-core/src/dockview/dockviewComponent.ts`

## Current implementation map (evidence anchors)

Core graph and ops:

- `crates/fret-core/src/dock/mod.rs` (`DockGraph`, `DockNode`, `DropZone`)
- `crates/fret-core/src/dock/op.rs` (`DockOp`)
- `crates/fret-core/src/dock/apply.rs` (apply ops)
- `crates/fret-core/src/dock/mutate.rs` (graph mutation helpers; canonicalization + edge-insert semantics)
- `crates/fret-core/src/dock/layout.rs` (`DockLayout` schema, versioning, validation)

Docking UI and policy:

- `ecosystem/fret-docking/src/dock/space.rs` (`DockSpace` interaction core)
- `ecosystem/fret-docking/src/dock/layout.rs` (layout map; already N-ary friendly)
- `ecosystem/fret-docking/src/dock/hit_test.rs` (split handle hit-testing via `handle_hit_rects`)
- Canonical form keeps same-axis splits flat (legacy same-axis nested stabilization removed).
- `ecosystem/fret-docking/src/runtime.rs` (app/runner integration; tear-off fallbacks; close/merge)

Diagnostics and scripted repros:

- Docs: `docs/ui-diagnostics-and-scripted-tests.md`
- CLI: `apps/fretboard/src/diag.rs`
- Scripts: `tools/diag-scripts/*`

## Proposed design

### 0) Terminology: “fractions” vs “shares”

The current code uses `fractions: Vec<f32>` in `DockNode::Split`. For this workstream we treat this
vector as **shares**:

- values are non-negative,
- only relative magnitudes matter,
- and they are normalized as needed for layout and persistence.

We may keep the field name `fractions` for persistence stability, but internal APIs should prefer
“shares” vocabulary to avoid confusion (especially once N-ary insertion and min-size clamping are
implemented).

### 1) Keep `DockLayout` schema shape; change the *semantics* of how we apply ops

We keep the on-disk schema shape stable (no immediate bump required) because it already represents:

- `Split { children: Vec<u32>, fractions: Vec<f32> }`
- `Tabs { tabs: Vec<PanelKey>, active: usize }`
- window roots + floatings

We change the “edge docking” behavior so that the graph tends toward a canonical N-ary form.

### 2) Canonical form and invariants

Introduce an internal simplification step that ensures, after each `DockOp`:

- **No empty tabs**: `Tabs.tabs.len() > 0` (or the tabs node is removed).
- **No single-child splits**: `Split.children.len() >= 2`.
- **No nested same-axis splits**:
  - A `Split(axis = Horizontal)` must not have a child which is a `Split(axis = Horizontal)` unless
    we are in the middle of a transactional mutation step.
  - Same-axis adjacency is flattened into a single split node with N children.
- **Fractions/shares are normalized**:
  - `fractions.len() == children.len()`
  - all `fractions[i] >= 0`
  - sum is normalized to 1.0 (within a tolerance), and the last entry is adjusted to close drift.

This pushes “tree hygiene” into `fret-core` where it belongs (pure data), and reduces policy logic
in UI code.

Suggested simplify pipeline order (deterministic):

1. prune empty tabs (and bubble removal upward),
2. prune single-child splits,
3. flatten nested same-axis splits (“join nested linear containers”),
4. normalize shares (and clamp non-finite values),
5. fix up `active` indices for tab stacks.

Practical note:

- The simplification pass should be a pure data transform, deterministic, and cheap enough to run
  after each dock transaction.
- When debugging, it should be possible to *disable* simplification temporarily (debug-only) to
  understand which raw ops produced which shapes.

### 3) “Insert instead of wrap” rule for edge docking

When applying:

- `DockOp::MovePanel { target_tabs, zone: Left/Right/Top/Bottom, .. }`, or
- `DockOp::MoveTabs  { target_tabs, zone: Left/Right/Top/Bottom, .. }`,

prefer this behavior:

1. Find the nearest parent split container in the *same axis* implied by `zone`.
2. Insert a new tabs node (or moved tabs node) into that split at the correct index:
   - `Left/Top` inserts before the target subtree,
   - `Right/Bottom` inserts after the target subtree.
3. Update fractions by **splitting the target child’s share**, rather than resetting to 50/50.
   - Start with: `target_share`
   - Replace it with: `[target_share * (1.0 - k), target_share * k]` and insert the new node
     accordingly (where `k` is a policy default, e.g. 0.5).

Implementation default (v1):

- `k = 0.5` (split the anchor share in half).

Fallback:

- If no suitable same-axis parent exists, wrap the target in a new split (existing behavior), then
  immediately simplify (flatten nested same-axis splits, normalize fractions).

This is the key change that prevents deep trees from repeated edge docking.

### 3.1) Tree depth and “bounded complexity” expectations

We should be explicit about a testable property:

- In a layout with a single same-axis split container, repeated edge docking should keep the split
  node count **O(1)** (the container grows children), rather than **O(n)** via nesting.

This becomes a unit test in `fret-core` and a scripted diag regression in `fret-docking`.

### 3.2) Algorithm sketch (edge docking)

For an edge drop into `target_tabs` with `zone != Center`:

1. Map `zone` to `axis` (`Left/Right => Horizontal`, `Top/Bottom => Vertical`).
2. Find the “anchor subtree” to split/insert next to:
   - when dropping on a tab stack, the anchor is the tab stack node,
   - when dropping on an outer host region, the anchor is the root subtree for that window.
3. Try to find the nearest ancestor `Split(axis)` that already contains the anchor as a descendant:
   - if found: insert a new child into that split adjacent to the anchor child (by index),
   - if not found: create a new `Split(axis)` that wraps the anchor, then simplify.
4. Update shares by splitting the anchor child share:
   - `old_share = shares[anchor_index]`
   - replace with `old_share * (1-k)` and insert `old_share * k` for the new child (or vice-versa),
     where `k` is a default share ratio (start with `k=0.5`).

Notes:

- This logic must be used consistently by both `MovePanel` and `MoveTabs`.
- The preview model must reflect whether we are doing “insert” vs “wrap”.

### 4) Splitter drag semantics for N children

Splitter drags should update only the two adjacent children (i and i+1), preserving total share.

For a horizontal split:

- The handle at index `i` adjusts `share[i]` and `share[i+1]`.
- Other shares remain unchanged, then a normalize pass runs (to control drift).

Constraints hook (v1, docking policy layer):

- If children have minimum sizes, clamp the two shares so the computed pixel sizes do not drop
  below minima.

### 5) Constraints and editor policy hooks (owned by `ecosystem/fret-docking`)

We add a “policy seam” in the docking layer (not in `fret-ui`) to support editor-grade needs:

- `min_size` / `preferred_size` for panels (viewport panels in particular),
- drop masks (disallow certain zones),
- group locking (e.g. “no-drop-target”, “no-tear-off”),
- and “can tear off to OS window” gating (beyond platform capabilities).

These do not need to be persisted on day 1; they can be computed from panel kinds/roles.

Minimum “editor feel” constraints to implement early:

- viewport panels must have a minimum size clamp (both axes),
- tab stacks should have a minimum tab-bar + content height (prevents “collapsed to nothing”),
- floating window minimum size clamp (prevents un-draggable tiny floatings).

### 5.1) Policy seam (minimal API sketch)

We should introduce a small docking-layer policy interface so editor behavior does not leak into
`fret-core` or `fret-ui`:

- Location: `ecosystem/fret-docking` (service or trait object).
- Call sites:
  - during drop intent resolution (zone masks, tear-off permission),
  - during layout computation (min sizes),
  - during splitter drags (clamping).

Minimal sketch (names TBD):

- `panel_min_size(panel: &PanelKey) -> Option<Size>`
- `panel_preferred_size(panel: &PanelKey) -> Option<Size>`
- `allow_tear_off_to_os_window(panel: &PanelKey) -> bool`
- `drop_zone_mask(source: &PanelKey, target_tabs: DockNodeId) -> DockDropZoneMask`
  - where mask is a small struct/bitset covering `{Center, Left, Right, Top, Bottom}`.
- `group_lock_mode(tabs: DockNodeId) -> DockGroupLockMode`
  - e.g. `None | Locked | NoDropTarget | NoTearOff`.

Default behavior should be permissive to preserve current demos.

### 6) UI impact and compatibility with existing component/layout design

This workstream is intentionally **compatible** with the current Fret UI design:

- It changes `fret-core`’s pure data ops and invariants, not `fret-ui` mechanisms.
- `ecosystem/fret-docking` already computes split layouts with a generic `count` + `fractions` API
  via `resizable_panel_group::compute_layout`, which is naturally N-ary.
- Declarative components do not need to know whether the dock graph is binary or N-ary; they receive
  rectangles and input routing decisions from the docking layer.

The main required UI changes are:

- drop preview geometry must match the new commit semantics (no more “always half” preview),
- nested same-axis stabilization can be simplified or removed once tree depth is controlled,
- min-size constraints must be wired into layout computation and handle drag clamping.

## Undo/redo and “op volume” considerations

N-ary splits do not change the fundamental transaction model (ADR 0013), but they can increase the
frequency of split updates during drags.

Recommendations:

- Splitter drags should coalesce updates and emit `DockOp::SetSplitFractionsMany` at a controlled
  cadence (e.g. once per frame), rather than emitting multiple ops per pointer event.
- Keep the UI preview model pure and cheap so it can run during pointer-move without allocations.

## Migration from existing (binary-heavy) layouts

Even without bumping `DockLayout`, we may already have persisted layouts (internal dev) that were
produced by the binary-wrapping behavior.

Recommendation:

- On load (or after import), run the same canonical `simplify` pass once to:
  - prune empty/single-child nodes,
  - and flatten nested same-axis splits.

This makes old layouts naturally converge toward the N-ary canonical form without requiring a
schema migration.

## Persistence and versioning stance

Recommendation for v1:

- Do **not** bump `DockLayout` version if we only change how ops are applied and simplified, while
  preserving the schema shape.
- Bump the version (or reset to v1) only when we add new persisted fields (e.g. group locks,
  constraints, panel roles).

Rationale:

- Keeping persistence stable reduces churn while the semantics settle.
- You explicitly allow bumps/resets today (no users), so we can choose a clean cut later when
  persisting policy becomes a hard requirement.

## Diagnostics and perf gates (from day 1)

This workstream must be “fearless” in practice. Each milestone should add at least one gate.

### Scripted correctness gates (`fretboard diag`)

Target: `docking_arbitration_demo` is the best integration harness because it already covers:

- docking + viewport panels,
- overlays (non-modal + modal),
- and capture arbitration.

Plan:

- Add a small suite of scripts under `tools/diag-scripts/` that:
  - constructs a representative layout,
  - performs a deterministic drag (tab drag + edge drop + split handle drag),
  - captures a bundle,
  - and asserts invariants from the bundle (geometry, active tab, focus, no stuck capture).

Prefer invariants over pixels; capture screenshots only when needed.

Recommended diagnostics additions (tracked in the TODO, implemented):

- [x] A small dock graph stats snapshot (node count, max depth, split/tabs/floating counts).
  - Evidence:
    - `crates/fret-runtime/src/interaction_diagnostics.rs` (`DockGraphStatsDiagnostics`)
    - `ecosystem/fret-docking/src/dock/space.rs` (`dock_graph_stats_for_window`, published via `WindowInteractionDiagnosticsStore`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export: `UiDockGraphStatsDiagnosticsV1`)
- [x] An explicit “preview decision” field for drop hovers (`wrap_binary` vs `insert_into_split(...)`).
  - Evidence:
    - `crates/fret-runtime/src/interaction_diagnostics.rs` (`DockDropPreviewDiagnostics`, `DockDropPreviewKindDiagnostics`)
    - `ecosystem/fret-docking/src/dock/space.rs` (`compute_dock_drop_resolve_diagnostics` sets `preview`)
    - `crates/fret-diag-protocol/src/lib.rs` (`UiPredicateV1::DockDropPreviewKindIs`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (predicate evaluation + bundle export)

Suggested initial scripts (start small; grow into a suite):

- `tools/diag-scripts/docking-arbitration-demo-nary-preview-insert-into-existing-split.json`
- (follow-up) `tools/diag-scripts/docking-nary-splitter-drag-adjacent-only.json`
- (follow-up) `tools/diag-scripts/docking-nary-escape-cancels-drag-no-stuck-hover.json`

### Performance gates

Correctness changes can still regress perf (layout churn, hit-test cost, paint cost).

Plan:

- Add a perf probe script that performs repeated splitter drags and records:
  - frame time summary,
  - number of layout nodes visited,
  - and any debug counters exposed by docking.
- Gate with `fretboard diag perf` or the existing resize probe gates:
  - `tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3`

## Open questions (to lock down in the TODO/milestones docs)

- Do we want to bump `DockLayout` version now, or only when we add new persisted fields?
- Do we want N-ary splits only, or also a first-class grid container (later)?
- Which constraints are required for “editor feel” v1:
  - viewport min size,
  - locked groups,
  - no-drop-target zones,
  - no-tear-off policies?

## Acceptance criteria (v1)

This workstream is “done” when the following are true and gated:

Core invariants (unit-tested):

- After applying any dock op that changes structure, the graph simplifies to canonical form:
  - no nested same-axis splits,
  - no single-child splits,
  - no empty tab stacks,
  - `children.len() == shares.len()` for all splits,
  - shares are normalized and finite.

Edge docking semantics (unit-tested + diag-gated):

- Repeated edge docking into an existing same-axis split **inserts** rather than nests:
  - split node count does not grow linearly with the number of docked panels.

Preview/commit alignment (diag-gated):

- During a drag hover, diagnostics expose whether the preview decision is:
  - `wrap_binary` or `insert_into_split(axis, index)`,
  - and scripts assert that the decision matches the intended rule.

Splitter drags (unit-tested + diag-gated):

- Dragging handle `i` only changes shares for children `i` and `i+1` (modulo normalization drift).
- With min-size constraints enabled, handles clamp predictably with explainable diagnostics.

Perf (gated):

- A repeatable probe exists for:
  - splitter drags in a large layout,
  - and hover updates during a tab drag.
- Gates prevent regressions relative to an established baseline.

## Risks and mitigations

- Risk: preview geometry diverges from commit semantics.
  - Mitigation: encode the preview decision (“wrap vs insert”) in diagnostics and assert it in
    scripted gates.
- Risk: performance regressions from repeated parent/subtree scans.
  - Mitigation: compute parent maps once per op application; add perf probes for drag hover.
- Risk: min-size constraints create “stuck split handles”.
  - Mitigation: clamp adjacent shares with clear diagnostics (why clamped) and unit tests for edge
    cases.
