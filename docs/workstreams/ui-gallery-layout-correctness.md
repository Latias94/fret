# UI Gallery Layout Correctness — Tracker

Status: Draft (workstream note; ADRs remain the source of truth)

This document tracks **layout correctness** (wrong bounds, wrong sizing, wrong clipping) issues found in
`apps/fret-ui-gallery`, with a bias toward bugs that are **visually severe** and **deterministically reproducible**.

For performance investigations, see `docs/workstreams/ui-gallery-perf-scroll-measure.md`.

## 0) Goal

- Turn “looks wrong” reports into **repeatable repro bundles** + **minimal test cases**.
- Fix root causes in the correct layer (policy vs mechanism).
- Prevent regressions with scripted repros and/or unit tests.

## 1) Triage Checklist (Layout vs Visual)

Before digging into code, classify the problem:

1. **Layout issue**: bounds are wrong (hit-testing, clipping, scroll extents, overlays anchored to the wrong rect).
2. **Visual issue**: bounds are correct, but rendering is wrong (transform stack, clip/scissor, effect bounds).

Reference workflow: `docs/debugging-playbook.md`.

## 2) Collection Workflow (Preferred)

### 2.1 Capture a deterministic diagnostics bundle (recommended)

- Use `fretboard diag run` with an existing script, or add a new `tools/diag-scripts/ui-gallery-*.json`.
- If the bug is visual, capture pixels too:
  - `FRET_DIAG_SCREENSHOT=1`: enable screenshot readback and write `frame.bmp` into the most recent bundle dir when a script requests it (via `capture_screenshot`) or when dumping bundles (writes `screenshot.request`).

### 2.2 Dump the solved layout tree (when bounds are wrong)

```powershell
$env:FRET_TAFFY_DUMP=1
$env:FRET_TAFFY_DUMP_ONCE=1
$env:FRET_TAFFY_DUMP_DIR=".fret\\taffy-dumps"

# Prefer filtering by a stable semantics label when possible:
$env:FRET_TAFFY_DUMP_ROOT_LABEL="Debug:ui-gallery:resizable-panels"
```

Notes:

- Prefer a stable `SemanticsProps.label` or `test_id`-adjacent label wrapper around the region.
- Avoid dumping in perf-critical runs (dumps will stutter).

### 2.3 Compare two bundles (regressions / “only happens with toggle X”)

Use `fretboard diag compare`:

- If you want correctness only (ignoring bounds), add: `--compare-ignore-bounds`.
- If the bug is bounds-related, keep bounds enabled and set a strict epsilon: `--compare-eps-px <px>`.

## 3) Active Issues

| ID | Severity | Page | Symptom | Repro Script | Status | Owner | Notes |
|---|---:|---|---|---|---|---|---|
| L1 | P0 | `resizable` | Resizable panel group height looks wrong (ignores the intended `h_px(320)` in the demo; can collapse to ~0 when parent height is indefinite). | `tools/diag-scripts/ui-gallery-resizable-initial-bundle.json` | Fixed (pending merge) | codex | Evidence bundles: `.fret/diag-resizable-bundle/1769674921459-ui-gallery-resizable-initial/frame.bmp` (before) vs `.fret/diag-resizable-bundle/1769675245065-ui-gallery-resizable-initial/frame.bmp` (after). Debug label: `SemanticsProps.label="Debug:ui-gallery:resizable-panels"`, `test_id="ui-gallery-resizable-panels"`. |
| L2 | P0 | `intro` | “Core / UI Kit / Shadcn” preview cards (and the note) can be laid out wider than the window (tab panel expands to max-content width). | `tools/diag-scripts/ui-gallery-intro-preview-width-bundle.json` | Fixed (pending merge) | codex | Debug anchors: `label="Debug:ui-gallery:intro:preview-grid"`, `test_id="ui-gallery-intro-preview-grid"` and `label="Debug:ui-gallery:intro:preview-note"`, `test_id="ui-gallery-intro-preview-note"`. |

### L1 Notes (Resizable Panels)

- Root cause: `fret-ui-shadcn` `ResizablePanelGroup` unconditionally overwrote the caller’s size with
  `Fill`/`Fill` (percentage-like sizing). In an auto-height parent, the Fill height can resolve to ~0.
- Fix: only default to Fill when the caller did not specify width/height; do not force
  `props.layout.size` to Fill/Fill.
- Regression: `ecosystem/fret-ui-shadcn/tests/resizable_panel_group_layout.rs`.

### L2 Notes (Intro Preview Width / Tabs TabPanel)

- Root cause: `fret-ui-shadcn` `TabsContent` / `TabPanel` was auto-sized (shrink-wrapped), so
  max-content descendants could force the tab panel wider than its parent.
- Fix: make `TabsContent` default to `w_full().min_w_0()` (and keep optional `flex_1()` when
  `content_fill_remaining` is enabled).
- Regression: keep `tools/diag-scripts/ui-gallery-intro-preview-width-bundle.json` anchored via
  `ui-gallery-intro-preview-grid` / `ui-gallery-intro-preview-note` test IDs.

## 4) Next Actions

- Pick the top P0 issue and add a dedicated `tools/diag-scripts/ui-gallery-...json` repro.
- Add `SemanticsProps.test_id` / labels to make the hot region and broken bounds discoverable in bundles and dumps.
- Convert the repro into a `crates/fret-ui/src/declarative/tests/*` test when possible.
- Keep a small layout-only suite runnable via `fretboard diag suite ui-gallery-layout --launch -- cargo run -p fret-ui-gallery --release`.

## 5) Work Plan (TODOs + Milestones)

This workstream is intentionally **demo-driven** and **contract-aware**:

- Prefer fixing issues at the correct layer (mechanism vs policy) to avoid scattering one-off callsite tweaks.
- Prefer **deterministic scripted repros** over manual “it looks wrong” reports.
- Use the smallest amount of diagnostics metadata needed (`test_id` + optional label wrappers).

### 5.1 Milestones

| Milestone | Outcome | Exit Criteria |
|---|---|---|
| M0 | A repeatable layout regression harness | `fretboard diag suite ui-gallery-layout` passes locally and produces bundles + `frame.bmp` on demand. |
| M1 | High-risk layout invariants audited (shadcn ecosystem) | A short list of “layout invariants” is codified (see 5.2) and every relevant container component is checked for defaults. |
| M2 | P0 issues converted into repro scripts | Every P0 issue has: (1) script, (2) hot-region anchors, (3) before/after evidence bundle dirs recorded in this doc. |
| M3 | Regression prevention | Each fixed P0 has either a unit test (preferred) or a dedicated script in `ui-gallery-layout` suite (acceptable fallback). |
| M4 | Ongoing maintenance | New layout regressions are filed into this tracker first (ID + severity + script) before deeper refactors. |

### 5.2 Layout Invariants (tailwind-aligned “risk semantics”)

These are the most common “editor UI” failure modes, expressed in tailwind language with the intended Fret meaning.

**Invariant A — Flex children must be allowed to shrink**

- Tailwind: `min-w-0` / `min-h-0`
- Fret rule: any `flex_1()` child that can contain long text, grids, or scroll views should default to `min_w_0()` (and `min_h_0()` for vertical flex) unless there is a strong reason not to.
- Typical symptom when broken: tab panels / cards / columns expand to max-content and overflow the window.

**Invariant B — Do not force-fill caller sizing**

- Tailwind: `w-full` / `h-full` defaults that override user intent
- Fret rule: components may choose sensible defaults, but should not unconditionally overwrite the caller’s explicit size constraints.
- Typical symptom when broken: “height collapses to ~0 in auto-height parents” or “unexpected full-window stretching”.

**Invariant C — No-wrap text must be boxed in**

- Tailwind: `text-nowrap` + `truncate`
- Fret rule: if a subtree introduces `TextWrap::None` / ellipsis behavior, it must live under a width-constrained container (`w_full + min_w_0` in flex contexts, or explicit max width).
- Typical symptom when broken: a single label can blow up the layout width of an entire panel.

**Invariant D — Scroll areas need a min constraint in flex**

- Tailwind: `min-h-0` (the classic flex + scroll gotcha)
- Fret rule: scroll viewports inside `flex` should generally have `min_h_0()` so the viewport can actually become smaller than its content.
- Typical symptom when broken: scroll areas refuse to scroll, or force parent height to grow, or collapse.

### 5.3 TODO (Near-term)

- [ ] Add/maintain a “P0-first” issue queue in this doc (table in section 3 stays authoritative).
- [ ] Audit shadcn containers for Invariants A–D:
  - Tabs content / panels
  - Scroll areas used inside flex stacks
  - Cards/dialogs/sheets/popovers content wrappers
  - Resizable/split panel wrappers and handle rows
- [ ] Add a lightweight “page-sweep” script that visits a few core pages and asserts `bounds_within_window` for `ui-gallery-page-*` roots.
- [ ] For each new P0 issue:
  - [ ] Add a `tools/diag-scripts/ui-gallery-...json` repro (navigate + wait_until + bounds assertions + `capture_bundle` + `capture_screenshot`).
  - [ ] Add minimal hot-region `test_id` anchors (prefer `cx.semantics` wrappers; do not overload a11y labels).
  - [ ] Capture before/after evidence bundle dirs and record them under the issue row.
- [ ] When a fix changes a shadcn default, update the corresponding snapshot tests if they are intentional (document the reasoning in the PR/commit message).

### 5.4 TODO (Mid-term)

- [ ] Expand `fretboard diag suite ui-gallery-layout` as P0 issues grow (keep it small; no perf tortures here).
- [ ] Convert script-only regressions into unit tests where feasible (pure layout / semantics assertions).
- [ ] Document “known safe defaults” per container component (one paragraph each) to prevent future drift.
