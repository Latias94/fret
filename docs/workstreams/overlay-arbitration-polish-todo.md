# Overlay Arbitration Polish — TODO Tracker

Status: Active (workstream tracker; keep updated during refactors)

This tracker focuses on the “editor-grade UI feel” gaps around overlays/portals (menus, popovers, dialogs),
their lifecycle, and input arbitration — especially under view-cache reuse.

Primary goal: make overlay behavior deterministic and regression-tested before deeper refactors.

## Contract Gates (Must Drive Implementation)

- Overlays + multi-root: `docs/adr/0011-overlays-and-multi-root.md`
- Overlay placement contract: `docs/adr/0064-overlay-placement-contract.md`
- UI runtime contract surface boundaries: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Outside press contract: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Runtime contract matrix (placement + semantics expectations): `docs/runtime-contract-matrix.md`
- Focus scopes + traversal: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Overlay dismiss policy: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Modal vs non-modal overlays: `docs/adr/0095-menu-open-modality-and-entry-focus.md`

## Tracking Format

Each TODO is labeled:

- ID: `OVERLAY-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Existing Harnesses)

- Scripted UI regressions: `apps/fretboard/src/diag.rs`, `tools/diag-scripts/*`, `docs/ui-diagnostics-and-scripted-tests.md`
- Default suite entrypoint: `cargo run -p fretboard -- diag suite ui-gallery` (script list lives in `apps/fretboard/src/diag.rs`)
- View-cache reuse gating: `fretboard diag run/suite --check-view-cache-reuse-min <n>`
- Bundle comparison: `fretboard diag compare <a> <b> ...` (semantics + optional scene fingerprint)

## P0 — Lifecycle + Arbitration Correctness

- [x] OVERLAY-life-001 Define and document overlay lifecycle phases.
  - Output: `docs/workstreams/overlay-lifecycle-phases.md`
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `ecosystem/fret-ui-kit/src/window_overlays/state.rs`, `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
  - Notes: documents current authoritative `open/present` (modal + popover) and explicitly calls out hover/tooltip as per-frame-only (tracked separately).
- [x] OVERLAY-life-002 Introduce authoritative presence for hover/tooltip overlays.
  - Target: make hover/tooltip safe under view-cache reuse without creating “ghost overlays” (authoritative `open/present` + liveness gate).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `ecosystem/fret-ui-kit/src/window_overlays/state.rs`
  - Evidence: `crates/fret-ui/src/elements/queries.rs` (`element_is_live_in_current_frame`)
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (hover/tooltip synthesis + closing interactivity)
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`, `ecosystem/fret-ui-kit/src/primitives/hover_card.rs` (API surface)
  - Evidence: `ecosystem/fret-ui-shadcn/src/tooltip.rs`, `ecosystem/fret-ui-shadcn/src/hover_card.rs` (integration)
  - Checklist:
    - [x] Add `open: Model<bool>` + `present: bool` to `HoverOverlayRequest`/`TooltipRequest`
    - [x] Synthesize hover/tooltip requests from cached declarations under view-cache reuse
    - [x] Add liveness gate to prevent ghost overlays (trigger must be live in current frame)
    - [x] Update ecosystem primitives + shadcn recipes to provide `open` + `present`
    - [x] Add regressions for cache-hit synthesis + close-transition interactivity
  - Notes: landing this surfaced a Radix parity dependency around menu initial focus targets:
    - `DropdownMenu` pointer-open focuses the `role=menu` content (state regression: `radix_web_dropdown_menu_open_navigate_select_matches_fret`).
    - `Menubar` pointer-open keeps focus on the roving container so ArrowDown navigation can reach submenu triggers (state regression: `radix_web_menubar_submenu_keyboard_open_close_matches_fret`).
- [x] OVERLAY-in-002 Specify input arbitration ordering across roots.
  - Target: consistent rules for: modal barrier, underlay click-through, escape routing, focus restore, pointer capture.
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`active_input_layers`, `topmost_pointer_occlusion_layer`, `enforce_modal_barrier_scope`)
  - Evidence: `crates/fret-ui/src/tree/tests/window_input_arbitration_snapshot.rs`
- [x] OVERLAY-in-003 Specify “outside press” semantics for stacked overlays (menu → submenu → popover).
  - Target: deterministic outside press observers + dismissal propagation rules.
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`dispatch_pointer_down_outside`)
  - Evidence: `crates/fret-ui/src/tree/tests/outside_press.rs`

## P0 — Regression Scenarios (Executable)

- [x] OVERLAY-reg-010 Add a “menu stack” scripted scenario (submenu hover + outside press + focus handoff).
  - Touches: `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json`, `apps/fretboard/src/diag.rs`
  - Notes: validates submenu opens on hover and that an outside press dismisses overlays and focuses the underlying target.
- [x] OVERLAY-reg-011 Add a “nested popover + dialog” scripted scenario (focus trap + escape + underlay).
  - Touches: `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json`, `apps/fretboard/src/diag.rs`
  - Notes: opens a dialog from inside a popover, asserts modal barrier blocks underlay, then verifies escape focus restore to the popover trigger.
- [x] OVERLAY-reg-012 Add a “portal geometry” scenario (floating placement + viewport clamp + scroll/resize).
  - Touches: `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-portal-geometry-scroll-clamp.json`
  - Assertion: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`bounds_within_window` predicate)
  - Notes: opens a popover inside a scroll viewport and asserts it stays within the window before/after wheel scroll.
- [x] OVERLAY-reg-013 Add a cache-hit bundle comparison baseline for overlay scenarios.
  - Mechanism: record cached+uncached bundles and enforce `diag compare` + `--check-view-cache-reuse-min`.
  - Evidence: `fretboard diag matrix ui-gallery` (runs both variants and compares per-script bundles).

## P1 — Ergonomics (Ecosystem Integration)

- [ ] OVERLAY-eco-020 Provide policy-heavy primitives in ecosystem with a strict core boundary.
  - Target: keep “dismissal policies / hover intent / focus restore / modality decisions” in `ecosystem/*`.
  - Evidence target: `ecosystem/fret-ui-kit/src/primitives/*`, `ecosystem/fret-ui-shadcn/src/*`
