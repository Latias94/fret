# Action Write Surface (Fearless Refactor v1) — Target Interface State

Last updated: 2026-03-17

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `CLOSEOUT_AUDIT_2026-03-17.md`

This file records the shipped target posture for this now-closed default write-side authoring lane.

Important constraint:

- this lane freezes write-shape ownership and teaching posture before it freezes exact helper
  renames
- changing names is optional; changing the product surface confusion is not

## Target matrix

| Need | Default app lane | Explicit / advanced lane | Reusable ecosystem lane | Owner |
| --- | --- | --- | --- | --- |
| Single-local write | one intentionally small companion family on `cx.actions()`: `local_update`, `local_set`, and `toggle_local_bool` | raw model writes remain explicit | no forced `fret` dependency | `ecosystem/fret` |
| Coordinated LocalState transaction | `locals::<A>(...)` unless a clearly better non-Todo replacement is proven | shared-model coordination remains explicit | no forced `fret` dependency | `ecosystem/fret` |
| Keyed payload row write | one canonical row-write helper: `payload_local_update_if::<A>(...)` | collection/model orchestration remains explicit | no forced `fret` dependency | `ecosystem/fret` |
| Multi-local payload transaction | not taught on the default path | `payload_locals::<A>(...)` remains an explicit advanced/reference seam until first-party proof exists | no forced `fret` dependency | `ecosystem/fret` |
| App-only effect handoff | `transient::<A>(...)` stays explicit | host/runtime seams remain explicit | generally app-only | `ecosystem/fret` + existing runtime semantics |
| Shared model graph coordination | not default | `models::<A>(...)` remains explicit | direct-crate usage remains supported | existing runtime/app semantics |
| Widget activation glue | adjacent only; widget-native `.action(...)` / `.action_payload(...)` / `.listen(...)` stay on their own lane | raw `.on_activate(...)` remains available | direct widget contracts remain supported | widget/component surfaces, not this lane |
| Selector/query authoring | adjacent only; already tracked and closed elsewhere | explicit selector/query surfaces remain available | direct `fret-selector` / `fret-query` usage stays valid | `dataflow-authoring-surface-fearless-refactor-v1` closeout |
| Router state / navigation | adjacent only; not part of this lane | explicit route/history/store semantics remain in router workstreams | direct `fret-router` / `fret-router-ui` usage | router workstreams |

## Teaching posture

### Default app lane

The default app lane should teach:

- one obvious one-slot companion family for semantically distinct writes
- one obvious coordinated transaction story
- one obvious keyed payload row-write story
- explicit `transient::<A>(...)` when the real work belongs in `&mut App`
- explicit `models::<A>(...)` only for shared ownership

It should not teach by default:

- multiple co-equal one-slot helper families beyond the chosen budget
- `payload_locals::<A>(...)` as a co-equal or reserve default row-write path
- `payload::<A>()` as a first-contact row-write path
- widget activation glue as a substitute for root write ownership
- shared-model coordination as the ordinary view-owned write path

Additional rule:

- `locals::<A>(...)` remains the primary explicit transaction story
- the one-slot trio is an intentional companion family, not a second transaction dialect
- keyed payload row writes teach `payload_local_update_if::<A>(...)` alone on the default path

### Advanced / editor-grade lane

The advanced lane must remain strong enough for:

- workspace/document graphs,
- render-time/runtime-owned effects,
- route-aware state coordination,
- background work and host-owned side effects.

This lane should stay explicit rather than leaking back into default app teaching.

### Reusable ecosystem lane

Reusable ecosystem libraries should be able to:

- stay independent of `fret` unless they intentionally target the default app lane
- keep widget contracts, selector/query usage, and router usage explicit
- add optional adapters only when the crate genuinely wants to author against `AppUi`

## Promotion rule

Do not promote a new write helper into the default path unless:

1. at least one generic app surface and one non-Todo runtime surface need the same shape,
2. the current default path is materially noisier on both,
3. editor-grade compatibility remains explicit and intact,
4. reusable ecosystem crates are not forced onto the wrong dependency tier.

## Delete-ready rule

Once the final write-side posture is chosen:

- old default-looking spellings should disappear from first-contact docs/templates/examples,
- source-policy gates should lock the chosen posture directly,
- displaced helpers should either become explicit advanced/reference seams or be deleted outright.
