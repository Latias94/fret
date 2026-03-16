# Dataflow Authoring Surface (Fearless Refactor v1) — Target Interface State

Last updated: 2026-03-16

This file records the intended target posture for the post-closeout dataflow authoring lane.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`

The purpose is not to freeze exact final method names too early.
The purpose is to freeze:

- which tier owns each surface,
- which path is the default teaching path,
- which path stays advanced/editor-grade,
- and which adjacent domains remain out of scope.

## Target matrix

| Need | Default app lane | Advanced / editor-grade lane | Reusable ecosystem lane | Owner |
| --- | --- | --- | --- | --- |
| View-owned local writes | one obvious LocalState-first write story | explicit raw `Model<T>` writes remain available | optional adapters only if truly reusable | `ecosystem/fret` facade over app/runtime semantics |
| Multi-slot LocalState transaction | one canonical transaction story | explicit shared-model coordination remains separate | no forced `fret` dependency | `ecosystem/fret` |
| Keyed payload row writes | one canonical payload-row helper path | multi-model payload orchestration stays explicit | optional adapters only | `ecosystem/fret` |
| App-only effect handoff | one explicit transient/effects story | host/runtime seams remain explicit | generally app-only | `ecosystem/fret` + existing runtime semantics |
| Shared model graph coordination | not default | explicit `Model<T>` / shared ownership lane | direct-crate usage remains supported | existing runtime/app semantics |
| LocalState-first derived values | one default selector story that does not require raw dependency-builder knowledge on first contact | raw dependency signatures remain explicit | direct `fret-selector` usage stays valid | app-facing sugar in `ecosystem/fret`; engine in `fret-selector` |
| Shared-model derived values | not default | explicit raw selector dependency path | direct `fret-selector` usage stays valid | `fret-selector` |
| Async resource creation | explicit key + policy + fetch stay visible | full handle/state machine remains available | direct `fret-query` usage stays valid | `fret-query` engine, optional facade sugar |
| Async resource reads | one shorter default read-side posture | full handle/status/value surface remains available | direct `fret-query` usage stays valid | app-facing sugar in `ecosystem/fret`; engine in `fret-query` |
| Router state / navigation | adjacent only; not part of the default dataflow lane | explicit route/history/store semantics remain in router workstreams | direct `fret-router` / `fret-router-ui` usage | router workstreams, not this lane |

## Teaching posture

### Default app lane

The default app lane should teach:

- `use fret::app::prelude::*;`
- `LocalState<T>` for view-owned state
- one compact action dialect
- one compact LocalState-first selector dialect
- one explicit but lower-noise query dialect

It should not teach by default:

- raw `Model<T>` handles,
- raw `DepsBuilder` choreography,
- router/history/link policy,
- multiple co-equal write helper families for the same common use case.

### Advanced / editor-grade lane

The advanced lane must remain strong enough for:

- shared document/workspace graphs,
- background indexing and query invalidation,
- command-heavy/editor-grade surfaces,
- multi-view/window coordination,
- route-aware desktop applications.

The goal is not to hide this lane.
The goal is to keep it intentionally explicit instead of letting it leak into first-contact docs.

### Reusable ecosystem lane

Reusable ecosystem libraries should be able to:

- depend directly on `fret-selector` / `fret-query` when needed,
- provide optional adapters for the default app lane,
- stay free of app-owned router/runtime assumptions unless the crate actually targets that tier.

## Delete-ready rules

Once the new default surface lands:

- old co-equal default spellings should be removed from first-contact docs/templates/examples,
- advanced/raw spellings should remain only where they are genuinely advanced,
- source-policy tests should lock the chosen default path directly.
