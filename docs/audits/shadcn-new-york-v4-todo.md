# shadcn/ui new-york-v4 TODO (Fret)

This is a short, high-signal backlog to keep the “goldens-driven 1:1” effort grounded.
Prefer adding deterministic gates over adding more smoke coverage.

## P0 (Overlays)

- Menus: destructive *idle* vs *focused* state matrix (ContextMenu done; replicate where applicable).
- Menus/listboxes: add a constrained-viewport golden gate that asserts **menu height** + scroll buttons/clamp behavior.
- Menubar destructive variant: **not exercised** by `menubar-demo` in upstream `new-york-v4` registry.
  Options:
  - Add a dedicated upstream example (requires upstream changes; avoid), or
  - Introduce a “golden-only” harness page in our extractor (keeps repo-ref pristine), or
  - Add a separate style harness that includes `base-nova` examples (new goldens + theme alignment).

## P0 (Chart)

- Convert `chart-*` from broad gates to **targeted** gates (tracked keys are currently concentrated under `web_vs_fret_layout.rs` / `snapshots.rs`).
- Start with high-signal geometry contracts (tooltip/legend/pie legend panels) and add a small set of viewport variants early.

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
