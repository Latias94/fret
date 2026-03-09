# Action-First Authoring + View Runtime (Fearless Refactor v1) — Command-First Widget Contract Audit

Status: draft, post-v1 audit
Last updated: 2026-03-09

Related:

- Hard-delete gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

---

## Scope

This note audits the remaining public widget/component surfaces that still read as
`CommandId`-first even after the action-first/view-runtime migration.

The narrow question is:

> Which `CommandId`-centric contracts are still justified as mechanism-level seams, and which ones
> should gain action-first aliases/adapters so the default authoring path stays coherent?

This is not a proposal to remove `CommandId` from the runtime.
The runtime decision from ADR 0307 still holds for v1:

- `ActionId` remains compatible with `CommandId`,
- keymap / menu / command-palette routing still lower to the command pipeline,
- the cleanup pressure is now on the **public authoring surface**, not on the routing mechanism.

---

## Summary

Current conclusion:

1. **Not all remaining `CommandId`-first APIs are equal.**
   - Some are genuinely registry/catalog-driven and should stay command-centric internally.
   - Some are now default-facing authoring surfaces and should expose action-first aliases.
2. **The main blocker is menu-family item APIs** (`ContextMenu*`, `Menubar*`, parts of
   `NavigationMenu`, and a few smaller action-bearing recipes).
3. **The repo should not try to delete `CommandId` from these widgets immediately.**
   - The safer move is a split:
     - keep command-centric storage/dispatch internally,
     - add action-first aliases on default-facing builder methods,
     - keep explicit command naming where metadata/shortcut display/catalog integration is the point.
4. **`Button` and `CommandItem` already show the intended pattern.**
   - `Button` has both `action(...)` and legacy `on_click(...)`.
   - `CommandItem` already has both `on_select(CommandId)` and `on_select_action(...)`.
   - That dual-surface pattern is the likely migration template for the remaining widget families.

---

## Audit matrix

| Surface / family | Current public shape | Why `CommandId` exists here today | Audit status | Recommended direction |
| --- | --- | --- | --- | --- |
| `Button` | `action(...)`, `action_payload(...)`, legacy `on_click(CommandId)` | generic pressable dispatch; default authoring already converged | Aligned | Keep current dual surface; treat `on_click(...)` as compat naming only |
| `CommandItem` / command palette entries | `on_select(CommandId)` plus `on_select_action(...)` / `on_select_value_action(...)`; host-command catalog builders derive from `CommandMeta` | palette items naturally map to registry metadata, shortcut display, and gating snapshots | Mostly aligned | Keep command-first catalog APIs; keep action-first callback path for custom items |
| `DropdownMenuItem` / `DropdownMenuCheckboxItem` / `DropdownMenuRadioItem` | `on_select(CommandId)` only (plus `trailing_on_select(...)` on item rows) | menu rows dispatch through command gating and may expose trailing command affordances | Default-facing blocker | Add `action(...)` / `trailing_action(...)` aliases first; keep command-centric internals and trailing command pipeline intact |
| `ContextMenuItem` / `ContextMenuCheckboxItem` / `ContextMenuRadioItem` | `on_select(CommandId)` only | menu rows display shortcut labels and dispatch through command gating | Default-facing blocker | Add `action(...)` alias first; keep command-centric internals and shortcut display logic |
| `MenubarItem` / `MenubarCheckboxItem` / `MenubarRadioItem` | `on_select(CommandId)` only | same as context menu, plus stronger OS/menu parity expectations | Default-facing blocker | Same as context menu: action-first alias on public builders, command-centric core retained |
| `NavigationMenuLink` / `NavigationMenuItem` | `on_click(CommandId)` only | current activation path reuses command gating/dispatch | Medium blocker | Add `action(...)` alias; this surface reads like a regular app-facing widget, not a registry API |
| `BreadcrumbItem` | `on_click(CommandId)` plus `on_activate(...)` closure | historical command pipeline reuse | Medium blocker | Add `action(...)` alias; keep `on_click(...)` as compat name |
| Material `Snackbar` | `action(...)` plus `action_id(...)` / `action_command(...)` aliases | toast action currently lowers to command dispatch | Aligned (dual surface) | Prefer `action_id(...)` in default-facing docs/examples; keep `action(...)` / `action_command(...)` as compat/low-level spellings |

---

## Findings

### 1) The real blocker is not the command pipeline; it is command-shaped builder naming

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/button.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`
- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- `ecosystem/fret-ui-material3/src/snackbar.rs`

What changed after v1:

- the repo already teaches typed actions by default,
- `ActionId == CommandId` is an accepted v1 compatibility rule,
- the user-visible friction is therefore no longer runtime capability; it is that some widget APIs
  still make authors spell the old command-centric naming even when they are clearly using
  action-first authoring.

Practical implication:

- In many cases, adding an action-first alias is enough to align the mental model without changing
  storage, dispatch, diagnostics, or keymap integration.

### 2) `Button` and `CommandItem` already provide the template

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/button.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`

Current shape:

- `Button` exposes `action(...)` as the default-facing name while keeping `on_click(CommandId)` as
  a legacy/compat path.
- `CommandItem` exposes:
  - `on_select(CommandId)` for registry-like command dispatch,
  - `on_select_action(...)` for explicit callback-based items,
  - `on_select_value_action(...)` for cmdk-style value callbacks.

Assessment:

- This is already the correct split for the rest of the audit:
  - **catalog / shortcut / gating-driven surfaces** may keep command-centric shapes,
  - **default-facing app-author widgets** should expose action-first naming even if they still lower
    into the same command dispatch path.

### 3) Menu families are the largest remaining default-surface inconsistency

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`

Current shape:

- item builders still expose only `on_select(CommandId)`,
- checkbox/radio variants also expose only `on_select(CommandId)`,
- rendering logic uses command availability and shortcut display helpers.

Assessment:

- The internals are still legitimately command-centric because:
  - menu rows frequently need shortcut labels,
  - menu gating aligns with command metadata,
  - OS/menu integration continues to depend on stable command identities.
- But the public builder naming is now behind the repo’s default authoring story.

Recommendation:

- Keep internals unchanged for now.
- Add public aliases such as:
  - `action(...)` for `Item`,
  - `action(...)` for checkbox/radio variants when the action is the semantic trigger,
  - possibly payload-aware aliases later only if payload actions become part of menu-family policy.
- Do **not** remove the explicit command-oriented method yet; keep it as the lower-level/legacy
  spelling.

### 4) `NavigationMenu` and `Breadcrumb` are more app-facing than registry-facing

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`

Current shape:

- these surfaces expose `on_click(CommandId)` while also supporting direct activation callbacks or
  app-facing link semantics.

Assessment:

- These widgets are less tightly coupled to shortcut display or command metadata catalogs than menu
  entries are.
- From the author’s perspective they behave more like buttons/links than registry-driven command
  surfaces.

Recommendation:

- Add `action(...)` as the preferred public alias.
- Keep `on_click(CommandId)` only as compat naming.
- Treat these as good early migration candidates because they have lower product-surface risk than
  the full menu family.

### 5) Material `Snackbar` is small but visibly off the v2 mental model

Representative evidence:

- `ecosystem/fret-ui-material3/src/snackbar.rs`

Current shape:

- `Snackbar::action(label, CommandId)` still exists as the historical spelling,
- `Snackbar::action_id(label, action)` and `Snackbar::action_command(label, command)` now provide
  the explicit action-first / low-level split on top of the same toast dispatch path.

Assessment:

- This is not a deep runtime blocker.
- It is a naming/product-surface mismatch: app authors see a high-level widget API but still have
  to supply a raw `CommandId`.

Recommendation:

- Keep `action_id(...)` as the default public spelling in docs/examples.
- Keep `action_command(...)` and the historical `action(...)` spelling as additive compatibility
  surfaces until the repo decides whether this family should deprecate command-shaped naming.

### 6) One documentation note is now stale relative to the action-first surface

Representative evidence:

- `docs/component-author-guide.md`

Current wording still says:

- “Commands + shortcuts: always go through `CommandId` + keymap”

Assessment:

- That guidance still makes sense for runtime identity, keymap, and menu metadata.
- It is too strong for the current public authoring story because the repo now intentionally
  teaches typed actions first and only lowers to command IDs at the routing boundary.

Recommendation:

- Update this guide in a later follow-up so it distinguishes:
  - runtime identity / menu-keymap integration,
  - versus default public builder naming on ecosystem widgets.

---

## Recommended next steps

### Phase 1 — Low-risk alias alignment

Start with the smallest product-surface risk:

1. `BreadcrumbItem`
2. `NavigationMenuLink` / `NavigationMenuItem`
3. Material `Snackbar`

Goal:

- prove the alias pattern on clearly app-facing widgets before touching the heavier menu stack.

Progress update (as of 2026-03-09):

- `BreadcrumbItem::action(...)` now exists in `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`.
- `NavigationMenuLink::action(...)` and `NavigationMenuItem::action(...)` now exist in
  `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`.
- The navigation-menu gallery snippets (`demo.rs`, `docs_demo.rs`, `rtl.rs`) now prefer
  `action(...)` as the default public spelling.
- Material `Snackbar::action_id(...)` and `Snackbar::action_command(...)` now exist in
  `ecosystem/fret-ui-material3/src/snackbar.rs`.
- The Material3 gallery snackbar snippet now prefers `action_id(...)`:
  `apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs`.
- Material3 regression coverage also uses the alias in
  `ecosystem/fret-ui-material3/tests/radio_alignment.rs`.
- A narrow default-surface gate now protects that choice:
  `tools/gate_material3_snackbar_default_surface.py`.

### Phase 2 — Menu-family public alias pass

Then add action-first aliases to:

1. `DropdownMenuItem` family
2. `ContextMenuItem` family
3. `MenubarItem` family

Guardrail:

- keep shortcut display, trailing command affordances, gating, and command metadata integration unchanged internally.

Progress update (as of 2026-03-09, follow-up):

- `DropdownMenuItem::action(...)` / `trailing_action(...)`,
  `DropdownMenuCheckboxItem::action(...)`,
  `DropdownMenuRadioItemSpec::action(...)`, and
  `DropdownMenuRadioItem::action(...)` now exist in
  `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`.
- The main dropdown-menu gallery teaching snippets now also prefer `action(...)`:
  - `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/demo.rs`
- The gallery overlay preview surfaces now also prefer `action(...)` for dropdown/context menu rows:
  - `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/menus.rs`
  - `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay/widgets.rs`
- `tools/gate_menu_action_default_surfaces.py` now covers dropdown-menu snippets in addition to
  context-menu / menubar snippets, plus the two overlay preview teaching surfaces above.
- A final curated-internal follow-up also moved the remaining obvious app/internal dropdown menu
  residue onto the same spelling:
  - `ecosystem/fret-workspace/src/tab_strip/overflow.rs`
  - `ecosystem/fret-genui-shadcn/src/resolver/overlay.rs`
- `tools/gate_menu_action_curated_internal_surfaces.py` now protects those two intentionally
  chosen internal/app-facing surfaces from drifting back to `.on_select(...)` /
  `trailing_on_select(...)`.

Previous phase-2 progress:

- `ContextMenuItem::action(...)`, `ContextMenuCheckboxItem::action(...)`,
  `ContextMenuRadioItemSpec::action(...)`, and `ContextMenuRadioItem::action(...)` now exist in
  `ecosystem/fret-ui-shadcn/src/context_menu.rs`.
- `MenubarItem::action(...)`, `MenubarCheckboxItem::action(...)`,
  `MenubarRadioItemSpec::action(...)`, and `MenubarRadioItem::action(...)` now exist in
  `ecosystem/fret-ui-shadcn/src/menubar.rs`.
- The gallery default snippets now also prefer the alias across the main menu/reference surface:
  - Context menu:
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/usage.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/destructive.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/groups.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/icons.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/checkboxes.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/radio.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/shortcuts.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/sides.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/submenu.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/context_menu/rtl.rs`.
  - Menubar:
    `apps/fret-ui-gallery/src/ui/snippets/menubar/demo.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/usage.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/radio.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/checkbox.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/parts.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/submenu.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/rtl.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/menubar/with_icons.rs`.
- The command-centric storage and routing model remains unchanged; this phase is still strictly a
  public builder-surface alignment pass.

### Phase 3 — Decide long-term naming/deprecation

After aliases land in the main default-facing families:

- decide whether command-shaped names become:
  - permanent low-level spellings,
  - or deprecated compat aliases.

That decision should be made only after:

- gallery/cookbook/reference surfaces demonstrate the new spelling,
- docs are updated,
- and there is no ambiguity about which naming is the default path.

---

## Practical verdict

The remaining `CommandId` blocker is now well-scoped:

- **keep command-centric routing internals,**
- **add action-first naming on default-facing widgets,**
- **do not attempt a runtime-level command removal.**

This means the next work is mostly surface cleanup and naming convergence, not another deep
architecture change.
