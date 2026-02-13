# Mind model: Models, actions, and commands

Goal: structure app state and interactions so UI remains declarative and debuggable.

## Prefer `Model<T>` for UI state

- Keep long-lived state in app/runtime models and pass `Model<T>` into shadcn recipes.
- Treat components as pure renderers + event wiring; avoid hidden global state in recipes.

## Use commands for app actions

Prefer `CommandId`-based wiring for app-level actions:

- Use typed messages as the mutation boundary.
- Reserve literal `CommandId` strings for globally addressable keymap/menu actions.

For dynamic actions:

- `MessageRouter<M>` for per-frame dynamic routes.
- `KeyedMessageRouter<K, M>` when commands are emitted inside view-cached subtrees.

- Enables command gating (disabled states) consistently.
- Keeps UI and app logic loosely coupled.
- Improves automation: scripts can click by `test_id` and you can also assert command gating traces in bundles.

## Split state by responsibility

Use a three-layer model instead of one large mutable blob:

- Local mutable state: `Model<T>` and element-local state helpers.
- Derived state: `fret-selector` (`Selector`, `use_selector`) for memoized read-only projections.
- Async resource state: `fret-query` (`QueryClient`, `use_query*`) for loading/error/cache lifecycle.

This split keeps UI code predictable and avoids ad-hoc refresh counters or one-off async caches.
## Use action hooks for component policy

For cross-cutting policies (dismiss on escape/outside press, focus restore, toggle behaviors):

- Keep the mechanism in `fret-ui` (hooks plumbing),
- Keep the policy in `fret-ui-kit` / `fret-ui-shadcn` (action hooks / handlers),
- Avoid adding “policy shortcuts” directly to `fret-ui` primitives.

## Regression default

When adding a new interactive surface:

1. Provide stable `test_id` on trigger + key nodes.
2. Add a minimal `tools/diag-scripts/*.json` script to reproduce the interaction.
3. Add one invariant test for the most fragile semantics/geometry.

## See also

- `fret-diag-workflow` (scripted repro + packaging)
- Command routing and keymaps: `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0022-when-expressions.md`, `docs/adr/0023-command-metadata-menus-and-palette.md`
- Action hooks (policy lives in components): `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
