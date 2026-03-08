# Shadcn Source Alignment v1 TODO

Status: active
Last updated: 2026-03-08

Related:

- Alignment: `docs/workstreams/shadcn-source-alignment-v1/ALIGNMENT.md`
- Milestones: `docs/workstreams/shadcn-source-alignment-v1/MILESTONES.md`
- Declarative progress: `docs/shadcn-declarative-progress.md`

## Priority order

1. Form-control families with shared `control_id(...)` / label association pressure
2. Trigger/listbox families (`Select`, `Combobox`, `DatePicker`)
3. Overlay composition families (`Dialog`, `Popover`, `Drawer`, `Tooltip`)
4. Menu/navigation families
5. Data/composite families only if teaching-surface pressure remains high

## Backlog table

| ID | Priority | Family | Scope | Status | Evidence target | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SSA-001 | P0 | Input / Textarea | Re-audit post-v1 text-value bridge against upstream public surface and remaining authoring friction | Pending | audit note + cookbook/doc references | The bridge landed; now verify whether any remaining source-level drift still matters. |
| SSA-002 | P0 | Select | Audit trigger/listbox surface, `control_id(...)`, focus target, and overlay semantics | Pending | focused tests + audit/doc note | Good next family because it combines form control + overlay behavior. |
| SSA-003 | P0 | Combobox | Audit source surface and distinguish headless/listbox concerns from shadcn recipe concerns | Pending | focused tests + workstream note | Keep layering strict; avoid solving source drift by hiding it in demos. |
| SSA-004 | P0 | DatePicker | Recheck trigger association, builder-first content path, and field integration | Pending | focused tests + audit note | Should align with the same form-control contract as `Select`. |
| SSA-005 | P1 | Dialog | Audit root/content/trigger builder parity and note remaining eager seams | Pending | example + audit/workstream update | Use builder-first composition checklist from v2 golden path. |
| SSA-006 | P1 | Popover / Tooltip / Drawer | Audit trigger/content APIs and outside-press/focus outcomes | Pending | tests or diag scripts + audit update | Group only when they share the same migration lesson. |
| SSA-007 | P1 | DropdownMenu / Menubar | Audit constrained viewport, keyboard semantics, and action-first teaching examples | Pending | focused tests + docs | Keep separate from overlay chrome-only work. |
| SSA-008 | P2 | NavigationMenu | Re-audit interaction parity after menu family rules stabilize | Pending | audit note | Lower priority until base menu family is clearer. |
| SSA-009 | P2 | Table / composite builders | Check whether any remaining eager-only seams still block default teaching surfaces | Pending | cookbook/gallery evidence | Only do this if real authoring pressure remains. |

## Working checklist per family

| Check | Done? | Notes |
| --- | --- | --- |
| Upstream source/demo reviewed | No | Fill relative `repo-ref/ui/...` path in the family note. |
| Public surface gap written down before coding | No | State what feels more eager/model-centric than upstream. |
| Layer decision recorded | No | `fret-ui` vs `fret-ui-kit` vs `fret-ui-shadcn`. |
| Focus/label/payload semantics checked | No | Especially for form controls and trigger-based surfaces. |
| Focused regression gate added | No | Unit/integration test first; diag when interaction complexity justifies it. |
| Audit/workstream docs updated | No | Keep status traceable. |

## Deferred on purpose

| Topic | Why deferred | Revisit when |
| --- | --- | --- |
| New macros for shadcn authoring | Would freeze an unstable default surface too early | After local-state + action-default ergonomics settle |
| Broad helper proliferation | Risks making the surface less teachable | Only if at least two real families need the same shape |
| Cross-family mega-refactor | Too hard to review and easy to drift | Prefer family-by-family slices with evidence |

## Notes for the component author

| Situation | Recommended move |
| --- | --- |
| The component still forces `Model<T>` for a plain local value | First ask whether a narrow snapshot/value constructor should exist. |
| Label click or focus behavior is inconsistent | Check `control_id(...)` + registry contracts before adding widget-local patches. |
| The component still needs early `into_element(cx)` for normal composition | Prefer a root/section/trigger `build(...)` path first. |
| A demo is awkward but the component surface looks fine | Fix the demo last; do not hide a missing component API in app code. |
