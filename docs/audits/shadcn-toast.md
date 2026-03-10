# shadcn/ui v4 Audit — Toast

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit records why Fret keeps `toast` as a compatibility surface while treating `sonner` as the
actual parity target.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radix/toast.mdx`
- Replacement surface: `repo-ref/ui/apps/v4/content/docs/components/base/sonner.mdx`

## Fret implementation

- Compatibility alias: `ecosystem/fret-ui-shadcn/src/toast.rs`
- Primary implementation: `ecosystem/fret-ui-shadcn/src/sonner.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/toast.rs`

## Audit checklist

### Surface classification

- Pass: upstream `toast` is explicitly deprecated and points users to `sonner`.
- Pass: Fret therefore treats `sonner` as the primary parity target and keeps `toast` only as a migration/compatibility alias.
- Pass: the `toast` module simply re-exports the Sonner-shaped surface, so there is no separate component contract to grow here.

### Gallery / docs parity

- Pass: the gallery keeps a minimal `Deprecated` page, matching the upstream docs intent instead of pretending there is still a standalone toast component to align.
- Pass: the deprecation card points callers toward the `sonner` surface, which is the real maintained implementation in this repo.
- Pass: this is a deliberate `Skip`, not a missing implementation.

## Validation

- Existing maintained surface: `docs/audits/shadcn-sonner.md`
