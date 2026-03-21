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
- Pass: a small project-local wrapper around `Spinner::new().icon(...)` is the closest Fret equivalent of the upstream "edit the Spinner component source" customization story.
- Pass: `speed(...)` remains a focused Fret-specific follow-up for diagnostics and recipes rather than part of the upstream docs path.
- Pass: Button/Badge/InputGroup already expose the required inline slot surfaces for spinner compositions, so Spinner itself does not need a generic `compose()` / children API.

### Semantics & accessibility

- Pass: Spinner now maps to status semantics (`role="status"` equivalent) rather than a numeric progress role, matching the upstream shadcn source more closely.
- Pass: the spinner keeps polite live-region behavior and atomic announcements without inventing numeric progress values.
- Pass: this audit revealed a small mechanism-layer gap in the shared semantics enum; adding `SemanticsRole::Status` was the right fix, rather than keeping a recipe-level `ProgressBar` approximation.

### Layout & default-style ownership

- Pass: the default loader icon, intrinsic 16px box, current-color behavior, and spin animation remain recipe-owned because the upstream component source defines those defaults on the spinner itself.
- Pass: explicit size overrides, page width caps, and embedding inside button / badge / input-group layouts remain caller-owned or host-recipe-owned composition choices.
- Pass: existing unit and web-vs-Fret layout gates already cover the default spin timing and key geometry outcomes; the remaining parity work here was semantic, not layout-related.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream Spinner docs path first: `Demo`, `Usage`, `Customization`, `Size`, `Button`, `Badge`, `Input Group`, `Empty`, and `RTL`.
- Pass: `Extras` and `API Reference` remain explicit Fret follow-ups after the upstream path because they document Fret-only refinements and ownership notes.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-a11y-accesskit maps_extended_semantics_roles_to_accesskit_roles`
- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --lib spinner`
- Existing geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/spinner.rs`
