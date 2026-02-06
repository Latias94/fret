# UI Gallery Layout Correctness — Tracker

Status: Complete (baseline established; ongoing maintenance only)

This document tracks **layout correctness** (wrong bounds, wrong sizing, wrong clipping) issues found in
`apps/fret-ui-gallery`, with a bias toward bugs that are **visually severe** and **deterministically reproducible**.

For performance investigations, see `docs/workstreams/ui-gallery-perf-scroll-measure.md`.

## 0) Goal

- Turn “looks wrong” reports into **repeatable repro bundles** + **minimal test cases**.
- Fix root causes in the correct layer (policy vs mechanism).
- Prevent regressions with scripted repros and/or unit tests.

## 0.1 Milestones (This Workstream)

- M0 (done): Gate the most severe P0 layout regressions in `fretboard diag suite ui-gallery-layout`.
- M1 (done): Expand correctness coverage with a small number of additional “high-risk” gates (prefer page roots + 1–3 critical controls).
- M2 (later): Build a separate track for **visual (non-layout)** artifacts (stale paint / cache invalidation / scissor) with screenshot-based evidence and targeted checks.

## 1) Triage Checklist (Layout vs Visual)

Before digging into code, classify the problem:

1. **Layout issue**: bounds are wrong (hit-testing, clipping, scroll extents, overlays anchored to the wrong rect).
2. **Visual issue**: bounds are correct, but rendering is wrong (transform stack, clip/scissor, effect bounds).

Reference workflow: `docs/debugging-playbook.md`.

## 1.1 Severity Rubric (Visual Impact)

Severity is about **user-visible breakage** (not internal correctness). Prefer concrete, falsifiable criteria:

| Severity | Definition | Typical Examples | “Done” Criteria |
|---:|---|---|---|
| P0 | Visually broken or blocks interaction in common window sizes. | Critical controls clipped/off-window; a panel/card overflows the window; scroll viewport can’t scroll; overlays anchor to the wrong rect; hit-test region is wrong. | Deterministic repro script + minimal fix + regression gate (script or unit test). |
| P1 | Noticeable but not blocking; workaround exists. | Minor clipping, awkward wrapping, suboptimal spacing, rare small-window overflow. | Repro + fix (may be deferred), keep tracked. |
| P2 | Cosmetic / polish only. | Minor alignment / color / typography nits. | Fix opportunistically; do not expand suite scope for these. |

Notes:

- Prefer “small window” as the default stress case (`800×600` or `960×540`) because most editor UI failures show up there first.
- Treat “hard to repro” as a signal: spend 5–10 minutes to make the repro deterministic (add anchors / stabilize the script) before touching layout code.

## 2) Collection Workflow (Preferred)

### 2.1 Capture a deterministic diagnostics bundle (recommended)

- Use `fretboard diag run` with an existing script, or add a new `tools/diag-scripts/ui-gallery-*.json`.
- If the bug is visual, capture pixels too:
  - `FRET_DIAG_SCREENSHOT=1`: enable screenshot readback and write `frame.bmp` into the most recent bundle dir when a script requests it (via `capture_screenshot`) or when dumping bundles (writes `screenshot.request`).
- If you need to inspect `SemanticsProps.test_id` / `label` in exported `bundle.json`, disable text redaction:
  - `FRET_DIAG_REDACT_TEXT=0` (default is redaction enabled).

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
| L3 | P0 | `workspace` | Workspace top bar can clip critical controls: the right-side “Command palette” button and tab-strip scroll buttons can end up off-window when tab strip overflows (missing `min-w-0`/shrink constraints on flex/scroll containers). | `tools/diag-scripts/ui-gallery-topbar-command-palette-visible.json` | Fixed (pending merge) | codex | Evidence: `target/fret-diag/1769692404766-ui-gallery-topbar-command-palette-visible/frame.bmp`. Regression gate: `bounds_within_window(test_id=\"ui-gallery-command-palette\", eps_px=1.0)`. |
| L4 | P1 | `gallery` | In small windows (e.g. 800×600), the content header can squeeze the title column too much (page title wraps per-character). | (manual) | Mitigated | codex | Mitigation: truncate the page title (`nowrap + ellipsis`) to avoid “vertical text” when the right-side action row is wider than the remaining space. Follow-up: make the header responsive (wrap / overflow menu). |
| L5 | P0 | `data_grid` | DataGrid header row overlapped with body rows (rows painted under the header). | `tools/diag-scripts/ui-gallery-layout-sweep-extended.json` (data_grid step) | Fixed (pending merge) | codex | Root cause: body scroll region positioned at y=0 while header also at y=0. Fix: offset body by header height via an absolute-positioned body slot (`top=row_height`). Regression: `ecosystem/fret-ui-shadcn/tests/data_grid_layout.rs` `data_grid_header_does_not_overlap_body`. Evidence: `target/fret-diag/1769775649931-ui-gallery-layout-sweep-data-grid/frame.bmp` (after). |
| L6 | P0 | `workspace` | In-window menubar trigger bounds can overlap at non-1.0 DPI, causing top-bar labels (e.g. `Window` / `Gallery`) to visually collide. | `tools/diag-scripts/ui-gallery-menubar-text-overlap-command.json` | Fixed (pending merge) | codex | Root cause: subpixel rounding in a trigger row with `gap=0` can produce a small overlap between adjacent triggers. Fix: add a tiny horizontal gap (`gap=Px(1.0)`) for the trigger row. Regression gate: script asserts `bounds_non_overlapping(menubar-trigger-window, menubar-trigger-gallery, eps_px=0.1)`. Evidence: `target/fret-diag/1769790859739-ui-gallery-menubar-text-overlap-command/frame.bmp` (before) vs `target/fret-diag/1769822475774-ui-gallery-menubar-text-overlap-command/frame.bmp` (after). |
| L7 | P0 | `chrome_torture` | Overlay triggers can render on top of the “One/Two/Three” chrome row (distinct controls end up sharing the same bounds). | `tools/diag-scripts/ui-gallery-chrome-torture-layout.json` | Fixed (pending merge) | codex | Root cause: `preview_chrome_torture` returned multiple siblings directly under a `SemanticsProps` wrapper, so siblings were not vertically stacked and could overlap. Fix: wrap the page body in a `VStack` so overlay content and chrome controls are laid out top-to-bottom. Regression gate: `bounds_non_overlapping(ui-gallery-dropdown-trigger, ui-gallery-chrome-btn-1)` and `bounds_non_overlapping(ui-gallery-context-trigger, ui-gallery-chrome-btn-3)`. Evidence: `target/fret-diag/1769824461382-ui-gallery-layout-sweep-chrome-torture/frame.bmp` (before) vs `target/fret-diag/1769827764966-ui-gallery-layout-sweep-chrome-torture/frame.bmp` (after). |
| L8 | P0 | `toast` | Toast entry can land partially off-window in tight sizes (e.g. `800×600`) due to transition state collisions (multiple transitions in one element share state). | `tools/diag-scripts/ui-gallery-toast-visible.json` | Fixed (pending merge) | codex | Root cause: transition driver state was keyed only by element id + type, so multiple calls shared a single timeline. Fix: key transition driver state by callsite (so each call gets independent state). Gate: `toast-entry-1` within window after clicking `ui-gallery-toast-default`. |
| L9 | P0 | `data_table_torture` | Retained table torture rows were intrinsically sized (no fixed column widths), causing per-row width drift and header/body column misalignment. | (manual) | Fixed (pending merge) | codex | Root cause: `table_virtualized_retained_v0` rendered header/body as simple `HStack` children without width constraints. Fix: compute `resolve_column_width` once per render, wrap header/body cells in fixed-width containers, and share a horizontal `ScrollHandle` between header and body. Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_retained_v0`). |

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
- Add a small-window sweep run (catches overflow earlier):

```powershell
$env:FRET_UI_GALLERY_MAIN_WINDOW_SIZE="800x600"
cargo run -p fretboard -- diag suite ui-gallery-layout --env FRET_DIAG_SCREENSHOT=1 --launch -- cargo run -p fret-ui-gallery --release
```

### 4.1 Latest Size Matrix Runs

- 2026-01-31: `fretboard diag suite ui-gallery-layout` passes at `800x600`, `960x540`, `1024x768`, `1280x720` (with `--timeout-ms 240000` and `FRET_DIAG_SCREENSHOT=1`).
- 2026-01-31: `ui-gallery-layout-sweep-extended.json` passes at `800x600`, `960x540`, `1024x768`, `1280x720` (with `--timeout-ms 240000` and `FRET_DIAG_SCREENSHOT=1`).
- 2026-01-31: `ui-gallery-layout-sweep-extended-chrome.json` passes at `800x600`, `960x540`, `1024x768`, `1280x720` (with `--timeout-ms 240000` and `FRET_DIAG_SCREENSHOT=1`).
- 2026-01-31: `ui-gallery-layout-sweep-torture.json` passes at `800x600`, `960x540` (with `--timeout-ms 240000` and `FRET_DIAG_SCREENSHOT=1`).

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

As of 2026-01-31:

- M0–M3: complete for the current UI Gallery surfaces.
- M4: ongoing (this tracker stays open for future regressions).

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

Implementation note:

- `fret-ui-shadcn` `ScrollArea` now defaults to `min_w_0().min_h_0()` so it is “safe by default” in editor-like flex shells.
- `fret-ui-shadcn` `DialogContent` / `SheetContent` / `PopoverContent` now default to `min_w_0().min_h_0()` and
  use shrink-safe internal stacks to reduce “max-content blowup” and flex+scroll edge cases.

### 5.2.1 Where to Fix: Component Defaults vs Helper Methods

Rule of thumb:

- If the issue is a **generic invariant violation** (A–D) that can affect many callsites, fix it as a **safe-by-default component behavior** (e.g. `TabsContent`, `ScrollArea`, overlay content wrappers).
- If the issue is **policy-specific** (product look/feel, bespoke spacing rules, demo-only composition), prefer a **local helper method** or per-page wrapper to avoid baking policy into the mechanism layer.

Definition of “safe-by-default” here:

- The default should prevent common editor-UI failure modes without surprising explicit caller intent.
- Callers must still be able to override the default (explicit props/layout should win).

### 5.3 TODO (Near-term)

- [x] Add/maintain a “P0-first” issue queue in this doc (table in section 3 stays authoritative).
- [x] Audit shadcn containers for Invariants A–D (keep the checklist explicit):
  - [x] Tabs content / panels (L2)
  - [x] Scroll areas used inside flex stacks (`ScrollArea` defaults to `min_w_0().min_h_0()`)
  - [x] Resizable/split panel wrappers and handle rows (L1)
  - [x] Cards/dialogs/sheets/popovers content wrappers (shrink-safe defaults)
- [x] Add a lightweight “page-sweep” script that visits a few core pages and asserts `bounds_within_window` for `ui-gallery-page-*` roots:
  - `tools/diag-scripts/ui-gallery-layout-sweep-core.json`
  - Current coverage (kept intentionally small): `intro`, `layout`, `scroll_area`, `tabs`, `accordion`, `overlay`, `resizable`.
- [x] Keep P0 regressions in the default layout suite:
  - `tools/diag-scripts/ui-gallery-menubar-text-overlap-command.json`
  - `tools/diag-scripts/ui-gallery-chrome-torture-layout.json`
- [x] Add an “extended sweep” script (not in the default suite) that visits more pages to discover new layout gaps early:
  - Target pages: `data_table`, `tree`, `virtual_list`, `code_view`, `sidebar`, `menus`, `command`, `toast`, `material3_menu` (and any newly added gallery pages).
  - Gate style: a small number of `bounds_within_window` assertions on page roots + 1–3 critical controls per page.
  - Current script: `tools/diag-scripts/ui-gallery-layout-sweep-extended.json` (kept out of `ui-gallery-layout` suite by default).
- [x] Harden `ui-gallery-menubar-text-overlap-command` to cover all adjacent triggers (not just `Window`/`Gallery`).
- [x] Add a dedicated “torture sweep” script (not in the default suite) that visits high-risk torture pages to discover layout gaps in tight windows:
  - Current script: `tools/diag-scripts/ui-gallery-layout-sweep-torture.json` (kept out of `ui-gallery-layout` suite by default).
- [x] Add a dedicated “chrome” sweep that targets non-page UI surfaces (top bar / in-window menubar / toast), since they are not addressable as gallery pages:
  - Current script: `tools/diag-scripts/ui-gallery-layout-sweep-extended-chrome.json` (kept out of the default suite by default).
- [x] Add a toast overlay bounds gate in the chrome sweep:
  - `tools/diag-scripts/ui-gallery-layout-sweep-extended-chrome.json` (clicks `ui-gallery-toast-default` and asserts `toast-entry-1` is within the window).
- [x] Turn capture-only P0 repro scripts into layout gates:
  - `tools/diag-scripts/ui-gallery-intro-preview-width-bundle.json`: assert `bounds_within_window` for `ui-gallery-intro-preview-grid` and `ui-gallery-intro-preview-note`.
  - `tools/diag-scripts/ui-gallery-resizable-initial-bundle.json`: assert `bounds_min_size` for `ui-gallery-resizable-panels` (prevents collapse regressions without requiring it to fit within the window).
- [x] Add an overlay regression script that opens key modals/popovers and asserts bounds:
  - `tools/diag-scripts/ui-gallery-overlay-modals-visible.json`
- [x] Extend the overlay script with a “flex + scroll” stress interaction:
  - Scroll dialog / sheet viewports to catch `min_h_0` regressions.
- [x] Extend the overlay script with “hover + menu” coverage:
  - Tooltip / HoverCard open + bounds assertions
  - DropdownMenu / ContextMenu open + bounds assertions
- [x] Add a scroll + clamp regression for hover overlays:
  - `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json` (opens the hover overlays, then scrolls the main content viewport and asserts the overlay stays within the window).
- [x] Add a submenu bounds regression for dropdown menus:
  - `tools/diag-scripts/ui-gallery-dropdown-submenu-bounds.json` (opens the `More` submenu and asserts the nested item stays within the window).
- [x] Add a bounds regression for context menus:
  - `tools/diag-scripts/ui-gallery-contextmenu-edge-bounds.json` (right-clicks the dedicated edge trigger `ui-gallery-context-trigger-edge` and asserts the first item stays within the window).
- [x] Add a popover scroll + clamp regression:
  - `tools/diag-scripts/ui-gallery-overlay-portal-geometry-clamp.json` (opens a popover, scrolls the main content viewport, and asserts the popover content stays clamped within the window).
- [ ] For each new P0 issue:
  - [ ] Add a `tools/diag-scripts/ui-gallery-...json` repro (navigate + wait_until + bounds assertions + `capture_bundle` + `capture_screenshot`).
  - [ ] Add minimal hot-region `test_id` anchors (prefer `cx.semantics` wrappers; do not overload a11y labels).
  - [ ] Capture before/after evidence bundle dirs and record them under the issue row.
- [ ] When a fix changes a shadcn default, update the corresponding snapshot tests if they are intentional (document the reasoning in the PR/commit message).

### 5.4 TODO (Mid-term)

- [ ] Expand `fretboard diag suite ui-gallery-layout` as P0 issues grow (keep it small; no perf tortures here).
- [ ] Convert script-only regressions into unit tests where feasible (pure layout / semantics assertions).
- [ ] Document “known safe defaults” per container component (one paragraph each) to prevent future drift.
