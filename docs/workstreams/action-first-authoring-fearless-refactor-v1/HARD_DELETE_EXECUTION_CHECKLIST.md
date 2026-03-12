# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Execution Checklist

Last updated: 2026-03-09

Related:

- Endgame index: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- Gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Status matrix: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- App-entry policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`
- App-entry inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- App-entry removal playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- Compat-driver inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Compat-driver policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- Compat-driver quarantine playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- Source alignment audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/SOURCE_ALIGNMENT_AUDIT_2026-03-09.md`
- `use_state` inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `use_state` policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `use_state` surface playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- Command-first retained-seam decision: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

This note turns the gap analysis into an execution sequence.

It answers a narrower operational question:

> If the repo wants to finish the remaining cleanup after the action-first/view-runtime migration,
> what should be done next, in what order, and what must be true before each hard delete?

---

## Scope

This checklist covers the remaining post-v1 compatibility surfaces that are still visible to users
or still influence default teaching paths, plus one recently closed lane kept here for historical
context:

1. `fret::App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}`
2. `fret::run_native_with_compat_driver(...)`
3. `ViewCx::use_state::<T>()` as a user-visible alias alongside `use_local*`
4. `CommandId`-first widget contracts that still sit on the default-facing component surface

Non-scope:

- historical MVU archive docs,
- builder/patch `.ui()` on composed widgets,
- `ui::children!` as a composition escape hatch,
- advanced interop surfaces that are intentionally kept and clearly documented as such.

---

## Current snapshot

| Surface | Current status | Default-path risk | Hard-delete readiness |
| --- | --- | --- | --- |
| `App::ui*` closure-root entry | Removed from code pre-release; docs/tests/gates now only teach `view::<V>()` / `view_with_hooks::<V>(...)` | Closed | Closed |
| `run_native_with_compat_driver(...)` | Root entry removed; quarantined under `fret::advanced::interop::run_native_with_compat_driver(...)` | Low for default path, lower facade-size pressure | Deferred after quarantine |
| `use_state::<T>()` alias | Still user-visible; now classified as explicit raw-model hook, not default local state | Low for default path, medium for facade clarity | Deferred |
| `CommandId`-first widget contracts | Action-first aliases landed on the main public builder families, and the curated internal/app-facing menu residue now also prefers `action(...)`; remaining command-shaped usage is now mostly intentional advanced/internal surface area recorded in `COMMAND_FIRST_INTENTIONAL_SURFACES.md` | Medium | Low |

Interpretation:

- The app-entry surface is no longer a blocker; it is now historical evidence for the closed lane.
- The other three items still need explicit product-surface decisions before deletion would be
  defensible.
- Of those three, command-first widgets are no longer a broad implementation pass by default; they
  are now mostly a retained-surface/deprecation-management question, while compat runner and
  `use_state` remain policy-held seams.

---

## Execution order

### Stage 1 — Keep the default path locked while deprecations age

Status: **done (2026-03-09)**

Goal:

- Ensure first-contact surfaces cannot drift back to deprecated authoring paths while the remaining
  cleanup takes time.

Exit evidence:

- `ecosystem/fret/src/app_entry.rs` deprecates `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}`
- `ecosystem/fret/src/lib.rs` and `ecosystem/fret/README.md` teach `view::<V>()` /
  `view_with_hooks::<V>(...)` as the only default path
- `ecosystem/fret/src/lib.rs` test `authoring_surface_policy_tests` locks that wording

Why it comes first:

- Without this gate, later deletions can be undermined by docs/example churn even if the runtime
  direction is already correct.

---

### Stage 2 — Close the app-entry closure lane

Status: **done (2026-03-10)**

Checklist:

| Item | Status | Notes |
| --- | --- | --- |
| In-tree example/demo migration complete | Done | See `APP_ENTRY_CALLER_INVENTORY.md` |
| Public methods removed | Done | `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` no longer exist on `fret::App` |
| Default docs/test gate added | Done | `authoring_surface_policy_tests` |
| Hard-delete regression gate added | Done | `tools/gate_fret_builder_only_surface.py` now forbids the closure-root builder path from returning |
| Final policy recorded | Done | `APP_ENTRY_POLICY_DECISION_DRAFT.md` now records the pre-release hard-delete outcome |
| Historical execution note exists | Done | `APP_ENTRY_REMOVAL_PLAYBOOK.md` records the landed patch shape |

Hard-delete exit criteria:

- no in-tree callers remain,
- docs/tests still teach only `view::<V>()`,
- the old methods are deleted from the public facade,
- lower-level bootstrap/driver seams remain available for intentional advanced integrations.

Recommended next action:

- keep the docs/test gate stable so the closure-root surface does not drift back,
- treat any proposal to restore `App::ui*` as a fresh product decision rather than pending cleanup.

---

### Stage 3 — Quarantine the compat runner and keep capability explicit

Status: **quarantine executed (2026-03-12)**

Recommended decision:

- keep `run_native_with_compat_driver(...)` as an advanced low-level interop seam,
- remove the naked root `fret::run_native_with_compat_driver(...)` entry,
- keep the capability available under `fret::advanced::interop::run_native_with_compat_driver(...)`,
- do not treat hard delete as a near-term target while the caller families still exist.

Checklist:

| Item | Status | Notes |
| --- | --- | --- |
| Inventory current in-tree callers | Done | `COMPAT_DRIVER_CALLER_INVENTORY.md` now records the current family split |
| Decide keep-vs-remove policy | Done | `COMPAT_DRIVER_POLICY_DECISION_DRAFT.md` records “keep capability, quarantine root surface” |
| Execute quarantine behind explicit boundary | Done | surface now lives under `fret::advanced::interop::run_native_with_compat_driver(...)` |
| Keep advanced interop wording aligned | Done | README / crate docs / workstream notes now describe the quarantined seam as non-default advanced low-level interop |
| Hard delete remaining callers | Open | avoid breaking the real retained interop proof points |
| Add docs gate for default-path exclusion | Done | `tools/gate_compat_runner_default_surface.py` now keeps first-contact docs off compat-runner entrypoints while requiring advanced/non-default wording on the quarantined seam |

Current policy exit criteria:

- the runner capability is clearly retained as advanced interop in docs and workstream notes,
- the naked root entry is gone from the main `fret` facade,
- the seam remains outside the default path story,
- future deletion work is deferred until the caller families shrink materially.

Recommended next action:

- keep the quarantined path stable and move attention to the next unresolved hard-delete blocker.
- only reopen deletion if the caller families shrink enough to remove the retained interop need.

---

### Stage 4 — Decide whether `use_state::<T>()` is legacy or permanent shorthand

Status: **decision drafted**

Recommended decision:

- keep `use_state` for now as an explicit raw-model seam,
- do not teach it as the default local-state path,
- defer any deprecation/delete decision until first-contact surfaces stop relying on it.

Checklist:

| Item | Status | Notes |
| --- | --- | --- |
| Inventory remaining teaching/reference usage | Done | `USE_STATE_CALLER_INVENTORY.md` records current starter/reference callers |
| Choose shorthand-vs-legacy policy | Done (draft) | `USE_STATE_POLICY_DECISION_DRAFT.md` recommends “keep now, non-default, reevaluate later” |
| Keep starter/reference snippets off the default path | Done | `hello`, the `hello` template, the gallery action-first snippet, `overlay_basics`, and `imui_action_basics` now use `use_local*` |
| Add a narrow default-path gate | Done | `tools/gate_no_use_state_in_default_teaching_surfaces.py` now guards the approved first-contact/reference files; template output stays covered by `templates.rs` unit assertions |
| If permanent: document semantics clearly | In progress | docs should consistently say “explicit raw-model hook, non-default” |

Current policy exit criteria:

- `use_local*` remains the only default local-state teaching path,
- `use_state` is documented consistently as explicit/non-default,
- future deprecation is deferred until starter/template surfaces no longer rely on it.

Recommended next action:

- keep the first-contact/docs/template gate stable, then revisit whether `use_state` should remain
  permanent or eventually move to deprecation.
- if future facade reduction is chosen, use `USE_STATE_SURFACE_PLAYBOOK.md` to execute the
  explicit-seam-vs-deprecation decision without reopening the default-path migration debate.

---

### Stage 5 — Reduce `CommandId`-first widget surface from default teaching paths

Status: **scoped**

Decision still required:

- Which command-first widget APIs are permanent mechanism-level contracts, and which should gain
  typed-action-first replacements/adapters?

Checklist:

| Item | Status | Notes |
| --- | --- | --- |
| Audit the remaining public command-first widgets | Done | `COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md` now classifies menu/command/navigation/snackbar families |
| Split mechanism-level vs default-facing APIs | In progress | audit recommends keeping catalog/mechanism surfaces command-centric while adding aliases to app-facing widget builders |
| Add typed-action-first adapters where needed | Mostly done | low-risk alias pass plus `ContextMenu*` / `Menubar*` menu-family aliases are now landed |
| Update teaching surfaces | In progress | navigation-menu, context-menu, and menubar gallery snippets prefer `action(...)`; remaining app/internal helper surfaces and focused gates still need follow-through |
| Add focused docs/examples gate | Done | `tools/gate_menu_action_default_surfaces.py` now keeps the primary context-menu / menubar gallery snippets on `action(...)`, and `tools/pre_release.py` runs it in the default policy suite |

Hard-delete exit criteria:

- typed-action-first is the real default story across the default component surface,
- remaining `CommandId`-first APIs are either lower-level mechanism seams or explicitly advanced.

Recommended next action:

- keep the alias surface stable and finish the remaining app/internal helper migration plus the
  default-surface gate strategy before discussing any command-shaped API deprecation.
- treat `COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md` as the current boundary rule: retained
  command-centric seams stay in maintenance mode unless a new default-facing leak or real
  deprecation decision appears.

---

### Stage 6 — Add the remaining narrow gates

Status: **open**

These gates should only be added after the corresponding policy decisions above are explicit.

Checklist:

| Gate | Depends on |
| --- | --- |
| Prevent first-contact docs from reintroducing `App::ui*` as default | Landed and now doubled as a hard-delete regression gate |
| Prevent default docs/templates from reintroducing `use_state` | Landed via `tools/gate_no_use_state_in_default_teaching_surfaces.py` + scaffold template unit assertions |
| Prevent default docs/examples from teaching command-first widget contracts where avoidable | widget-contract policy decision |
| Prevent default docs from drifting toward compat-runner entry points | compat-runner policy wording |

Rule:

- prefer narrow teaching-surface gates over repository-wide code bans when a surface is still
  intentionally supported in advanced code.

---

## Practical completion rule

The cleanup is complete when all four remaining surfaces are in one of only two states:

1. **retained intentionally** and clearly documented as advanced/interop/compat, or
2. **deprecated + gated + removed** from the default-facing facade and teaching paths.

What should not remain:

- a public surface that is neither default nor explicitly advanced,
- a deprecated surface with no removal/quarantine plan,
- a hidden product decision masquerading as “just cleanup”.

---

## Recommended immediate next step

If the repo wants the next move to stay narrow and landable, the next concrete step should be:

1. finish documentation alignment for the compat-runner policy,
2. keep `use_state` on its current non-default/raw-model policy unless the facade policy changes,
3. use the next real implementation pass on remaining command-first widget adoption/gates,
   especially `ContextMenu*` / `Menubar*` internal helper surfaces and default docs/examples,
4. only revisit compat-runner quarantine later if facade-size pressure justifies it.

That order keeps the already-completed app-entry progress from stalling while avoiding premature
deletions in the other three areas.
