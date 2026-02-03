# Shadcn Web Goldens v4 (new-york-v4) — TODO Tracker

Status: Active (workstream tracker; keep updated as gates land)

This document tracks executable TODOs for the shadcn-web golden parity workstream.

- Narrative plan: `docs/workstreams/shadcn-web-goldens-v4.md`
- Canonical tracker: `docs/shadcn-declarative-progress.md`
- Coverage snapshot: `docs/audits/shadcn-new-york-v4-coverage.md`
- Alignment notes: `docs/audits/shadcn-new-york-v4-alignment.md`
- Depth backlog: `docs/audits/shadcn-new-york-v4-todo.md`

Tracking format:

- ID: `SWG-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## Baseline (keep honest)

- [x] SWG-base-001 Tracked-only coverage is 100% gated/targeted/smoke-parse for `v4/new-york-v4`.
  - Evidence: `pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly`
- [x] SWG-base-002 Deterministic date-dependent goldens supported via extractor `--freezeDate`.
  - Evidence: `goldens/shadcn-web/scripts/extract-golden.mts`

---

## P0 — Overlays (depth first)

- [ ] SWG-ovl-010 Expand menu/listbox “height as styling” gates to any remaining overlay families not yet covered.
  - Target families: Popover-like list surfaces, nested listboxes, anything that clamps under low height.
- 2026-02-01: gated NavigationMenu underlay scroll anchor stability and fixed paint-cache replay to keep last-frame visual bounds in sync (prevents scroll-induced anchor drift).
- 2026-02-02: added `context-menu-demo.vp375x240` + `menubar-demo.vp375x240` panel-size gates (light/dark) to treat constrained viewport menu height as a styling outcome.
- 2026-02-02: added `context-menu-demo.submenu-kbd-vp375x240` + `menubar-demo.submenu-kbd-vp375x240` submenu panel-size + surface-color + shadow-insets gates (light/dark) to lock in constrained viewport clamping behavior for nested menus.
- 2026-02-02: added `context-menu-demo.submenu-highlight-first-vp375x240` + `menubar-demo.submenu-highlight-first-vp375x240` highlighted-item chrome gates (background + text color) for nested menus.
- 2026-02-02: added `context-menu-demo.vp375x240-scrolled-80.open` + a wheel-scroll parity gate (overlay x/y stable + first-visible label matches web) to catch scroll-induced anchor drift.
- 2026-02-02: added `dropdown-menu-demo.vp375x240-scrolled-80.open` + a wheel-scroll parity gate (overlay x/y stable + first-visible label matches web) to catch scroll-induced anchor drift.
- 2026-02-02: added a Menubar wheel anchor-stability gate on `menubar-demo.vp375x240.open` (no scroll range; wheel must not move overlay).
- 2026-02-02: added a NavigationMenu wheel "no-op" gate (wheel over a non-scrollable portal surface must not jitter trigger/content anchor) in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`.
- 2026-02-03: added a Select wheel gate: wheeling outside the listbox must not scroll the underlay (modal barrier), and wheeling inside the listbox must scroll options without drifting the anchored panel (`ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`).
- 2026-02-03: fixed the web golden extractor to support hover-only scripted steps (`hoverNoWait=...`) so hover highlight variants don't deadlock waiting for new portal surfaces; regenerated `*.highlight-first-vp375x240.open.json` for `dropdown-menu-demo`, `context-menu-demo`, and `menubar-demo`.
- [x] SWG-ovl-020 Add destructive state matrix gates where upstream uses distinct idle vs focused chrome.
  - Target families: DropdownMenu / ContextMenu / Menubar / NavigationMenu.
- 2026-02-03: added `button-group-demo.destructive-focus` open golden + idle/focused destructive item chrome gates (light/dark) to lock in `bg-destructive/10` vs idle behavior for DropdownMenu.
- 2026-02-03: added `menubar-demo.destructive-idle` + `menubar-demo.destructive-focus-first` open goldens and matched Menubar destructive idle/focused chrome (light/dark). (NavigationMenu has no destructive variant in upstream v4.)
- [~] SWG-ovl-030 Add “constrained height” variants for remaining overlay pages that currently only gate default viewport.
  - 2026-02-03: added `vp375x240` open goldens + gates for `hover-card-demo`, `combobox-dropdown-menu`, `command-dialog`, and `select-scrollable` (treat mobile constrained viewports as first-class overlay behavior).
  - 2026-02-03: added `vp375x240` open goldens + placement/insets gates for modal overlays: `dialog-demo`, `sheet-demo`, `alert-dialog-demo`, `drawer-demo`, and `drawer-dialog`.
  - 2026-02-03: added `vp375x240` open goldens + menu height/item chrome gates for `dropdown-menu-checkboxes`, `dropdown-menu-radio-group`, and `dropdown-menu-dialog`.
  - 2026-02-03: added `vp375x240` open goldens + menu/listbox height gates for `item-dropdown`, `breadcrumb-dropdown`, `combobox-popover`, and `combobox-responsive`.
  - 2026-02-03: added `vp375x240` open goldens + gates for `mode-toggle` and `sheet-side` (top/right/bottom/left), treating Sheet side widths/edge insets as a conformance outcome under constrained mobile height.
  - 2026-02-03: added `vp375x240` open goldens + placement/menu-height gates for `button-group-demo` (DropdownMenu in a tight horizontal control cluster).

---

## P1 — Typography (multi-width first)

- [~] SWG-typo-010 Add at least two width variants for `typography-*` pages and gate wrap/margins/list markers.
  - Suggested widths: ~375 (mobile) + ~768 (tablet) or a tight fixed content width.
- 2026-01-31: added `typography-demo.vp375x900` + a wrap/max-width contract gate in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_typography.rs`.
- 2026-01-31: added `typography-demo.vp768x900` + a wrap/max-width contract gate in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_typography.rs`.
- 2026-01-31: added a `typography-list` indent + gap contract gate (derived from the web golden’s `marginLeft` + `marginTop` rules).
- [ ] SWG-typo-020 Add a minimal font-metrics drift gate once wrap behavior is stable (defer DPI matrix to P3).

---

## P1 — Calendar/Date (depth)

- [~] SWG-cal-010 Add selection/hover/disabled state chrome gates on month grids across a constrained viewport.
- [x] SWG-cal-020 Add nested overlay “stacking order + clamp + scroll” gates (Select inside DatePicker popover).
  - Evidence: `date-picker-with-presets.select-open-vp375x160.open` + `date-picker-with-presets.select-open-vp375x160-scrolled-80.open` + placement + listbox panel-size gate + paint-order gate + scroll parity gate.
- 2026-02-01: fixed Radix popper “size()” available-height metrics to apply collision padding/boundary when computing `--radix-*-content-available-height` equivalents (unblocks strict max-height parity for `SelectPosition::Popper` under constrained viewports).
- 2026-02-01: hardened the overlay-chrome panel-size matcher to prefer semantics-bounded chrome quads (then fallback by size) to avoid nested-overlay ambiguity (Popover + ListBox in the same scene).
- 2026-01-31: added `date-picker-with-presets.preset-tomorrow-vp375x240` open golden + placement gate (Select interaction + deterministic date selection).
- 2026-02-01: added a selected-day background gate (`calendar-14`) and fixed calendar chrome margins so row gaps don't inflate the selected background quad.
- 2026-02-01: added a range-middle background gate (`calendar-04`) using the web golden’s computed `backgroundColor` and a scene quad matcher that prefers opaque backgrounds.
- 2026-02-01: added range-start/range-end background gates (`calendar-04`) and a semantics gate for disabled navigation buttons (`calendar-11`).
- 2026-02-01: added a disabled-day semantics gate (`calendar-08`, `rdp-disabled` / `data-disabled=true`).
- 2026-02-01: fixed calendar day number alignment (centered text within day cells) and added a selected-day text-centering gate (`calendar-14`).
- 2026-02-01: added a hover-day background gate using deterministic hover goldens (`calendar-14.hover-day-13`).
- 2026-02-01: added a focus-visible ring gate using deterministic keyboard-focus goldens (`calendar-14.focus-kbd-selected`).
- 2026-02-01: added constrained-viewport (vp375x320) gates for Calendar selection/hover/focus (`calendar-14.vp375x320`, `calendar-14.hover-day-13-vp375x320`, `calendar-14.focus-kbd-selected-vp375x320`).
- 2026-02-02: added purpose-built `web_vs_fret_calendar.rs` chrome gates for `calendar-14*` (selected/hover backgrounds + day-number centering).
  - TODO: remove the legacy duplicates that still live in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` once the new gates have baked for a bit.
- 2026-02-02: added popover panel-size gates for `calendar-22.vp375x240` and `calendar-23.vp375x240` (light/dark) to lock in constrained-height calendar popover behavior.
- 2026-02-01: extracted constrained-viewport calendar variants (`calendar-04/08/11.vp375x320`) and added geometry/semantics gates; added range background paint gates for `calendar-04.vp375x320`.
- 2026-02-01: aligned disabled day styling with upstream (`opacity-50` on day label) and added paint-level gates for disabled day text opacity + disabled navigation “icon” color (fallback to text when SVG ops are not emitted by the test harness services).
- 2026-02-01: added an outside-day disabled text opacity gate for `calendar-11` (and `calendar-11.vp375x320`) using web `computedStyle.color` + `computedStyle.opacity` as the contract.

---

## P2 — Tooling (avoid doc drift)

- [x] SWG-tool-010 Extend `tools/golden_coverage.ps1` to report coverage in explicit dimensions:
  - smoke-parse coverage (dynamic traversal),
  - referenced-by-tests coverage (string-literal heuristic),
  - high-signal targeted gates (excluding broad files).
- 2026-02-01: `tools/golden_coverage.ps1` reports `Gated`, `Targeted`, and `Smoke` coverage for `shadcn-web/v4/new-york-v4`.

---

## P3 — DPI / Resolution (keep tiny)

- [ ] SWG-dpi-010 Prototype a 2x2 matrix (2 viewports x 2 scales) for typography + menus/listboxes + calendar.
  - Blocked on: stable baseline geometry in P0/P1.
