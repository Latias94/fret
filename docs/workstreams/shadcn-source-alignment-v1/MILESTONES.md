# Shadcn Source Alignment v1 Milestones

Status: active
Last updated: 2026-03-08

Related:

- Alignment: `docs/workstreams/shadcn-source-alignment-v1/ALIGNMENT.md`
- TODO: `docs/workstreams/shadcn-source-alignment-v1/TODO.md`

## Milestone table

| Milestone | Goal | Exit criteria | Status |
| --- | --- | --- | --- |
| M0 Baseline | Establish shared migration checklist and status board | Alignment/TODO/milestones docs exist and identify current family status + workflow | Done |
| M1 Form-control closure | Stabilize reusable form-control source-alignment pattern | `Input`/`Textarea`/`Select`/`Combobox`/`DatePicker` each have a documented source-surface decision and shared `control_id(...)` rules stay consistent | In progress |
| M2 Overlay composition closure | Stabilize builder-first migration rules for overlay families | `Dialog`/`Popover`/`Drawer`/`Tooltip` have a documented composition checklist and at least one representative focused gate each | Pending |
| M3 Menu/navigation parity | Audit command/listbox/menu families with constrained viewport semantics | `DropdownMenu`/`Menubar`/`NavigationMenu` have audit outcomes and focused regression gates for the highest-risk interactions | Pending |
| M4 Teaching-surface convergence | Ensure cookbook/gallery examples reflect the aligned public surfaces | New and updated teaching surfaces use the intended default component APIs instead of local workaround glue | Pending |

## Current milestone notes

| Milestone | Evidence | Notes |
| --- | --- | --- |
| M0 | `docs/workstreams/shadcn-source-alignment-v1/ALIGNMENT.md`, `TODO.md`, `MILESTONES.md` | This is the documentation landing step. |
| M1 | `docs/workstreams/control-id-form-association-v1/ALIGNMENT.md`, `docs/audits/shadcn-checkbox.md`, `docs/audits/shadcn-switch.md`, `docs/audits/shadcn-toggle.md` | Discrete controls already provide one reusable source-alignment pattern; form-control families are the next highest-value target. |
| M4 | `apps/fret-cookbook/examples/toggle_basics.rs`, `apps/fret-cookbook/examples/simple_todo_v2_target.rs` | Teaching-surface evidence already exists for some local-state + action-first paths, but broader shadcn family coverage is still incomplete. |

## Suggested rollout order

| Order | Family set | Why now |
| --- | --- | --- |
| 1 | `Select`, `Combobox`, `DatePicker` | Highest leverage: combines public surface, form association, and overlay interaction concerns. |
| 2 | `Dialog`, `Popover`, `Drawer`, `Tooltip` | Builder-first composition rules should be stabilized family-by-family before more helpers appear. |
| 3 | `DropdownMenu`, `Menubar`, `NavigationMenu` | Best done after base overlay/listbox rules stop moving. |
| 4 | Remaining composite/data-heavy surfaces | Lower urgency unless a teaching-surface example proves real pressure. |

## Definition of done for a migrated family

| Requirement | Meaning |
| --- | --- |
| Source surface documented | The common-path constructor/builder delta vs upstream is written down. |
| Layer ownership clear | Mechanism vs recipe changes are separated cleanly. |
| Focused regression gate exists | At least one narrow test or diag script protects the migrated behavior. |
| Audit/workstream synced | The family status can be read without opening the implementation diff. |
| Teaching example honest | At least one in-tree example shows the intended authoring path without workaround glue. |
