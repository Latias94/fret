# Foreground Style Context (Fearless Refactor v1) — Milestones

Status: Draft

Related:

- Design: `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/foreground-style-context-fearless-refactor-v1/TODO.md`

---

## M0 — Problem locked and risk surface audited

Exit criteria:

- The current `ForegroundScope` contract is described clearly enough that reviewers can explain why
  it is risky.
- All `scope_children(...)` and direct `foreground_scope(...)` call sites are inventoried.
- High-risk migration candidates are identified and grouped by area.

---

## M1 — Design accepted for inherited foreground v1

Exit criteria:

- The preferred design is explicit: inherited foreground is context, not a synthetic layout
  fragment.
- Ownership is clear across `crates/fret-ui`, `fret-ui-kit`, and `fret-ui-shadcn`.
- The compatibility stance for `ForegroundScope` is explicit.
- It is clear whether v1 is foreground-only or also includes broader text-style inheritance.

---

## M2 — Mechanism path landed behind compatibility

Exit criteria:

- `crates/fret-ui` has a mechanism path for inherited foreground that does not require an
  author-facing wrapper node.
- Real subtree roots can install inherited foreground.
- Core consumers resolve foreground by the documented precedence contract.
- Legacy `ForegroundScope` still works during migration.

---

## M3 — First migration slice landed on high-risk surfaces

Exit criteria:

- At least the first high-risk recipe surfaces no longer depend on `scope_children(...)`.
- Wrapped-text and overlay-content regressions are gated.
- The migration path is demonstrated in one or more representative recipe families.

Recommended first slice:

- `dropdown_menu`
- `select`
- `tabs`
- `input_group`

---

## M4 — Public authoring story simplified

Exit criteria:

- New docs/examples teach a single preferred path for inherited foreground.
- `scope_children(...)` is clearly transitional, deprecated, or removed from guidance.
- Reviewers can identify wrapper-shaped inheritance as legacy rather than normal authoring.

---

## M5 — Cleanup decision on `ForegroundScope`

Exit criteria:

- The public fate of `ForegroundScope` is decided:
  - supported compatibility surface,
  - deprecated public surface,
  - or internal-only bridge.
- Any required ADR has been added or updated.
- Compatibility tests and migration notes match the final decision.

---

## M6 — Optional v2 decision: full text-style cascade

Exit criteria:

- A conscious decision exists on whether Fret should support a broader inherited text-style context.
- If yes, scope and risks are documented separately from v1 foreground inheritance.
- If no, docs explicitly say that v1 stops at foreground inheritance.

This milestone is intentionally separate because it is a broader contract than the v1 problem.
