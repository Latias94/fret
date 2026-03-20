# View-Locals Authoring (Fearless Refactor v1) — Milestones

Status: closed
Last updated: 2026-03-20

## M1 — Open the lane and freeze the rule

- Add the dedicated workstream docs.
- Record the no-new-API decision.
- Freeze the recommendation:
  - tiny inline locals stay allowed,
  - grouped view-owned locals prefer `*Locals`,
  - router/query/storage contract remain out of scope.

Exit criteria:

- `DESIGN.md`, `TARGET_INTERFACE_STATE.md`, and `TODO.md` agree on the lane boundary.
- `docs/README.md` and `docs/roadmap.md` point to the new lane.

## M2 — Prove the rule on canonical default app surfaces

- Migrate the default Todo compare set:
  - `simple_todo_demo`
  - `todo_demo`
  - cookbook `simple_todo`
  - cookbook `simple_todo_v2_target`
- Update scaffold templates so generated apps teach the same organization rule.
- Update source-policy tests that currently assert the older helper shape.

Exit criteria:

- First-party default app examples and generated templates converge on the same local-bundle shape.
- Source-policy tests assert `*Locals` markers rather than legacy forwarding helpers.

## M3 — Prove it is not Todo-only

- Migrate at least one non-Todo app-lane example (`form_basics` first).
- Update golden-path docs and README guidance to teach the bundle rule explicitly.

Exit criteria:

- The lane has at least one non-Todo proof surface.
- Docs, examples, and scaffold guidance say the same thing.

## M4 — Close or narrow

- Run the relevant tests/gates.
- Decide whether any residue remains beyond wording maintenance.

Exit criteria:

- If all targeted surfaces converge with no new API need, close the lane with a closeout audit.
- If residue remains, it must be named precisely and promoted only as another narrower lane.
