# Shadcn Web Goldens v4 (new-york-v4) — Workstream Plan

Status: Active (workstream note; not a contract)

This workstream drives strict **1:1 parity** between:

- Upstream shadcn/ui v4 registry pages: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx`
- Fret shadcn recipes: `ecosystem/fret-ui-shadcn/src/*.rs`

The approach is **goldens-driven**:

- Use shadcn-web JSON goldens as the baseline truth for layout/style outcomes.
- Convert breadth “this key is referenced somewhere” into depth “this behavior is hard-gated”.
- Treat viewport-dependent outcomes (especially overlays) as *first-class styling outcomes*.

Related canonical trackers / audits:

- Canonical shadcn progress tracker: `docs/shadcn-declarative-progress.md`
- Coverage snapshot (what is referenced by tests): `docs/audits/shadcn-new-york-v4-coverage.md`
- Alignment notes (what differs and why): `docs/audits/shadcn-new-york-v4-alignment.md`
- Depth backlog (high-signal next targets): `docs/audits/shadcn-new-york-v4-todo.md`
- Golden extraction: `docs/shadcn-web-goldens.md`

---

## Guiding Principles

1. **Determinism first**
   - If a golden depends on time (`new Date()`), fonts, or async rendering, freeze inputs (e.g. extractor `--freezeDate`) and wait for stable geometry.
2. **Depth by family, not by file count**
   - Prioritize families where small refactors tend to drift outcomes: overlays, typography, calendar/date, charts.
3. **Semantic ownership is a boundary**
   - Do not “fix” layout by globally changing `Text` defaults (e.g. making all text `Fill`).
   - Block-vs-inline must be decided at the recipe/component level (align to the upstream element semantics).
4. **Treat overlay height as style**
   - Max-height clamping, scroll buttons, and listbox/menu row height are parity-critical and should be gated explicitly.

---

## Milestones (v4/new-york-v4)

### M0 — Breadth coverage + smoke parse (done)

Definition of done:

- All tracked golden keys exist and are referenced by tests.
- A smoke pass parses all goldens and validates finite rects.

Evidence:

- `docs/audits/shadcn-new-york-v4-coverage.md`
- `pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly`

### M1 — Deterministic goldens (in progress)

Goal: eliminate non-determinism sources that block reviewability and stable gating.

Examples:

- Freeze time for DatePicker preset pages (extractor `--freezeDate`).
- Use production builds (`next build --webpack`) and stable viewport sizes.

### M2 — Overlay depth gates (active)

Goal: for each overlay family, gate:

- panel chrome (radius/border/shadow/padding),
- placement + collision behavior under constrained viewports,
- panel size (width/height) and scroll behavior,
- row height as a styling outcome (menus/listboxes).

### M3 — Typography depth gates (active)

Goal: move from “single viewport smoke” to parity across:

- multiple widths (wrap/balance/list markers/margins),
- a tiny DPI matrix (later; see M5),
- sensitive inline metrics (inline code padding, letter spacing, list marker offsets).

### M4 — Calendar/Date depth gates (active)

Goal: parity for:

- month grid geometry + outside days + week numbers,
- selected/hovered/disabled state chrome,
- nested overlay interactions (e.g. Select inside DatePicker popover),
- deterministic day labels and formatting.

### M5 — Minimal DPI/font-metrics matrix (blocked on M2–M4 stability)

Do not expand this dimension until baseline geometry is stable.

Suggested minimal matrix:

- 2 viewports: default + constrained height
- 2 scale factors: 1.0 + 1.25 (or 1.5)
- 3 families: typography + menus/listboxes + calendar

---

## What This Workstream Does Not Cover

- Runtime contracts in `crates/fret-ui` (those must be ADR-backed).
- General “UI diagnostics + scripted tests” (see `docs/ui-diagnostics-and-scripted-tests.md`).

