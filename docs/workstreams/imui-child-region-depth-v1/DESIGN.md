# ImUi Child Region Depth v1

Status: closed closeout reference
Last updated: 2026-04-22

Status note (2026-04-22): this document remains the lane-opening rationale. The shipped verdict now
lives in `M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md` and `CLOSEOUT_AUDIT_2026-04-22.md`.
References below to implementation-heavy child-depth work should be read as opening-state
rationale rather than an active execution queue.

Related:

- `M1_TARGET_SURFACE_FREEZE_2026-04-22.md`
- `M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
- `CLOSEOUT_AUDIT_2026-04-22.md`
- `M0_BASELINE_AUDIT_2026-04-22.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

This lane exists because the closed collection/pane proof lane already proved that the current
generic `child_region` seam can host real pane-first composition. What remains open is no longer
"can pane composition work at all?" but the narrower `BeginChild()`-scale depth question:

> which child-region semantics are important enough to admit into generic `fret-ui-kit::imui`,
> and which ones should stay shell-owned or product-owned instead of turning `child_region` into a
> large flag bag?

## Why this is a new lane

This work should not be forced back into `imui-collection-pane-proof-v1`.

That folder is already closed on a no-helper-widening verdict for proof breadth.
Reopening it would blur two different questions:

- proof breadth
  - already closed by the asset-browser and shell-mounted pane proofs;
- child-region depth
  - still open as a narrower generic-helper design question.

This also should not be mixed with the closed menu/tab policy lane or the active docking
multi-window parity lane.
The current evidence is narrower than that.

## Assumptions-first baseline

### 1) The pane-first proof question is already closed; the remaining gap is helper depth.

- Evidence:
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would accidentally reopen the proof-breadth question and drift into demo churn.

### 2) The current generic helper is intentionally thin compared with Dear ImGui `BeginChild()`.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/child_region.rs`
  - `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
  - `repo-ref/imgui/imgui.h`
  - `repo-ref/imgui/imgui.cpp`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would overstate the remaining gap and widen surface that already exists.

### 3) Embedded menu composition is no longer the blocker for child regions.

- Evidence:
  - `ecosystem/fret-imui/src/tests/composition.rs`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would chase a child-specific menu mechanism instead of the actual depth question.

### 4) `crates/fret-ui` must stay unchanged unless ADR-backed evidence proves a mechanism gap.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `docs/architecture.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would turn into runtime widening instead of an ecosystem helper decision.

### 5) Not every Dear ImGui child flag deserves a generic Fret equivalent.

- Evidence:
  - `repo-ref/imgui/imgui.h`
  - `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would either clone upstream API shape too literally or reject useful depth too
    aggressively.

## Goals

1. Freeze which `BeginChild()`-scale concerns are credible candidates for generic
   `child_region[_with_options]` admission.
2. Keep the current pane-first proof surfaces explicit while avoiding a new pane-only demo by
   default.
3. Decide whether one bounded `ChildRegionOptions` slice is justified, or whether the correct
   outcome is a no-new-generic-surface verdict.
4. Leave one repro package, one gate package, and one evidence set for the lane.

## Non-goals

- Reopening collection-first or pane-first proof breadth.
- Adding a new pane-only proof demo or a narrower pane-only diagnostics path by default.
- Widening `crates/fret-ui`.
- Reopening menu/tab policy depth.
- Reopening key-owner / collection keyboard-owner work.
- Reopening shell-helper promotion.
- Reopening runner/backend multi-window parity.

## Initial target surface

The current helper is intentionally small.
`ChildRegionOptions` currently exposes only:

- `layout`,
- `scroll`,
- `test_id`,
- and `content_test_id`.

`child_region_element(...)` then builds:

- one keyed scroll area,
- one default vertical content flow,
- one framed card-like style,
- and coarse clipping through the scroll-area substrate.

The current credible depth buckets are therefore:

1. Frame and padding policy
   - the current helper always renders a framed card-like surface, while Dear ImGui distinguishes
     bare child windows from `FrameStyle`.
2. Axis-specific resize
   - Dear ImGui exposes `ResizeX` / `ResizeY`; Fret currently has no generic resize admission on
     `child_region`.
3. Axis-specific auto-resize
   - Dear ImGui exposes `AutoResizeX` / `AutoResizeY` plus `AlwaysAutoResize`, with explicit
     clipping/perf tradeoffs; Fret currently has no comparable generic contract.
4. Focus and navigation boundary posture
   - Dear ImGui exposes `NavFlattened`; Fret currently keeps child-region focus behavior implicit.
5. Visibility / clipping posture
   - Dear ImGui's `BeginChild()` return contract and coarse clipping behavior are part of the
     depth story; Fret currently exposes only the declarative child helper, not an early-out style
     return contract.

The lane does not assume that all five buckets should ship.
M1 should first decide which of them are even worth generic admission.

## Default owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- any additive `child_region` facade/options surface,
- the default generic child-region style/policy that remains worth sharing,
- and the generic contract language for child-depth semantics that stay below shell/product.

### `ecosystem/fret-imui`

Owns:

- focused composition/interaction proof for the admitted child-region floor,
- and regression proof that embedded composition still works after any bounded helper change.

### `apps/fret-examples`

Owns:

- the first-party pane-first proof surfaces,
- source-policy tests that keep the lane and its owner split explicit,
- and any future proof promotion if a new dedicated surface becomes unavoidable.

### `ecosystem/fret-workspace`

Owns:

- workbench-shell composition above generic `child_region`,
- editor-grade tab/toolbar/inspector choreography,
- and any shell-only pane behavior that should not become a generic helper contract.

### Not owned here

- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- collection breadth and keyboard-owner depth
  - still separate from this lane.
- menu/tab policy
  - already closed in its own narrow lane for this cycle.
- runner/backend multi-window hand-feel
  - remains in `docs/workstreams/docking-multiwindow-imgui-parity/`.

## First landable target

Do not start with resize or auto-resize implementation immediately.
The first landable target should be a target-surface freeze:

1. decide which `BeginChild()`-scale concerns are truly generic,
2. reject the ones that should stay shell/product-owned,
3. and admit at most one narrow option family if the first-party proof is strong enough.

If the evidence is still too thin after M1, the correct M2 outcome is a no-new-generic-surface
verdict rather than a wide `ChildRegionFlags` clone.
