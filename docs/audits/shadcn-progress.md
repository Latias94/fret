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

- Pass: `Progress::new(model)` covers the common shadcn authoring path.
- Pass: `Progress::new_opt(model)` matches the upstream `value || 0` outcome by rendering `None` as 0%.
- Pass: `Progress::new_values_first(model)` provides a useful bridge for slider-style `Vec<f32>` models.
- Note: `Progress` is a leaf display control, so Fret intentionally does not add a generic `compose()` builder here.

### Visual defaults (shadcn parity)

- Pass: Track uses `primary/20` with `rounded-full` chrome.
- Pass: Default height matches `h-2`.
- Pass: Indicator fills the full width and uses a translate transform driven by the current value, matching the upstream `translateX(-${100 - value}%)` outcome.

## Validation

- `cargo test -p fret-ui-shadcn --lib progress`