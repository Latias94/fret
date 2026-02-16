# Radix Primitives Audit бк Label


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned label substrate against the upstream Radix
`@radix-ui/react-label` implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/label/src/label.tsx`
- Public exports: `repo-ref/primitives/packages/react/label/src/index.ts`

Key upstream concepts:

- `Label` is a thin wrapper over a native `<label>`.
- It prevents accidental text selection on double-click (`onMouseDown`, `event.detail > 1`),
  except when interacting with form controls inside the label.

## Fret mapping

- Primitive facade: `ecosystem/fret-ui-kit/src/primitives/label.rs`
- shadcn re-export: `ecosystem/fret-ui-shadcn/src/label.rs`

## Current parity notes

- Pass: Provides a stable, discoverable `Label` surface as a primitives module.
- Note: Fret does not currently model native `<label>` semantics (relationships like
  `htmlFor` / `aria-labelledby`) as a first-class concept.
- Note: Fret does not currently model browser text-selection behavior; the Radix `onMouseDown`
  prevention logic does not translate directly to Fret's runtime today.

