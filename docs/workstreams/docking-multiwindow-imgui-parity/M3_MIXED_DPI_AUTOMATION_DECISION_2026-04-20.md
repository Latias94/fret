# M3 Mixed-DPI Automation Decision - 2026-04-20

Status: historical decision note, superseded by
`M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md` and closed by
`M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`

Supersession note (2026-04-25):

- This note was correct when the docking lane still lacked a diagnostics-owned
  monitor-topology environment source and source-scoped campaign predicate.
- The separate diagnostics environment-predicate lane has since shipped
  `host.monitor_topology` + `host_monitor_topology` admission.
- The current stance is now captured in
  `M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`: keep the bounded P3 campaign generic, but allow
  a dedicated real-host mixed-DPI campaign with honest environment admission.
- The first accepted real-host run is recorded in
  `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`.

Related:

- `WORKSTREAM.json`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
- `M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`
- `docking-multiwindow-imgui-parity-todo.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_index.rs`
- `crates/fret-diag/src/bundle_index.rs`
- `crates/fret-diag/src/commands/dock_routing.rs`
- `crates/fret-diag/src/registry/campaigns.rs`
- `crates/fret-launch/src/runner/desktop/runner/window.rs`

## Purpose

`DW-P0-dpi-006` still had one open automation question after the acceptance posture and Windows
capture runbook landed:

> can the repo honestly auto-detect a mixed-DPI host well enough to add a bounded automated gate,
> or should mixed-DPI remain a manual acceptance proof until diagnostics grows a stronger
> environment contract?

This note freezes the current answer so the docking lane stops treating that question as implicit.

## Assumptions-first resume set

### 1) `bundle.json.env.scale_factors_seen` is not a host monitor-topology contract

- Area: diagnostics environment fingerprint
- Assumption: the current bundle-level `scale_factors_seen` field is derived from the last-known
  per-window snapshots seen during the run, not from an explicit monitor inventory.
- Evidence:
  - `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- Confidence: Confident
- Consequence if wrong: we would treat a run-observed window summary as if it were a portable host
  preflight signal.

### 2) `mixed_dpi_signal_observed` remains drag evidence, not a preflight capability

- Area: bounded routing evidence
- Assumption: `dock-routing` computes `observed_scale_factors_x1000` and
  `mixed_dpi_signal_observed` from the docking drag evidence already captured in the bundle; it
  does not prove the host was mixed-DPI before the run started.
- Evidence:
  - `crates/fret-diag/src/bundle_index.rs`
  - `crates/fret-diag/src/commands/dock_routing.rs`
  - `docs/ui-diagnostics-and-scripted-tests.md`
- Confidence: Confident
- Consequence if wrong: we would let a post-run behavior summary decide whether the run should have
  been scheduled in the first place.

### 3) Campaign manifests still only gate on stable `requires_capabilities`

- Area: diagnostics contract
- Assumption: diag campaigns currently support capability strings, not host-environment predicates
  such as "must have two monitors with distinct scale factors".
- Evidence:
  - `crates/fret-diag/src/registry/campaigns.rs`
  - `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- Confidence: Confident
- Consequence if wrong: we would add a one-off mixed-DPI branch without a stable contract for other
  environment-dependent lanes.

### 4) The runner can inspect monitors for behavior, but diagnostics does not export that as a reusable environment source yet

- Area: source availability
- Assumption: desktop runner code can enumerate monitors and use them for placement/fallback logic,
  but diagnostics bundles do not yet export a monitor-topology fingerprint that campaigns or scripts
  could consume as a first-class preflight input.
- Evidence:
  - `crates/fret-launch/src/runner/desktop/runner/window.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
  - `crates/fret-diag/src/registry/campaigns.rs`
- Confidence: Confident
- Consequence if wrong: this lane would miss an already-shipped contract and leave automation on the table unnecessarily.

## Findings

### 1) The current env fingerprint is too weak for honest mixed-DPI preflight

`bundle.json.env.scale_factors_seen` is useful explainability data, but it is not enough to answer:

- how many monitors were attached,
- which monitor each scale factor came from,
- whether two distinct scale factors existed before the script moved any window,
- or whether the host should have qualified for a mixed-DPI-only gate.

That means it is not a trustworthy skip/run predicate.

### 2) `dock-routing` is the right bounded proof surface, but only after a run exists

The current `dock-routing` output is already good enough for acceptance review:

- `observed_scale_factors_x1000`,
- `mixed_dpi_signal_observed`,
- `sf_run` / `sf_cur` / `sf_move`,
- `scr` / `scr_used` / `origin`,
- and hover-routing evidence.

That is exactly why it belongs in the manual acceptance flow.
It still cannot answer a campaign-level "should I run this mixed-DPI gate?" question before the run.

### 3) Adding `requires mixed-dpi` now would overstate the current contract

The lane would need at least two missing pieces before an honest automated gate exists:

1. a diagnostics-owned host monitor-topology/environment fingerprint,
2. and a campaign/script predicate contract that can branch on that environment source.

This lane has neither today. Adding an ad hoc manifest string or skip heuristic would turn
evidence-only signals into a fake capability contract.

### 4) The remaining open work in `DW-P0-dpi-006` is now only the real-host acceptance pair

After this decision, the docking lane no longer has two open mixed-DPI questions.
It has one:

- capture and record the Windows real-host `pre-crossing` + `post-crossing` acceptance pair.

If future pressure later demands automation, that should start as a diagnostics follow-on, not as a
small hidden tweak to this docking lane.

## Decision

From this point forward:

1. Do not add an automated mixed-DPI gate in this lane yet.
2. Keep the bounded P3 campaign generic and portable across single-monitor and mixed-DPI hosts.
3. Keep `mixed_dpi_signal_observed` and `scale_factors_seen` classified as bounded run evidence,
   not as host capability or campaign preflight signals.
4. Treat the real Windows mixed-DPI acceptance pair as the only remaining open proof item inside
   `DW-P0-dpi-006`.
5. If the repo later wants automated mixed-DPI gating, start a narrow diagnostics follow-on for:
   - monitor-topology environment fingerprinting,
   - and explicit campaign/script environment predicates.

## Immediate execution consequence

For this lane:

1. `M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md` remains the default next-runbook.
2. `docking-multiwindow-imgui-parity-todo.md` should now keep only the manual acceptance capture as
   the remaining open mixed-DPI subtask.
3. `DW-P1-win-002` and `DW-P1-linux-003` stay as the next follow-up slices after the real-host
   acceptance note lands.

## Recommended next slice

The next landable slice after this decision is one of these:

1. capture the real Windows mixed-DPI acceptance pair and add a dated evidence note for it, or
2. if that capture reveals a reusable diagnostics environment source after all, start a narrow
   diagnostics follow-on instead of mutating this docking lane in place.
