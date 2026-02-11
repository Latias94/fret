# Overlay Arbitration Polish — TODO Tracker

Status: Complete (v1; keep this tracker updated if new overlay/arbitration gaps are discovered)

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
- Modal vs non-modal overlays: `docs/adr/0094-menu-open-modality-and-entry-focus.md`

## Tracking Format

Each TODO is labeled:

- ID: `OVERLAY-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Existing Harnesses)

- Scripted UI regressions: `crates/fret-diag/src/lib.rs`, `tools/diag-scripts/*`, `docs/ui-diagnostics-and-scripted-tests.md`
- Default suite entrypoint: `cargo run -p fretboard -- diag suite ui-gallery` (script list lives in `crates/fret-diag/src/lib.rs`)
- View-cache reuse gating: `--check-view-cache-reuse-min <n>`
- Bundle comparison: `fretboard diag compare <a> <b> ...` (semantics + optional scene fingerprint)
- Matrix runner: `fretboard diag matrix ui-gallery` (runs cached+uncached variants and compares per-script bundles)

## P0 — Lifecycle + Arbitration Correctness

- [x] OVERLAY-life-001 Define and document overlay lifecycle phases.
  - Output: `docs/workstreams/overlay-lifecycle-phases.md`
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `ecosystem/fret-ui-kit/src/window_overlays/state.rs`, `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
  - Notes: baseline lifecycle phases were documented before hover/tooltip gained authoritative presence; see OVERLAY-life-002 for the updated contract.
- [x] OVERLAY-life-002 Introduce authoritative presence for hover/tooltip overlays.
  - Target: make hover/tooltip safe under view-cache reuse without creating “ghost overlays” (authoritative `open/present` + liveness gate).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `ecosystem/fret-ui-kit/src/window_overlays/state.rs`
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/state.rs` (`OVERLAY_CACHE_TTL_FRAMES`, `last_seen_frame`)
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (hover/tooltip synthesis + closing interactivity)
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`, `ecosystem/fret-ui-kit/src/primitives/hover_card.rs` (API surface)
  - Evidence: `ecosystem/fret-ui-shadcn/src/tooltip.rs`, `ecosystem/fret-ui-shadcn/src/hover_card.rs` (integration)
  - Checklist:
    - [x] Add `open: Model<bool>` + `present: bool` to `HoverOverlayRequest`/`TooltipRequest`
    - [x] Synthesize hover/tooltip requests from cached declarations under view-cache reuse
    - [x] Add a short TTL liveness gate to prevent ghost incidental overlays under producer suppression
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
  - Evidence: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`render_entries` in submenu panel propagates `DropdownMenuItem.test_id`,
    `dropdown_menu_submenu_items_propagate_test_ids`), `apps/fret-ui-gallery/src/ui.rs` (gallery dropdown uses `modal(false)` to validate
    click-through outside press).
- [x] OVERLAY-reg-011 Add a “nested popover + dialog” scripted scenario (focus trap + escape + underlay).
  - Touches: `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json`, `apps/fretboard/src/diag.rs`
  - Notes: opens a dialog from inside a popover, asserts modal barrier blocks underlay, then verifies escape focus restore to the popover trigger.
- [x] OVERLAY-reg-012 Add a “portal geometry” scenario (floating placement + viewport clamp + scroll/resize).
  - Touches: `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-portal-geometry-scroll-clamp.json`
  - Assertion: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`bounds_within_window` predicate)
  - Notes: opens a popover inside a scroll viewport and asserts it stays within the window before/after wheel scroll.
- [x] OVERLAY-reg-013 Add a cache-hit bundle comparison baseline for overlay scenarios.
  - Mechanism: record cached+uncached bundles and enforce a per-script compare (semantics + optional scene fingerprint).
  - Evidence:
    - `fretboard diag matrix ui-gallery` runs cached+uncached variants and compares each script via `diag compare`:
      `apps/fretboard/src/cli.rs`, `apps/fretboard/src/diag.rs`, `docs/ui-diagnostics-and-scripted-tests.md`.

## P0 — Diagnostics (Synthesis Observability)

- [x] OVERLAY-diag-014 Export cached overlay synthesis events to diagnostic bundles.
  - Target: make it easy to assert (and debug) whether cached request synthesis happened under view-cache reuse.
  - Output: `bundle.json` includes `debug.overlay_synthesis` events for each window/frame.
  - Evidence:
    - Bundles export `debug.overlay_synthesis`: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
    - `fretboard` gates on synthesized outcomes via `--check-overlay-synthesis-min`: `apps/fretboard/src/diag.rs`, `docs/ui-diagnostics-and-scripted-tests.md`.
  - Done when: scripts can gate on “at least N synthesized overlays” and report suppression reasons when synthesis does not occur.

## P1 — Ergonomics (Ecosystem Integration)

- [x] OVERLAY-eco-020 Provide policy-heavy primitives in ecosystem with a strict core boundary.
  - Target: keep “dismissal policies / hover intent / focus restore / modality decisions” in `ecosystem/*`.
  - Evidence:
    - Tooltip trigger intent + suppressions moved into `fret-ui-kit`:
      `ecosystem/fret-ui-kit/src/primitives/tooltip.rs` (`tooltip_trigger_update_gates`, `tooltip_install_default_trigger_dismiss_handlers`, `tooltip_wrap_trigger_with_pointer_move_open_gate`)
      and wired by `ecosystem/fret-ui-shadcn/src/tooltip.rs` (no per-recipe driver structs).
    - HoverCard intent driver moved into `fret-ui-kit`:
      `ecosystem/fret-ui-kit/src/primitives/hover_card.rs` (`hover_card_update_interaction`)
      and wired by `ecosystem/fret-ui-shadcn/src/hover_card.rs`.
