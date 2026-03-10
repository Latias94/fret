# shadcn/ui v4 Audit — Native Select

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `NativeSelect` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radix/native-select.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/native-select.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/native-select-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/native-select-groups.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/native-select-disabled.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/native-select-invalid.tsx`, `repo-ref/ui/apps/v4/examples/base/native-select-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/native_select.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/native_select.rs`

## Audit checklist

### Authoring surface

- Pass: `NativeSelect::new(model, open)` covers the fully controlled authoring path.
- Pass: `NativeSelect::new_controllable(...)` covers the common `defaultValue` / uncontrolled authoring path.
- Pass: `options(...)`, `optgroups(...)`, `control_id(...)`, `placeholder(...)`, `size(...)`, `disabled(...)`, and `aria_invalid(...)` cover the documented recipe surface.
- Pass: `options(...)` and `optgroups(...)` are the source-aligned structured equivalent of upstream option/optgroup children, so Fret does not need an extra generic `children` / `compose()` builder here.

### Contract gap vs upstream DOM native select

- Pass: upstream `NativeSelect` wraps a DOM `<select>` and inherits browser-native picker behavior.
- Pass: Fret's current `NativeSelect` is a popover-backed fallback that preserves the shadcn authoring shape and form semantics, but it does not yet provide backend-native select widgets.
- Pass: prefer `Select` when you need rich overlay behavior today; revisit `NativeSelect` once platform-native select surfaces are implemented per backend.

### Layout & default-style ownership

- Pass: trigger chrome follows the same input recipe tokens as `Input`, keeping height, border, focus ring, invalid state, and chevron layout aligned with the form field family.
- Pass: upstream `size="default"` / `size="sm"` heights are now explicitly locked by a unit gate in `ecosystem/fret-ui-shadcn/src/native_select.rs`.
- Pass: surrounding width caps, form/page layout, and when to use `Select` instead remain caller-owned decisions.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Groups`, `Disabled`, `Invalid`, `Native Select vs Select`, and `RTL`.
- Pass: `Label Association` and `API Reference` remain focused Fret follow-ups after the upstream path because they document control-registry wiring and ownership/defer notes.
- Pass: this work is docs/public-surface parity plus a small surface gate, not a mechanism-layer fix.

### Defer rationale

- Pass: the current shadcn-facing authoring surface is already audited enough to avoid inventing new APIs.
- Pass: status remains `Defer` because true parity ultimately depends on backend-native select widgets, which are not yet the active priority.
- Pass: follow-up work should resume when platform-native select plumbing becomes active or when a concrete authoring/semantics regression appears.

## Validation

- `CARGO_TARGET_DIR=target-codex-native-select cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-native-select cargo test -p fret-ui-shadcn --lib native_select`
