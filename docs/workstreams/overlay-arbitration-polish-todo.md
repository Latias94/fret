# Overlay Arbitration Polish — TODO Tracker

Status: Active (workstream tracker; keep updated during refactors)

This tracker focuses on the “editor-grade UI feel” gaps around overlays/portals (menus, popovers, dialogs),
their lifecycle, and input arbitration — especially under view-cache reuse.

Primary goal: make overlay behavior deterministic and regression-tested before deeper refactors.

## Contract Gates (Must Drive Implementation)

- Overlays + multi-root: `docs/adr/0011-overlays-and-multi-root.md`
- UI runtime contract surface boundaries: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Runtime contract matrix (placement + semantics expectations): `docs/runtime-contract-matrix.md`
- Focus scopes + traversal: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Overlay dismiss policy: `docs/adr/0087-overlay-policy-architecture-dismissal-focus-portal.md`
- Modal vs non-modal overlays: `docs/adr/0095-menu-open-modality-and-entry-focus.md`

## Tracking Format

Each TODO is labeled:

- ID: `OVERLAY-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Existing Harnesses)

- Scripted UI regressions: `apps/fretboard/src/diag.rs`, `tools/diag-scripts/*`, `docs/ui-diagnostics-and-scripted-tests.md`
- View-cache reuse gating: `fretboard diag run/suite --check-view-cache-reuse-min <n>`
- Bundle comparison: `fretboard diag compare <a> <b> ...` (semantics + optional scene fingerprint)

## P0 — Lifecycle + Arbitration Correctness

- [ ] OVERLAY-life-001 Define and document overlay lifecycle phases.
  - Target: a small state machine (requested → mounted → interactive → dismissing → unmounted) with explicit ownership.
  - Evidence target: `crates/fret-ui/src/overlay_placement/*`, `ecosystem/fret-ui-kit/src/window_overlays/*`
- [ ] OVERLAY-in-002 Specify input arbitration ordering across roots.
  - Target: consistent rules for: modal barrier, underlay click-through, escape routing, focus restore, pointer capture.
  - Evidence target: `crates/fret-ui/src/tree/dispatch.rs`, `crates/fret-ui/src/tree/tests/*`
- [ ] OVERLAY-in-003 Specify “outside press” semantics for stacked overlays (menu → submenu → popover).
  - Target: deterministic outside press observers + dismissal propagation rules.
  - Evidence target: `crates/fret-ui/src/tree/tests/outside_press.rs`

## P0 — Regression Scenarios (Executable)

- [x] OVERLAY-reg-010 Add a “menu stack” scripted scenario (submenu hover + outside press + focus handoff).
  - Touches: `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json`, `apps/fretboard/src/diag.rs`
  - Notes: validates submenu opens on hover and that an outside press dismisses overlays and focuses the underlying target.
- [ ] OVERLAY-reg-011 Add a “nested popover + dialog” scripted scenario (focus trap + escape + underlay).
  - Touches: `tools/diag-scripts/*`, `apps/fretboard/src/diag.rs`
- [ ] OVERLAY-reg-012 Add a “portal geometry” scenario (floating placement + viewport clamp + scroll/resize).
  - Touches: `crates/fret-ui/src/overlay_placement/*`, `tools/diag-scripts/*`
- [ ] OVERLAY-reg-013 Add a cache-hit bundle comparison baseline for overlay scenarios.
  - Mechanism: record cached+uncached bundles and enforce `diag compare` + `--check-view-cache-reuse-min`.

## P1 — Ergonomics (Ecosystem Integration)

- [ ] OVERLAY-eco-020 Provide policy-heavy primitives in ecosystem with a strict core boundary.
  - Target: keep “dismissal policies / hover intent / focus restore / modality decisions” in `ecosystem/*`.
  - Evidence target: `ecosystem/fret-ui-kit/src/primitives/*`, `ecosystem/fret-ui-shadcn/src/*`
