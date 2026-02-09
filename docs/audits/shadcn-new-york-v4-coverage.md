# shadcn/ui new-york-v4 Coverage (Fret)

This document tracks **coverage**, not deep conformance details:

- Which shadcn-web goldens exist for `new-york-v4`
- Which golden keys are **gated** by `ecosystem/fret-ui-shadcn/tests`
- Where to focus next to increase “1:1” confidence efficiently

For deep, high-impact alignment notes and “gaps to check” per component, see:

- `docs/audits/shadcn-new-york-v4-alignment.md`
- Chart-specific audit: `docs/audits/shadcn-chart.md`

## Why coverage matters (and why viewport variants are not “nice-to-have”)

If the goal is 1:1 parity with upstream shadcn/ui + Radix primitives, coverage must include:

- **Breadth**: every component gets at least one golden + one gate (default outcome)
- **Depth for behavior-shaping viewports**: overlays and responsive components must be gated under
  constrained viewports early (clamp/shift/flip/max-height/scroll buttons/viewport sizing)

Practical rule:

- First pass: add a “default golden + minimal gate” for each component.
- In parallel: for overlays (menus, popovers, dialogs, navigation menu, etc.), add at least one
  constrained viewport variant as soon as the default is gated.

## Current snapshot (shadcn-web, v4/new-york-v4)

This is a **snapshot** from running `tools/golden_coverage.py` in this repo.

Notes:

- `tools/golden_coverage.py` currently counts a key as “gated” if the key appears as a
  **string literal** somewhere under `ecosystem/fret-ui-shadcn/tests`, excluding `*_goldens_smoke.rs`.
- This is a reasonable proxy for “this golden is exercised by some dedicated test”, but it is not
  a perfect proxy for “high-signal behavior is gated”.
- `shadcn_web_goldens_smoke.rs` provides a separate, *dynamic* “smoke-parse” pass that ensures we
  can parse the JSON and that the exported rectangles are finite.
- `tools/golden_coverage.py` reports “smoke-parse coverage” only when it can infer that the smoke
  test targets the requested `--style` (otherwise it prints `n/a` to avoid false confidence).
- To avoid local, uncommitted goldens skewing the counts, prefer `--tracked-only`.
- In addition to the “any gate” percentage, `tools/golden_coverage.py` reports a **targeted**
  percentage that excludes the broad “catch-all” layout file(s) (`web_vs_fret_layout.rs` and
  `snapshots.rs` by default). This helps answer “how much is covered by high-signal, purpose-built
  checks” rather than “is every page referenced somewhere”.

- Golden files (tracked): `622`
- Golden keys (tracked, normalized `.open` suffix): `578`
- Gated keys (string-literal heuristic): `578` (`100%`)
- Targeted gates (excluding `web_vs_fret_layout.rs`, `snapshots.rs`): `578` (`100%`)
- Smoke-parse coverage: `100%` (via `shadcn_web_goldens_smoke_parse_and_rects_valid`)

As of 2026-02-03 (tracked-only).

Note on “targeted” gates:

- Most targeted keys have purpose-built assertions, but a small number are still *contract-only* checks
  (e.g. “golden contains expected tags”) while the deeper paint/layout assertions remain in the
  broad `web_vs_fret_layout.rs` file (notably `typography-table`).

Top missing prefixes (heuristic grouping by the substring before the first `.` or `-`):

```bash
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --as-markdown --group-missing-by-prefix --top-groups 20
```

At the time of writing, there are no missing groups (all keys are gated by tests).

The largest gated groups (already gated somewhere in `ecosystem/fret-ui-shadcn/tests`) were
(heuristic grouping by key prefix):

- `chart` (92)
- `calendar` (44)
- `button` (30)
- `input` (27)
- `navigation` (20)
- `form` (19)
- `typography` (16)
- `menubar` (16)
- `sidebar` (16)
- `sonner` (15)
- `dropdown` (18)
- `context` (14)
- `select` (13)
- `toggle` (13)
- `date` (12)
- `field` (12)
- `sheet` (11)
- `combobox` (12)
- `item` (11)
- `scroll` (10)
- `spinner` (10)
- `breadcrumb` (10)
- `textarea` (8)
- `empty` (7)
- `carousel` (6)
- `drawer` (6)
- `otp` (5)
- `signup` (5)
- `login` (5)
- `kbd` (5)
- `alert` (5)
- `command` (5)
- `badge` (4)
- `checkbox` (4)
- `native` (4)
- `dialog` (4)
- `resizable` (4)
- `tooltip` (3)
- `hover` (3)
- `popover` (3)

Sidebar note (2026-02-08): key-level gate coverage (`sidebar` keys gated) should not be treated as
full component-surface parity. Sidebar currently has dedicated component-surface + Base UI contract
tracking in `docs/audits/shadcn-sidebar.md`.

Top untargeted groups (i.e. keys only referenced by broad gates like `web_vs_fret_layout.rs` / `snapshots.rs`):

```bash
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --group-untargeted-by-prefix --top-groups 20
```

At the time of writing, there are no untargeted groups (all keys are referenced by purpose-built
gate tests outside the broad catch-all files).

Recompute locally:

```bash
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --show-missing --top-missing 50
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --show-targeted-missing --top-missing 50
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --show-used
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --group-used-by-prefix --top-groups 20
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --show-gate-breakdown
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --tracked-only --group-untargeted-by-prefix --top-groups 20
```

To drill into a specific family:

```bash
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --as-markdown --filter-missing-prefix calendar
python3 tools/golden_coverage.py --kind shadcn-web --style v4/new-york-v4 --as-markdown --filter-missing-prefix carousel
```

## What to do next (recommended order)

1. **Depth push**: convert “smoke” gates into high-signal conformance checks (especially where layout depends on viewport).
2. **Add constrained viewport variants** early for overlay-like components:
   - menus/listboxes: max-height clamp + scroll buttons + row height (treat “menu height” as a styling outcome)
   - popovers/tooltips: flip/shift under low height
   - dialogs/sheets: insets under low height/width
4. **DPI / font-metrics gates**: keep this dimension small and targeted until chart geometry is stable
   (recommended targets: typography + menus/listboxes).

## Common “coverage smells”

- Many goldens exist, but none are gated by tests (coverage drift).
- A component is gated only at one viewport size, but it changes behavior at others (false confidence).
- A gate checks only placement while ignoring geometry (panel width/height), allowing “menu height”
  regressions to slip through.
