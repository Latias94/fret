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

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/native_select.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/native_select.rs`

## Audit checklist

### Authoring surface

- Pass: `NativeSelect::new(model, open)` covers the fully controlled authoring path.
- Pass: `NativeSelect::new_controllable(...)` covers the common `defaultValue` / uncontrolled authoring path.
- Pass: `options(...)`, `optgroups(...)`, `control_id(...)`, and `placeholder(...)` cover the important recipe surface from the upstream docs.
- Pass: `NativeSelect` already exposes the hooks it needs, so Fret intentionally does not add a generic `compose()` builder here.

### Contract gap vs upstream DOM native select

- Pass: upstream `NativeSelect` wraps a DOM `<select>` and inherits browser-native picker behavior.
- Pass: Fret's current `NativeSelect` is a popover-backed fallback that preserves the shadcn authoring shape and form semantics, but it does not yet provide backend-native select widgets.
- Pass: prefer `Select` when you need rich overlay behavior today; revisit `NativeSelect` once platform-native select surfaces are implemented per backend.

### Visual defaults and ownership

- Pass: trigger chrome follows the same input recipe tokens as `Input`, keeping height, border, focus ring, and invalid state aligned with the form field family.
- Pass: placeholder-first options and disabled options match the upstream authoring expectations from the docs examples.

### Defer rationale

- Pass: the current shadcn-facing authoring surface is already audited enough to avoid inventing new APIs.
- Pass: status remains `Defer` because true parity ultimately depends on backend-native select widgets, which are not yet the active priority.
- Pass: follow-up work should resume when platform-native select plumbing becomes active or when a concrete authoring/semantics regression appears.

## Validation

- `cargo test -p fret-ui-shadcn --lib native_select`
- `cargo check -p fret-ui-gallery`
