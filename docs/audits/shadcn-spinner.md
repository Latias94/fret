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
- Secondary headless refs: `repo-ref/ui/apps/v4/registry/bases/radix/ui/spinner.tsx`, `repo-ref/ui/apps/v4/registry/bases/base/ui/spinner.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/examples/spinner-example.tsx`, `repo-ref/ui/apps/v4/registry/bases/base/examples/spinner-example.tsx`
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
- Pass: neither `repo-ref/primitives` nor `repo-ref/base-ui` defines a dedicated Spinner primitive, and the shadcn `registry/bases/{radix,base}` wrappers keep the same leaf `svg` contract, so the remaining work here is docs/recipe parity rather than a missing `fret-ui` / `fret-ui-kit` mechanism contract.

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
- Pass: the `Customization` preview now stays on the single customized spinner lane, matching the upstream docs more closely instead of teaching a before/after comparison row.
- Pass: the `RTL` preview now uses Arabic copy, which keeps the example closer to the upstream RTL docs surface instead of only flipping layout direction around English text.
- Pass: the page now records the source-axis decision inline (`repo-ref/ui` docs/source/examples, the `registry/bases/{radix,base}` leaf wrappers, and the explicit lack of `repo-ref/primitives` / `repo-ref/base-ui` spinner primitives) so future audits do not have to rediscover why this is a leaf recipe surface.
- Pass: UI Gallery docs-surface regression protection now includes `apps/fret-ui-gallery/tests/spinner_docs_surface.rs` plus `tools/diag-scripts/ui-gallery/spinner/ui-gallery-spinner-docs-screenshots.json`.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-a11y-accesskit maps_extended_semantics_roles_to_accesskit_roles`
- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --lib spinner`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-gallery --test spinner_docs_surface`
- `cargo nextest run -p fret-ui-gallery spinner_`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/spinner/ui-gallery-spinner-docs-screenshots.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- `python3 tools/check_diag_scripts_registry.py --write`
- Existing geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/spinner.rs`
