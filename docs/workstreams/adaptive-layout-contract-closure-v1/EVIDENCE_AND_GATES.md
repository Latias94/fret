# Adaptive Layout Contract Closure v1 — Evidence and Gates

Status: Active
Last updated: 2026-04-10

## Smallest current repro

Use this sequence before widening the adaptive audit:

```bash
cargo nextest run -p fret-ui-gallery --test popup_menu_narrow_surface
cargo build -p fret-ui-gallery --release
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json --dir target/fret-diag-popup-menu-narrow-sweep --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
```

What this proves now:

- current popup/menu Gallery surfaces stay inside a narrow window after the latest gallery-shell
  fixes,
- the Dialog demo trigger/content path also stays within a 420px-wide window after moving width
  ownership onto the preview-root shell,
- the lane starts from user-visible narrow-window evidence rather than abstract breakpoint theory,
- and future adaptive refactors have one immediate UI Gallery regression anchor.

## Current evidence set

- `docs/workstreams/container-queries-v1/container-queries-v1.md`
  - keeps the container-query mechanism and initial recipe-migration baseline visible.
- `docs/workstreams/environment-queries-v1/environment-queries-v1.md`
  - keeps the environment/device-query mechanism and capability-gating baseline visible.
- `docs/workstreams/adaptive-layout-contract-closure-v1/BASELINE_AUDIT_2026-04-10.md`
  - freezes the framework-level capability inventory, drift ranking, and first fearless-refactor
    order for this lane.
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
  - freezes the adaptive authoring taxonomy, layer ownership, and naming rules for future cleanup.
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
  - records the intended public-surface split, naming rules, and the current rename queue.
- `docs/workstreams/adaptive-layout-contract-closure-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
  - records the lane-local contract freeze and what it unblocks next.
- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
  - pins the owner split for editor rails / inspector sidebars so app-shell `Sidebar` does not
    become the accidental center of panel-adaptive work.
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
  - resolves the next-shell seam decision by pinning `WorkspaceFrame.left/right` as the existing
    outer rail seam while keeping rail recipes app-local for now.
- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_PANEL_RESIZE_GATE_PROMOTION_2026-04-10.md`
  - records the passing promotion run, the packed share artifact, and the script-compatibility note
    for the fixed-window panel-resize proof.
- `docs/known-issues.md`
  - already states that remaining viewport breakpoints should mean device-level behavior, not a
    substitute for container queries.
- `docs/crate-usage-guide.md`
  - already keeps adaptive helpers explicit on `fret::env::{...}`.
- `ecosystem/fret/src/lib.rs`
  - shows the current public adaptive export surface, including the explicit `fret::adaptive`
    facade lane.
- `ecosystem/fret-ui-kit/src/adaptive.rs`
  - shows the current shared adaptive policy vocabulary (`AdaptiveQuerySource`,
    `DeviceAdaptiveClass`, `PanelAdaptiveClass`) and the first classification helpers above raw
    query reads.
- `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
  - shows the current container-query helper budget and breakpoint tokens.
- `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
  - shows the current viewport/environment helper budget and breakpoint tokens.
- `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/demo.rs`
  - already contains an explicit container-vs-viewport comparison surface.
- `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
  - already documents that breakpoint choices remain caller-owned on that page.
- `apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs`
  - current narrow-window Gallery regression test.
- `apps/fret-ui-gallery/tests/dialog_docs_surface.rs`
  - keeps the Dialog demo width-lane and narrow-sweep script wired into the docs-alignment gate.
- `apps/fret-ui-gallery/tests/combobox_docs_surface.rs`
  - keeps whitespace-stable caller-owned width assertions and the explicit device-shell naming on
    the combobox docs-path proof surfaces.
- `apps/fret-ui-gallery/tests/field_docs_surface.rs`
  - keeps the public docs lane explicit about `FieldOrientation::ContainerAdaptive` while the page
    still mirrors the upstream `Responsive Layout` section naming.
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
  - keeps the sidebar page explicit that `is_mobile(...)` / `is_mobile_breakpoint(...)` are
    app-shell/device-shell controls rather than generic panel-adaptive helpers.
- `ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs`
  - keeps the viewport/device-shell branch on the responsive combobox follow-up under a focused
    real-runtime gate.
- `ecosystem/fret-ui-shadcn/tests/field_responsive_orientation.rs`
  - keeps the container-query field orientation proof explicit after renaming the public enum to
    `FieldOrientation::ContainerAdaptive`.
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json`
  - current narrow-window screenshot/bundle proof surface.
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-demo-narrow-sweep.json`
  - explicit narrow-window proof for the Dialog docs demo trigger and content bounds.
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-overlay-narrow-header-sweep.json`
  - sampled overlay-family proof that Popover / Sheet / Drawer / Alert Dialog still fit in a
  narrow window after the current width-hygiene slice.
- `tools/diag-scripts/container-queries-docking-panel-resize.json`
  - stable redirect path for the promoted panel-resize gate.
- `tools/diag-scripts/docking/container-queries/container-queries-docking-panel-resize.json`
  - current v2 script payload for the promoted panel-resize gate, now with layout-sidecar capture.

## Active gate set

### UI Gallery narrow-window proof

```bash
cargo nextest run -p fret-ui-gallery --test popup_menu_narrow_surface --test combobox_docs_surface --test dialog_docs_surface --no-fail-fast
cargo nextest run -p fret-ui-gallery --test field_docs_surface --no-fail-fast
cargo nextest run -p fret-ui-gallery --test sidebar_docs_surface --no-fail-fast
cargo nextest run -p fret-ui-shadcn --test combobox_responsive_breakpoint --test field_responsive_orientation --no-fail-fast
```

### UI Gallery narrow-window screenshot sweep

```bash
cargo build -p fret-ui-gallery --release
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json --dir target/fret-diag-popup-menu-narrow-sweep --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-demo-narrow-sweep.json --dir target/fret-diag-dialog-demo-narrow-sweep --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-overlay-narrow-header-sweep.json --dir target/fret-diag-overlay-narrow-header-sweep --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
```

### Fixed-window panel-resize proof

```bash
cargo build -p fret-demo --bin container_queries_docking_demo --release
cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/adaptive-panel-resize-promote --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo
```

Current promoted run:

- session dir:
  `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694`
- packed artifact:
  `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694/share/1775822919993.zip`

### Diff hygiene

```bash
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

## Remaining proof gap

The next M2 gap is no longer panel-resize promotion.

What still needs to land is:

- one explicit Gallery teaching surface that compares container-driven and viewport-driven adaptive
  behavior without blending the two into one ambiguous "responsive" story.
