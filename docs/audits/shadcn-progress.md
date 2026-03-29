# shadcn/ui v4 Audit - Progress

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Progress` against the upstream shadcn/ui v4 docs and
registry implementations in `repo-ref/ui`, with `repo-ref/primitives` / `repo-ref/base-ui` checked
as secondary headless references.

## Upstream references (source of truth)

Current source axes:

- Docs pages:
  `repo-ref/ui/apps/v4/content/docs/components/radix/progress.mdx`,
  `repo-ref/ui/apps/v4/content/docs/components/base/progress.mdx`
- Registry implementation (new-york visual baseline):
  `repo-ref/ui/apps/v4/registry/new-york-v4/ui/progress.tsx`
- Base/radix registry copies (secondary structure check):
  `repo-ref/ui/apps/v4/registry/bases/radix/ui/progress.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/base/ui/progress.tsx`
- Radix primitive:
  `repo-ref/primitives/packages/react/progress/src/progress.tsx`
- Base UI parts surface:
  `repo-ref/base-ui/packages/react/src/progress/*`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/progress.rs`
- Shared primitive: `ecosystem/fret-ui-kit/src/primitives/progress.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/progress.rs`
- Copyable snippets: `apps/fret-ui-gallery/src/ui/snippets/progress/*.rs`

## Audit checklist

### Authoring surface

- Pass: `Progress::from_value(...)` covers the common shadcn prop-driven `value` path without
  forcing a `Model<f32>` at the call site.
- Pass: `Progress::new(model)` remains the explicit model-backed lane for timer-driven or
  shared-state demos.
- Pass: `Progress::new_opt(model)` matches the upstream `value || 0` outcome by rendering `None`
  as 0%.
- Pass: `Progress::new_values_first(model)` provides the narrow bridge needed for slider-style
  `Vec<f32>` models.
- Pass: UI Gallery now teaches the snapshot/value lane first (`Usage`), keeps the upstream timer
  demo shape (`Demo`), and preserves the model-backed synchronized lane under `Controlled`.
- Note: `Progress` stays a leaf display control on the default shadcn lane, so Fret intentionally
  does not add a generic composable children / `compose()` API here.
- Note: Base UI's `ProgressLabel` / `ProgressValue` parts remain a useful headless reference, but
  that is a different public surface from the default shadcn recipe lane.

### Layout, visuals, and motion parity

- Pass: Track uses `primary/20` with `rounded-full` chrome, matching the upstream shadcn recipe.
- Pass: Default height matches `h-2`.
- Pass: Indicator fills the full width and uses a translate transform driven by the current value,
  matching the upstream `translateX(-${100 - value}%)` outcome.
- Pass: The timer demo keeps the upstream `13 -> 66` update after `500ms`.
- Pass: RTL mirroring is caller-owned in upstream examples (`rtl:rotate-180` on the call site), and
  Fret mirrors that lane explicitly via `mirror_in_rtl(true)` instead of widening the default root.

### Semantics defaults

- Pass: `role=ProgressBar`, horizontal orientation, numeric min/max/value, and default percentage
  value text are stamped on determinate progress.
- Pass: Indeterminate (`None`) progress omits numeric/value semantics while still rendering the
  shadcn `value || 0` visual outcome.
- Pass: Range normalization and percentage labeling are owned by the shared Radix-aligned helper in
  `ecosystem/fret-ui-kit/src/primitives/progress.rs`.

### Docs / gallery parity

- Pass: UI Gallery now keeps the docs-path section order explicit: `Demo`, `Usage`, `Label`,
  `Controlled`, `RTL`, and `API Reference`, with `Notes` as the focused Fret follow-up.
- Pass: The page now records all relevant source axes directly, so future audits can distinguish
  between shadcn visual baseline, Radix semantics, and Base UI parts-surface references.
- Pass: The page now states the children-API decision explicitly, so the lack of a generic children
  API is documented as an intentional lane choice rather than an accidental gap.

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer gap.
- Result: The concrete alignment work is primarily recipe/public-surface and gallery teaching-surface
  closure, not `crates/fret-ui` runtime work.
- Result: Base UI's parts API remains a credible future alternate surface, but it is not a blocker
  for the default shadcn lane and does not justify widening `Progress` today.

## Validation

- `cargo nextest run -p fret-ui-shadcn progress::tests::progress_from_value_stamps_numeric_value_and_default_value_text progress::tests::progress_mirror_in_rtl_flips_translate_fraction progress::tests::progress_opt_none_matches_shadcn_value_or_zero_and_stamps_semantics snapshot_progress_numeric_semantics web_vs_fret_progress_demo_control_chrome_matches progress::web_vs_fret_layout_progress_demo_track_and_indicator_geometry_light progress::web_vs_fret_layout_progress_demo_track_and_indicator_geometry_dark --status-level fail`
- `cargo build -p fret-ui-gallery`
- `cargo test -p fret-ui-gallery --test progress_docs_surface -- --nocapture`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/progress/ui-gallery-progress-docs-smoke.json --dir /tmp/fret-progress-docs-smoke --session-auto --launch -- cargo run -p fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/progress/ui-gallery-progress-numeric-semantics.json --dir /tmp/fret-progress-semantics --session-auto --launch -- cargo run -p fret-ui-gallery`
