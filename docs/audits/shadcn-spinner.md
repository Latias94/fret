# shadcn/ui v4 Audit — Spinner

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Spinner` against the upstream shadcn/ui v4 Radix docs,
new-york spinner source, example compositions, and the existing spinner unit/layout gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radix/spinner.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/spinner.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-custom.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-size.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-button.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-badge.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-input-group.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-empty.tsx`, `repo-ref/ui/apps/v4/examples/base/spinner-rtl.tsx`
- Existing gates: `ecosystem/fret-ui-shadcn/src/spinner.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/spinner.rs`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/spinner.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/spinner.rs`

## Audit checklist

### Authoring surface

- Pass: `Spinner::new()` covers the upstream leaf spinner path.
- Pass: `refine_layout(...)`, `icon(...)`, and `color(...)` cover the documented customization story without widening the component into a container API.
- Pass: `speed(...)` remains a focused Fret-specific follow-up for diagnostics and recipes rather than part of the upstream docs path.
- Pass: Spinner is a visual leaf primitive, so no extra generic `compose()` / children API is needed here.

### Layout & default-style ownership

- Pass: the default loader icon, intrinsic 16px box, current-color behavior, and spin animation remain recipe-owned because the upstream component source defines those defaults on the spinner itself.
- Pass: explicit size overrides, page width caps, and embedding inside button / badge / input-group layouts remain caller-owned or host-recipe-owned composition choices.
- Pass: existing unit and web-vs-Fret layout gates already cover the default spin timing and key geometry outcomes; this pass does not reveal a mechanism-layer gap.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream Spinner docs path first: `Demo`, `Usage`, `Customization`, `Size`, `Button`, `Badge`, `Input Group`, `Empty`, and `RTL`.
- Pass: `Extras` and `API Reference` remain explicit Fret follow-ups after the upstream path because they document Fret-only refinements and ownership notes.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --lib spinner`
- Existing geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/spinner.rs`
