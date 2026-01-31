# shadcn/ui new-york-v4 TODO (Fret)

This is a short, high-signal backlog to keep the “goldens-driven 1:1” effort grounded.
Prefer adding deterministic gates over adding more smoke coverage.

Status:

- Tracked shadcn-web `v4/new-york-v4` keys are now **100% gated** and **100% targeted-gated** (excluding `web_vs_fret_layout.rs` / `snapshots.rs`).
  The remaining work is to raise signal: migrate deeper geometry/paint assertions out of broad files and add viewport variants for behavior-shaping components.

## Plan adjustments (recommended)

To keep the effort “goldens-driven” without exploding scope, the plan should be explicitly staged:

1. **Breadth + determinism first** (one baseline viewport, deterministic data like frozen dates/timezones).
2. **Depth by family** (overlays/typography/calendar first, because they are viewport-sensitive).
3. **Only then add a small DPI/resolution matrix**, scoped to the families that actually drift with font metrics.

Important guardrail:

- Do **not** globally change “all text is `Fill`/block-level”. Treat block-vs-inline as a **recipe/component semantic**
  (e.g. `CardTitle` behaves like a `div`, while many labels are `w-fit` upstream).

## P0 (Overlays)

- Menus: destructive *idle* vs *focused* state matrix (ContextMenu done; replicate where applicable).
- Menus/listboxes: constrained-viewport gates now assert **menu/listbox height** + scroll behavior for the primary demos
  (DropdownMenu/ContextMenu/Menubar/SelectScrollable/Combobox/NavigationMenu). Next: extend the same pattern to any
  remaining overlay families that expose viewport-dependent sizing.
- DatePicker open overlays:
  - Added popover placement gates for `date-picker-demo.open`, `date-picker-with-presets.open`, and
    `date-picker-with-range.open`.
  - Added a nested listbox placement gate for `date-picker-with-presets.select-open.open` (Select listbox opened inside the popover).
  - Added a presets behavior gate for `date-picker-with-presets.preset-tomorrow.open` (trigger text + selected day ARIA label).
  - Remaining depth work: add a higher-signal nested overlay gate for stacking order + constrained viewport max-height/scroll behavior.
- NavigationMenu viewport content insets: current gates allow a small extra tolerance (`2px`) because the Fret test
  harness bounds are more integer-ish than the web's subpixel `border-box` snapshots. Prefer fixing this at the layout
  model level (border/box sizing), not by baking more per-component offsets.
- Menubar destructive variant: **not exercised** by `menubar-demo` in upstream `new-york-v4` registry.
  Options:
  - Add a dedicated upstream example (requires upstream changes; avoid), or
  - Introduce a “golden-only” harness page in our extractor (keeps repo-ref pristine), or
  - Add a separate style harness that includes `base-nova` examples (new goldens + theme alignment).

## P0 (Chart)

- `chart-*` keys are now covered by targeted gates; migrate the remaining higher-signal geometry assertions out of `web_vs_fret_layout.rs`.
- Focus next on tooltip/legend/pie label panels under constrained viewports (small width/height, overflow, long labels).

## P1 (Typography)

- Typography gates exist; add multi-width coverage (line wrapping, margins, list markers).
  - Done (v1): add `typography-demo.vp375x900` golden + a wrap/max-width contract gate (ensures prose paragraphs wrap under narrow widths).
  - Done (v1): add `typography-demo.vp768x900` golden + a wrap/max-width contract gate (ensures constraints stay stable at tablet widths).
  - Done (v1): add a targeted list indent + item gap contract gate derived from the web golden (`ml-6` + `mt-2`).
  - Next: expand list marker gates once the extractor records `list-style-*` properties (currently not in the computedStyle whitelist).

## P1 (Calendar)

- Calendar gates exist; expand month grid geometry + selection/hover states across viewports.

## P2 (Contracts / Variants)

- Prototype a “variants contract JSON” generator for one recipe-heavy component (e.g. Button).
  Evaluate maintenance cost before scaling to more components.

## P2 (Tooling)

- Keep `tools/golden_coverage.ps1` honest by reporting coverage as two dimensions (in addition to broad-gate exclusion):
  - **Gated** (string-literal heuristic, “referenced by tests”)
  - **Targeted** (excluding broad gates like `web_vs_fret_layout.rs` / `snapshots.rs`)
  - **Smoke-parse** (dynamic traversal, low-signal sanity)
  Avoid docs claiming “100% covered” unless both dimensions are stated.

## P3 (DPI / Resolution)

Do this only after the overlay + typography geometry is stable at the baseline viewport.

- Pick a tiny matrix and keep it high-signal:
  - 2 viewports (e.g. “default” + “constrained height”)
  - 2 scale factors (1.0 + 1.25 or 1.5)
  - Families: typography + menus/listboxes + calendar
