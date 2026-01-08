# Radix Primitives Audit — Portal

This audit compares Fret's portal substrate against the upstream Radix
`@radix-ui/react-portal` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/portal/src/portal.tsx`
- Public exports: `repo-ref/primitives/packages/react/portal/src/index.ts`

Key upstream concepts:

- `Portal` renders content into a different DOM container.
- If no `container` is provided, Radix defaults to `document.body` after mount.

## Fret mapping

Fret has no DOM. Layering/portal outcomes are modeled via per-window overlay roots (ADR 0011).

- Overlay root install/uninstall: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Runtime multi-root substrate: `crates/fret-ui` layer stack (ADR 0011)
- Radix-named facade (naming + scoping helper): `ecosystem/fret-ui-kit/src/primitives/portal.rs`

## Current parity notes

- Pass: A stable "portal target" can be represented via a root name.
- Pass: Callers can render within a portal root scope using `with_portal_root_name(...)`.

## Gaps / intentional differences

- Intentional: there is no concept of an arbitrary DOM container; portals target overlay roots.
- Deferred: general-purpose "custom portal root install" API; most consumers should use overlay
  primitives (`popover`, `dialog`, `menu`, etc.) which compose portal + dismissal + focus outcomes.

