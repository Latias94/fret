# Primitives Interaction Semantics Alignment v1 — Audit Matrix

Status: Active (workstream note; not a contract)

This matrix is the **progress tracker** for auditing primitives + shadcn recipes against upstream
interaction semantics (Radix + Base UI), mapped onto Fret’s layering:

- `crates/fret-ui`: mechanism substrate
- `ecosystem/fret-ui-kit`: policy / headless primitives
- `ecosystem/fret-ui-shadcn`: shadcn recipes (composition + styling)

Workstream overview: `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`

---

## Status legend

Each cell uses a single letter:

- `-` not audited
- `M` modeled (explicit outcome/state-machine written down)
- `I` implemented (policy exists in code)
- `G` gated (tests/scripts prevent regressions)

Note for the `Tests` column: use `I` for “some gates exist but incomplete”, and `G` only once the
high-value invariants are hard-gated (unit tests and/or stable diag scripts).

Dimensions (columns) are intentionally outcome-centric (not DOM-centric):

- `Model`: state machine + reasons + invariants written down
- `Policy`: split (Trigger/Listbox/Commit) with explicit knobs
- `Focus`: focus trap/restore + tab order outcomes
- `Dismiss`: escape/outside press/focus out/scroll close semantics
- `Pointer`: misclick guards, click-through vs modal barrier, pointer capture outcomes
- `Keys`: keyboard nav + typeahead outcomes
- `A11y`: semantics roles + active-descendant/roving mapping outcomes
- `Place`: anchored placement/collision/size outcomes
- `Time`: delays/timers are semantic (`Duration`) and reasoned
- `Tests`: regression gates exist (unit/diag/golden as applicable)

---

## Audit matrix (v1 targets + next candidates)

| Component | Baseline | Fret primitive target | shadcn recipe | Model | Policy | Focus | Dismiss | Pointer | Keys | A11y | Place | Time | Tests | Notes |
| --- | --- | --- | --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | --- |
| Select | Radix | `ecosystem/fret-ui-kit/src/primitives/select.rs` | `ecosystem/fret-ui-shadcn/src/select.rs` | M | I | I | I | I | I | I | I | M | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-select.md`. Next: make Base UI anti-misclick knobs explicit without changing defaults. |
| Combobox | Base UI | `ecosystem/fret-ui-kit/src/primitives/combobox.rs` | `ecosystem/fret-ui-shadcn/src/combobox.rs` | M | I | M | I | I | - | M | M | M | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-combobox.md`. Commit and query-close helpers are now policy-owned (`SelectionCommitPolicy`, `ClearQueryOnCloseState`); next: extract highlight/typeahead and reason-aware focus restore. |
| DropdownMenu | Radix | `ecosystem/fret-ui-kit/src/primitives/dropdown_menu.rs` | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | - | - | I | I | I | I | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-dropdown-menu.md`. Diag gate: `tools/diag-scripts/ui-gallery-dropdown-submenu-safe-corridor-sweep.json`. |
| ContextMenu | Radix | `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | - | - | I | I | I | I | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-context-menu.md`. Diag gate: `tools/diag-scripts/ui-gallery-context-menu-overlay-right-click-open-close.json`. |
| Menubar | Radix | `ecosystem/fret-ui-kit/src/primitives/menubar.rs` | `ecosystem/fret-ui-shadcn/src/menubar.rs` | - | - | I | I | I | I | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-menubar.md`. Diag gate: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json`. |
| NavigationMenu | Radix | `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs` | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | - | - | I | I | - | - | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-navigation-menu.md`. Diag gate: `tools/diag-scripts/ui-gallery-navigation-menu-hover-switch-and-escape.json`. |
| Tooltip | Radix | `ecosystem/fret-ui-kit/src/primitives/tooltip.rs` | `ecosystem/fret-ui-shadcn/src/tooltip.rs` | - | - | I | I | I | - | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-tooltip.md`. Diag gate: `tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json`. |
| HoverCard | Radix | `ecosystem/fret-ui-kit/src/primitives/hover_card.rs` | `ecosystem/fret-ui-shadcn/src/hover_card.rs` | - | - | I | I | I | - | - | I | I | I | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-hover-card.md`. Ensure hover intent/delays remain semantic and gated. |
| Popover | Radix | `ecosystem/fret-ui-kit/src/primitives/popover.rs` | `ecosystem/fret-ui-shadcn/src/popover.rs` | - | - | I | I | - | - | - | I | - | I | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-popover.md`. Focus restore + clamp are high-leverage. |
| Dialog | Radix | `ecosystem/fret-ui-kit/src/primitives/dialog.rs` | `ecosystem/fret-ui-shadcn/src/dialog.rs` | - | - | I | I | I | - | - | I | - | I | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-dialog.md`. Gate barrier + escape + restore. |
| AlertDialog | Radix | `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs` | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | - | - | I | I | I | - | - | I | - | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-alert-dialog.md`. Diag gate: `tools/diag-scripts/ui-gallery-alert-dialog-least-destructive-initial-focus.json`. |
| Sheet | Radix (Dialog) | `ecosystem/fret-ui-kit/src/primitives/dialog.rs` | `ecosystem/fret-ui-shadcn/src/sheet.rs` | - | - | I | I | I | - | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-sheet.md`. Diag gate: `tools/diag-scripts/ui-gallery-sheet-escape-focus-restore.json`. |
| Drawer | Vaul-style (Dialog-shaped) | `ecosystem/fret-ui-kit/src/primitives/dialog.rs` | `ecosystem/fret-ui-shadcn/src/drawer.rs` | - | - | I | I | I | - | - | I | I | G | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-drawer.md`. Diag gate: `tools/diag-scripts/ui-gallery-drawer-escape-focus-restore.json`. |
| Toast (Sonner) | Sonner (shadcn) + Radix (store) | `ecosystem/fret-ui-kit/src/primitives/toast.rs` | `ecosystem/fret-ui-shadcn/src/sonner.rs` | - | - | - | I | I | - | - | I | I | I | Audit sheet: `docs/workstreams/primitives-interaction-semantics-alignment-v1-toast.md`. Diag: `tools/diag-scripts/ui-gallery-toast-visible.json`. |

---

## Upstream references (local pinned)

- Select (Radix baseline): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Combobox (Base UI baseline): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/combobox.tsx`
- Sheet (Dialog baseline): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`
- Drawer (Vaul baseline): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx`
- Radix primitives sources: `repo-ref/primitives/packages/react/*`
- Base UI sources: `repo-ref/base-ui/packages/*`
