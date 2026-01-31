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
- [ ] SWG-ovl-020 Add destructive state matrix gates where upstream uses distinct idle vs focused chrome.
  - Target families: DropdownMenu / ContextMenu / Menubar / NavigationMenu.
- [ ] SWG-ovl-030 Add “constrained height” variants for remaining overlay pages that currently only gate default viewport.

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

- [ ] SWG-cal-010 Add selection/hover/disabled state chrome gates on month grids across a constrained viewport.
- [ ] SWG-cal-020 Add nested overlay “stacking order + clamp + scroll” gates (Select inside DatePicker popover).
- 2026-01-31: added `date-picker-with-presets.preset-tomorrow-vp375x240` open golden + placement gate (Select interaction + deterministic date selection).

---

## P2 — Tooling (avoid doc drift)

- [ ] SWG-tool-010 Extend `tools/golden_coverage.ps1` to report coverage in explicit dimensions:
  - smoke-parse coverage (dynamic traversal),
  - referenced-by-tests coverage (string-literal heuristic),
  - high-signal targeted gates (excluding broad files).

---

## P3 — DPI / Resolution (keep tiny)

- [ ] SWG-dpi-010 Prototype a 2x2 matrix (2 viewports x 2 scales) for typography + menus/listboxes + calendar.
  - Blocked on: stable baseline geometry in P0/P1.
