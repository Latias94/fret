# M1 Mixed-DPI Acceptance Posture - 2026-04-13

Status: historical next-slice decision, superseded where noted by
`M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md` and closed by
`M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`

Supersession note (2026-04-25):

- The posture to keep the bounded P3 campaign generic remains active.
- The "no mixed-DPI campaign yet" portion is superseded because diagnostics now has
  `host.monitor_topology` environment admission.
- The current real-host gate shape lives in `M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`.
- The real-host acceptance evidence lives in
  `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`; `DW-P0-dpi-006` is no longer open.

Related:

- `WORKSTREAM.json`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `docking-multiwindow-imgui-parity-todo.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json`

## Purpose

`DW-P0-dpi-006` was still open for two different reasons:

1. the lane still needed one real mixed-DPI acceptance pair on actual hardware,
2. and the repo still had no explicit decision about whether mixed-DPI should be modeled as a
   first-class diagnostics capability or remain a manual/evidence-driven posture for now.

This note freezes the current answer so the lane can keep moving without inventing a premature
campaign/schema branch.

## Assumptions-first resume set

### 1) Keep the existing bounded P3 campaign generic

- Area: gate shape
- Assumption: the current bounded P3 package should keep using
  `docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json` as a generic
  large-coordinate stress entry that still passes on single-monitor hosts.
- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
  - `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
- Confidence: Confident
- Consequence if wrong: this lane would split the bounded P3 package into host-specific branches too early.

### 2) `mixed_dpi_signal_observed` is evidence, not a host capability contract

- Area: diagnostics semantics
- Assumption: the new `dock-routing` top-level `observed_scale_factors_x1000` /
  `mixed_dpi_signal_observed` summary is useful evidence for routing review, but it does not mean
  the repo can already claim a reliable "this host is mixed-DPI" capability.
- Evidence:
  - `crates/fret-diag/src/bundle_index.rs`
  - `crates/fret-diag/src/commands/dock_routing.rs`
  - `docs/ui-diagnostics-and-scripted-tests.md`
- Confidence: Confident
- Consequence if wrong: we would overstate a bounded evidence summary into a portable gating contract.

### 3) Historical: do not add a new `requires mixed-dpi` campaign or script schema key yet

- Area: diagnostics contract
- Assumption: current diag manifests only know stable capabilities such as `diag.script_v2` and
  `diag.multi_window`; mixed-DPI host topology is not yet modeled as a first-class capability source.
- Evidence:
  - `crates/fret-diag/src/registry/campaigns.rs`
  - `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- Confidence: Confident
- Consequence if wrong: we would add a one-off gate key without a stable environment fingerprint behind it.

### 4) Real-host mixed-DPI acceptance remains the missing landable proof

- Area: acceptance posture
- Assumption: `DW-P0-dpi-006` stays open until one real mixed-DPI acceptance pair is captured and
  the automation decision is explicit.
- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - `docs/workstreams/standalone/docking-multi-window-imgui-alignment-v1.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence: Confident
- Consequence if wrong: the lane would claim closure with only synthetic or host-agnostic stress evidence.

## Findings

### 1) The current bounded P3 package already has the right generic stress entry

The repo does not need a second mixed-DPI-specific campaign just to keep moving.

The current `docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
script already does the right bounded thing:

- it stresses large outer-window movement,
- it still passes on single-monitor hosts,
- and it surfaces scale-factor evidence when mixed-DPI is actually observed.

### 2) The missing piece was posture, not another script

The repo now exposes `observed_scale_factors_x1000` and `mixed_dpi_signal_observed` in
`dock-routing`, which is enough to make manual mixed-DPI review much less ambiguous.

That still does not answer whether campaigns can skip or branch on mixed-DPI automatically.
Right now, they cannot do that honestly without a real environment/capability source.

### 3) Real mixed-DPI acceptance should be treated as a two-bundle proof pair

For this lane, the minimum real-host acceptance artifact is:

1. one "pre-crossing" bundle while the drag is still on the source monitor,
2. one "post-crossing" bundle after the floating window crosses into the second monitor,
3. and paired `dock-routing` summaries that show whether mixed-DPI signal was actually observed.

That is the current proof floor for closing the manual side of `DW-P0-dpi-006`.

## Decision

From this point forward:

1. Keep the existing bounded P3 campaign generic.
   The current large-outer-move script remains part of the bounded package and must continue to pass
   on single-monitor hosts.
2. Treat `mixed_dpi_signal_observed` as evidence, not as a host capability contract.
   It is a bounded report summary, not yet a portable skip/branch signal.
3. Historical: do not add a new `requires mixed-dpi` campaign or script schema key yet.
   If future automation needs this, it should come from an explicit diagnostics environment or
   capability source, not from an ad hoc manifest string.
4. `DW-P0-dpi-006` stays open until both the real-host acceptance pair and the automation decision
   are explicit.

## Immediate execution consequence

For this lane:

1. the next real proof task is to capture a real-host acceptance pair with "pre-crossing" and
   "post-crossing" bundles,
2. `fretboard-dev diag dock-routing <bundle_dir|bundle.schema2.json>` should be used to record
   `scale_factors_seen` and `mixed_dpi_signal_observed` for both bundles,
3. the local monitor-scale sweep debug script remains the easiest bounded helper for host setup and
   bundle capture,
4. and any future auto-gate proposal must first explain where mixed-DPI host detection lives as a
   diagnostics contract.

## Recommended next slice

The next landable slice after this note should be one of these, in this order:

1. add a dated evidence note for the real-host mixed-DPI acceptance pair, or
2. if that work uncovers a clear reusable environment source, start a narrow diagnostics follow-on
   for mixed-DPI host/capability detection instead of mutating campaign manifests ad hoc.
