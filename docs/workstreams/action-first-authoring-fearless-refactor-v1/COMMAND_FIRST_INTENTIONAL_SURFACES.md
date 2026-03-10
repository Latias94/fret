# Action-First Authoring + View Runtime (Fearless Refactor v1) — Command-First Intentional Surfaces

Status: draft, post-v1 inventory
Last updated: 2026-03-09

Related:

- Command-first widget contract audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- Hard-delete status matrix: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- DataTable authoring audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_AUTHORING_AUDIT.md`

---

## Purpose

This note records the remaining command-shaped surfaces that the repo should currently treat as
**intentional retained seams**, not as the next generic migration target.

The narrow question is:

> After the public alias pass and curated internal follow-up, which remaining command-first usages
> are still justified enough that we should stop chasing them as residue?

Current conclusion:

- the default-facing widget families are now aligned enough for v1,
- the remaining command-shaped usages mostly belong to advanced/catalog/test surfaces,
- future cleanup should focus on **new default-facing leaks only**, not on broad internal churn.

---

## Inventory matrix

| Surface family | Representative evidence | Why it still looks command-shaped | Current decision | Future trigger for change |
| --- | --- | --- | --- | --- |
| Command palette / command catalog | `ecosystem/fret-ui-shadcn/src/command.rs:121`, `ecosystem/fret-ui-shadcn/src/command.rs:166` | These rows are built from `CommandMeta`, keymap display, and gating snapshots; command identity is the product surface | Keep command-centric by default | Only revisit if the repo later splits catalog metadata from command routing in a deeper v2 |
| Business-table menu/action wiring | `ecosystem/fret-ui-shadcn/src/data_table.rs:268`, `apps/fret-ui-gallery/src/ui/snippets/data_table/basic_demo.rs:498`, `apps/fret-ui-gallery/src/ui/snippets/data_table/guide_demo.rs:510` | `DataTable` is an advanced integration surface; row/column actions, output wiring, and state ownership are intentionally explicit | Keep as advanced/reference surface; do not fold into generic action-sugar cleanup | Only revisit if a curated higher-level business-table recipe still proves too noisy after docs/productization |
| Compat / conformance tests for menu families | `ecosystem/fret-ui-shadcn/tests/menubar_keyboard_navigation.rs:95`, `ecosystem/fret-ui-shadcn/tests/context_menu_keyboard_navigation.rs:100`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs:4758` | Tests must cover legacy/compat spellings and low-level contracts directly; they are not teaching surfaces | Keep as-is unless a specific test becomes misleading | Update only when compat APIs are actually deprecated/removed |
| Non-menu callback widgets that expose `on_select(...)` closures | `apps/fret-ui-gallery/src/ui/snippets/ai/file_tree_demo.rs:123`, `apps/fret-ui-gallery/src/ui/snippets/ai/file_tree_large.rs:80`, `ecosystem/fret-ui-material3/src/exposed_dropdown.rs:288` | These are callback/event APIs, not `CommandId`-first builder spelling; they are outside the menu alias cleanup track | Explicitly out of scope for command-first residue work | Revisit only in their own domain audits, not under menu/action alias cleanup |

---

## Practical reading

### 1. Command palette should stay command-centric

The repo already decided that:

- `ActionId` is `CommandId`-compatible in v1,
- but command palette rows still fundamentally represent **catalogued commands**.

That means `CommandItem::on_select(CommandId)` is not a migration failure.
It is the right fit for:

- metadata-driven listing,
- shortcut display,
- command gating,
- and registry-backed command discovery.

### 2. DataTable should not be used as “more residue”

The remaining command-shaped `DataTable` examples are not evidence that menu aliases are unfinished.
They are evidence that business-table authoring is a different problem:

- explicit row/column action wiring,
- explicit state/output models,
- explicit page/selection coordination.

That surface should stay documented as advanced/reference until a deliberate product recipe says
otherwise.

### 3. Tests are allowed to stay blunt

The repo should not spend cleanup budget rewriting:

- compat tests,
- conformance tests,
- or widget self-tests

just to make them read like the default authoring path.

Their job is to prove behavior and contract coverage, including old spellings where relevant.

### 4. Some `on_select(...)` surfaces are simply a different API family

A callback-style domain widget such as AI file tree or Material autocomplete/dropdown:

- does not teach `CommandId`-first naming,
- does not belong to the menu-family alias pass,
- and should not be swept into this cleanup bucket just because the method name happens to match.

---

## Decision

For the current workstream:

1. Treat the public menu-family alias pass as effectively complete for v1.
2. Treat the remaining command-shaped surfaces above as intentional retained seams.
3. Do **not** schedule another broad migration pass unless a new default-facing leak appears.
4. Keep future cleanup narrow:
   - default docs/examples,
   - curated app-facing helpers,
   - or actual deprecation/removal of compat APIs.

That means the practical next step is no longer “find more `.on_select(...)` and replace it”.
It is:

- hold the current default path stable,
- document the intentional retained surfaces clearly,
- and only reopen this track when product/default-path evidence changes.
