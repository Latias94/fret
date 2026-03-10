# Shadcn Source Alignment v1

Status: active
Last updated: 2026-03-08

Related:

- Declarative progress tracker: `docs/shadcn-declarative-progress.md`
- Authoring migration guide: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
- Authoring v2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Control/form association contract: `docs/workstreams/control-id-form-association-v1/ALIGNMENT.md`
- Skill guidance: `.agents/skills/fret-shadcn-source-alignment/SKILL.md`

## Scope

This workstream tracks source-aligned migration of shadcn-facing component surfaces toward the
post-v1 Fret defaults:

- typed actions instead of command-string-centric authoring,
- builder-first composition until the final runtime boundary,
- `use_local*` for view-local state by default,
- `control_id(...)` + registry-backed label/description association for form controls,
- parity decisions grounded in upstream shadcn/ui v4 composition and Radix-style interaction outcomes.

## Goals

| Goal | Definition | Primary layer |
| --- | --- | --- |
| Surface parity | The public constructor/builder shape feels close to upstream shadcn for the common path. | `ecosystem/fret-ui-shadcn` |
| Interaction parity | Focus, outside press, keyboard routing, label forwarding, and disabled behavior match intended outcomes. | `fret-ui-kit` + `fret-ui-shadcn` |
| Authoring density | New examples avoid unnecessary `Model<T>`, early `into_element(cx)`, and command glue. | `ecosystem/fret` + `ecosystem/fret-ui-shadcn` |
| Regression closure | Every migrated family has at least one focused test and an audit/workstream note. | docs + tests |

## Layering rules

| If the change is about... | Put it in... | Notes |
| --- | --- | --- |
| Shared action dispatch, payload forwarding, focus forwarding, label/control contracts | `ecosystem/fret-ui-kit` or `crates/fret-ui` | Mechanism/contract, not shadcn-specific recipe code. |
| Public component constructors, snapshot/value entry points, recipe defaults, builder-first composition | `ecosystem/fret-ui-shadcn` | The normal home for shadcn-facing API work. |
| App/demo teaching surface | `apps/fret-cookbook`, `apps/fret-ui-gallery`, `apps/fret-examples` | Use these as evidence, not as the first place to hide missing component APIs. |
| Audit status and rollout sequencing | `docs/workstreams/*`, `docs/audits/*` | Keep implementation evidence linked here. |

## Migration workflow

| Step | What to check | Typical evidence |
| --- | --- | --- |
| 1. Upstream reference | Read the matching upstream `ui/<component>.tsx` and demo/example source. | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*` |
| 2. Public surface | Does Fret expose the same common-path constructor/builder shape? | `from_checked(...)`, `from_pressed(...)`, `build(...)`, `control_id(...)` |
| 3. State placement | Can the common path stay on plain values / `use_local*` instead of forcing `Model<T>`? | cookbook/example migrations |
| 4. Interaction contract | Do label forwarding, focus target choice, disabled semantics, payload dispatch, and keyboard routing match intended outcomes? | focused tests + `control-id` alignment doc |
| 5. Regression gate | Is there at least one narrow unit/integration test or diag script proving the behavior? | `cargo test -p fret-ui-shadcn --lib ...` |
| 6. Audit/doc sync | Update audit note, TODO, and milestone status. | `docs/audits/shadcn-*.md`, this workstream |

## Current family status

Legend:

- `Aligned`: source-aligned enough for the current default path
- `Partial`: common path exists, but parity or teaching surface still needs cleanup
- `Queued`: not yet audited in this workstream

| Family | Fret surface | Upstream shape to preserve | Current status | Evidence anchors | Next move |
| --- | --- | --- | --- | --- | --- |
| Checkbox | `ecosystem/fret-ui-shadcn/src/checkbox.rs` | `checked/defaultChecked`-style discrete control + label/form semantics | Aligned | `docs/audits/shadcn-checkbox.md`, `ecosystem/fret-ui-shadcn/src/checkbox.rs` | Keep stable; only revisit for narrower regressions. |
| Switch | `ecosystem/fret-ui-shadcn/src/switch.rs` | `checked/defaultChecked`-style discrete control + action-first local state | Aligned | `docs/audits/shadcn-switch.md`, `ecosystem/fret-ui-shadcn/src/switch.rs` | Keep stable; use as migration template. |
| Toggle | `ecosystem/fret-ui-shadcn/src/toggle.rs` | `pressed/defaultPressed`-style discrete control | Aligned | `docs/audits/shadcn-toggle.md`, `apps/fret-cookbook/examples/toggle_basics.rs` | Add gallery/diag evidence later if needed. |
| Field / label wrappers | `ecosystem/fret-ui-shadcn/src/field.rs` | Wrapped label content must preserve nested pressable ownership | Aligned | `docs/workstreams/control-id-form-association-v1/ALIGNMENT.md`, `ecosystem/fret-ui-shadcn/src/field.rs` | Keep as shared reference for form-control migrations. |
| Input / Textarea | `ecosystem/fret-ui-shadcn/src/input.rs`, `textarea.rs` | Text value widgets should accept post-v1 local-state path | Partial | `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md` | Continue auditing source-level parity beyond the text-value bridge. |
| Select / Combobox | `ecosystem/fret-ui-shadcn/src/select.rs`, `combobox.rs` | Trigger/listbox semantics, form association, constrained overlay behavior | Partial | `docs/workstreams/control-id-form-association-v1/ALIGNMENT.md` | Audit public surface and overlay interaction parity next. |
| DatePicker / Calendar | `ecosystem/fret-ui-shadcn/src/date_picker.rs`, `calendar.rs` | Trigger-based form control + calendar parity | Partial | `docs/workstreams/control-id-form-association-v1/ALIGNMENT.md` | Recheck control focus target + builder-first content path. |
| Dialog / Drawer / Popover / Tooltip | `ecosystem/fret-ui-shadcn/src/{dialog,drawer,popover,tooltip}.rs` | Root/content/trigger composition parity, overlay interaction rules | Partial | `docs/shadcn-declarative-progress.md` | Use builder-first and overlay parity checklist for each family. |
| DropdownMenu / Menubar / NavigationMenu | `ecosystem/fret-ui-shadcn/src/{dropdown_menu,menubar,navigation_menu}.rs` | Menu semantics + constrained viewport behavior | Queued | `docs/shadcn-declarative-progress.md` | Audit after form-control surfaces are stable. |
| Table / Data-heavy composites | `ecosystem/fret-ui-shadcn/src/table.rs` | Section builder parity + data-row teaching surface | Queued | `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md` | Revisit only if teaching/demo pressure stays high. |

## Recommended component migration template

Use this table when starting a new component family:

| Field | Fill in |
| --- | --- |
| Family | e.g. `Select` |
| Upstream references | relative `repo-ref/ui/...` paths |
| Fret implementation paths | component files to touch |
| Common-path API delta | what still feels more model-centric / eager than upstream |
| Shared-contract delta | focus, label forwarding, payload, dismissal, keyboard, etc. |
| Evidence to add | unit test, diag script, example, audit note |
| Exit condition | what must be true before calling the family aligned |

## Exit criteria for this workstream

| Exit condition | Meaning |
| --- | --- |
| Default-path discrete controls are stable | Checkbox/Switch/Toggle stay aligned without reopening helper sprawl. |
| Form-control association is reusable | New form controls can follow `control_id(...)` + registry + field wrapper patterns directly. |
| Overlay family migration checklist exists | New overlay/component audits can reuse one stable parity checklist. |
| Teaching surfaces stay honest | Cookbook/gallery examples reflect the intended v2 defaults instead of papering over missing APIs. |
