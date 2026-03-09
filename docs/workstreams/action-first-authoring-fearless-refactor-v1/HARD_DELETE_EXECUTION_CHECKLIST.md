# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Execution Checklist

Last updated: 2026-03-09

Related:

- Gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Status matrix: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- App-entry policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`
- App-entry inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- App-entry removal playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- Compat-driver inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Compat-driver policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- Compat-driver quarantine playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `use_state` inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `use_state` policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `use_state` surface playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

This note turns the gap analysis into an execution sequence.

It answers a narrower operational question:

> If the repo wants to finish the remaining cleanup after the action-first/view-runtime migration,
> what should be done next, in what order, and what must be true before each hard delete?

---

## Scope

This checklist only covers the remaining post-v1 compatibility surfaces that are still visible to
users or still influence default teaching paths:

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
| `App::ui*` closure-root entry | Deprecated in code; no in-tree example/demo callers remain; minimum deprecation window is now defined | Low once docs/tests stay locked | Medium |
| `run_native_with_compat_driver(...)` | Still public; now explicitly classified as advanced low-level interop | Low for default path, medium for facade size | Deferred |
| `use_state::<T>()` alias | Still user-visible; now classified as explicit raw-model hook, not default local state | Low for default path, medium for facade clarity | Deferred |
| `CommandId`-first widget contracts | Action-first aliases landed on the main public builder families, and the curated internal/app-facing menu residue now also prefers `action(...)`; remaining command-shaped usage is now mostly intentional advanced/internal surface area recorded in `COMMAND_FIRST_INTENTIONAL_SURFACES.md` | Medium | Low |

Interpretation:

- The app-entry surface is the closest to final cleanup.
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

### Stage 2 — Finish the app-entry closure removal plan

Status: **in progress**

Decision still required:

- final end-state is either:
  1. hard delete from `fret`, or
  2. move behind an explicit compat boundary that is intentionally not the default facade path.

Checklist:

| Item | Status | Notes |
| --- | --- | --- |
| In-tree example/demo migration complete | Done | See `APP_ENTRY_CALLER_INVENTORY.md` |
| Public methods deprecated | Done | `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` |
| Default docs/test gate added | Done | `authoring_surface_policy_tests` |
| Downstream deprecation window defined | Done | start 2026-03-09; earliest removal 2026-06-09; require one published deprecated release; see `APP_ENTRY_POLICY_DECISION_DRAFT.md` |
| Final compat-vs-delete decision | Open | product-surface decision, not just cleanup |
| Final removal/quarantine patch | Open | only after the two items above |

Hard-delete exit criteria:

- the deprecation window has elapsed (not before 2026-06-09),
- no in-tree callers remain,
- docs/tests still teach only `view::<V>()`,
- at least one published release has shipped with the deprecation warnings,
- either:
  - the old methods are deleted, or
  - they move to an explicitly named compat surface that is absent from default docs/templates.

Recommended next action:

- keep the docs/test gate stable and wait until the first published deprecated release has shipped
  before proposing the removal/quarantine patch.
- once the window and release preconditions are satisfied, use
  `APP_ENTRY_REMOVAL_PLAYBOOK.md` to execute the final delete-vs-quarantine patch.

---

### Stage 3 — Decide the compat-runner fate

Status: **decision drafted**

Recommended decision:

- keep `run_native_with_compat_driver(...)` for now as an advanced low-level interop seam,
- do not treat it as a near-term hard-delete target,
- if future surface reduction is desired, quarantine it behind a clearer compat/interop boundary
  before considering removal.

Checklist:

| Item | Status | Notes |
| --- | --- | --- |
| Inventory current in-tree callers | Done | `COMPAT_DRIVER_CALLER_INVENTORY.md` now records the current family split |
| Decide keep-vs-remove policy | Done (draft) | `COMPAT_DRIVER_POLICY_DECISION_DRAFT.md` recommends “keep now, reevaluate later” |
| If keeping: document as advanced interop | Done | README / crate docs / workstream notes now describe the surface as non-default advanced low-level interop |
| If removing: migrate/relocate remaining demos first | Open | avoid breaking the only real interop proof points |
| Add docs gate for default-path exclusion | Open | narrow gate, not a global ban |

Current policy exit criteria:

- the runner surface is clearly retained as advanced interop in docs and workstream notes,
- it remains outside the default path story,
- future deletion work is deferred until the caller families shrink or a quarantine boundary exists.

Recommended next action:

- keep the wording stable and move attention to the next unresolved hard-delete blocker rather than
  forcing a premature runner deletion.
- if the repo later chooses facade reduction, use `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` to move
  the seam behind an explicit compat boundary before reopening deletion.

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

---

### Stage 6 — Add the remaining narrow gates

Status: **open**

These gates should only be added after the corresponding policy decisions above are explicit.

Checklist:

| Gate | Depends on |
| --- | --- |
| Prevent first-contact docs from reintroducing `App::ui*` as default | already landed |
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

1. keep the `App::ui*` deprecation window visible and enforced in docs,
2. finish documentation alignment for the compat-runner policy,
3. keep `use_state` on its current non-default/raw-model policy unless the facade policy changes,
4. use the next real implementation pass on remaining command-first widget adoption/gates,
   especially `ContextMenu*` / `Menubar*` internal helper surfaces and default docs/examples,
5. only revisit compat-runner quarantine later if facade-size pressure justifies it.

That order keeps the already-completed app-entry progress from stalling while avoiding premature
deletions in the other three areas.
