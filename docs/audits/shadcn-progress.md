# shadcn/ui v4 Audit - Progress


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Progress` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/progress.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/progress.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/progress.rs`
- Shared primitive: `ecosystem/fret-ui-kit/src/primitives/progress.rs`

## Audit checklist

### Authoring surface

- Pass: `Progress::from_value(...)` covers the common shadcn prop-driven `value` path without forcing a `Model<f32>` at the call site.
- Pass: `Progress::new(model)` remains the model-backed lane for timer-driven or shared-state demos.
- Pass: `Progress::new_opt(model)` matches the upstream `value || 0` outcome by rendering `None` as 0%.
- Pass: `Progress::new_values_first(model)` provides a useful bridge for slider-style `Vec<f32>` models.
- Pass: UI Gallery `Usage` now teaches the snapshot/value lane first, while `Controlled` keeps the model-backed lane explicit.
- Note: `Progress` stays a leaf display control on the default shadcn lane, so Fret intentionally does not add a generic children/`compose()` API here. Base UI's label/value parts remain a reference, not the default shadcn surface.

### Visual defaults (shadcn parity)

- Pass: Track uses `primary/20` with `rounded-full` chrome.
- Pass: Default height matches `h-2`.
- Pass: Indicator fills the full width and uses a translate transform driven by the current value, matching the upstream `translateX(-${100 - value}%)` outcome.

### Semantics defaults

- Pass: `role=ProgressBar`, horizontal orientation, numeric min/max/value, and default percentage value text are stamped on determinate progress.
- Pass: Indeterminate (`None`) progress omits numeric/value semantics while still rendering the shadcn `value || 0` visual outcome.

## Validation

- `cargo nextest run -p fret-ui-shadcn progress`
