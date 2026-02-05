# Mind model: Models, actions, and commands

Goal: structure app state and interactions so UI remains declarative and debuggable.

## Prefer `Model<T>` for UI state

- Keep long-lived state in app/runtime models and pass `Model<T>` into shadcn recipes.
- Treat components as pure renderers + event wiring; avoid hidden global state in recipes.

## Use commands for app actions

Prefer `CommandId`-based wiring for app-level actions:

- Enables command gating (disabled states) consistently.
- Keeps UI and app logic loosely coupled.
- Improves automation: scripts can click by `test_id` and you can also assert command gating traces in bundles.

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

- `fret-commands-and-keymap` (command registry, keymap.json, `when` gating)
- `fret-action-hooks` (component-owned interaction policy)
- `fret-diag-workflow` (scripted repro + packaging)
