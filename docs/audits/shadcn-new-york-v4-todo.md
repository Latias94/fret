# shadcn/ui new-york-v4 TODO (Fret)

This is a short, high-signal backlog to keep the “goldens-driven 1:1” effort grounded.
Prefer adding deterministic gates over adding more smoke coverage.

Status:

- Tracked shadcn-web `v4/new-york-v4` keys are now **100% gated** and **100% targeted-gated** (excluding `web_vs_fret_layout.rs` / `snapshots.rs`).
  The remaining work is to raise signal: migrate deeper geometry/paint assertions out of broad files and add viewport variants for behavior-shaping components.

## P0 (Overlays)

- Menus: destructive *idle* vs *focused* state matrix (ContextMenu done; replicate where applicable).
- Menus/listboxes: constrained-viewport gates now assert **menu/listbox height** + scroll behavior for the primary demos
  (DropdownMenu/ContextMenu/Menubar/SelectScrollable/Combobox/NavigationMenu). Next: extend the same pattern to any
  remaining overlay families that expose viewport-dependent sizing.
- DatePicker open overlays:
  - Added popover placement gates for `date-picker-demo.open` and `date-picker-with-range.open`.
  - Next: implement + gate `date-picker-with-presets.open` (Select + calendar composition inside the popover).
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
