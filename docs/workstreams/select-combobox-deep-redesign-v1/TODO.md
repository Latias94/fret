## Select + Combobox Deep Redesign v1 (TODO + Tracker)

Last updated: 2026-03-02.

This tracker is **workstream-local**. It exists because `select` and `combobox` need deeper
structural work than the “part surface alignment” stream.

### Reference anchors (upstream)

- `select` base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx`
- `combobox` base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`

### Existing substrate in `fret-ui-kit` (do not duplicate)

Before extracting anything new, prefer reusing the existing primitives that already encode the
reference-stack outcomes (Radix-ish select + Base UI-ish combobox):

| Area | Existing module | Why it matters |
|---|---|---|
| Select (Radix outcomes) | `ecosystem/fret-ui-kit/src/primitives/select.rs` | Already encodes trigger a11y, open/close modeling, pointer-up guards, typeahead while closed/open, item-aligned placement helpers, and modal barrier/layer helpers. |
| Combobox (Base UI outcomes) | `ecosystem/fret-ui-kit/src/primitives/combobox.rs` | Encodes open-change reasons, reason-aware focus restore policies, open-change callback gating, and selection commit helpers (single + multi). |
| Active descendant | `ecosystem/fret-ui-kit/src/primitives/active_descendant.rs` | Provides the “active descendant” automation/a11y surface and gates for “present vs missing active option” stability. |
| Typeahead policy | `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs` + `ecosystem/fret-ui-kit/src/headless/mod.rs` (`typeahead`) | Prefix-buffer typeahead policies exist and are reused across menu/table surfaces; select/combobox should use the same buffer semantics where possible. |

### M0 — Current surface mapping (upstream → Fret)

This is the “copy/paste authoring” view: do we have the same identifiers available, and do they
mean roughly the same thing.

#### Select exports (v4)

| Upstream export (TS) | Upstream base file | Fret symbol (Rust) | Fret module | Notes / drift |
|---|---|---|---|---|
| `Select` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx` | `Select` | `ecosystem/fret-ui-shadcn/src/select.rs` | In Fret this is a recipe/builder over models (`value`, `open`) and an `entries` list. |
| `SelectValue` | same | `SelectValue` | same | In upstream this is a literal nested part; in Fret it is a configuration part used by the recipe. |
| `SelectTrigger` | same | `SelectTrigger` | same | In upstream this is a literal trigger element; in Fret it is a configuration part (size, label policy, test id, etc.). |
| `SelectContent` | same | `SelectContent` | same | In upstream this includes `Portal` + `Viewport`; in Fret it is configuration (side/align/offset/scroll buttons). |
| `SelectGroup` | same | `SelectGroup` | same | Part exists and is used as an entry container. |
| `SelectLabel` | same | `SelectLabel` | same | Part exists and is used as an entry label. |
| `SelectItem` | same | `SelectItem` (incl `SelectItemText`, `SelectItemIndicator`) | same | Upstream `SelectItem` renders `ItemText`/`ItemIndicator` slots; Fret models these as explicit parts/config. |
| `SelectSeparator` | same | `SelectSeparator` | same | Part exists as an entry separator. |
| `SelectScrollUpButton` | same | `SelectScrollUpButton` | same | Part exists; defaults differ based on `scroll_buttons` policy. |
| `SelectScrollDownButton` | same | `SelectScrollDownButton` | same | Part exists; defaults differ based on `scroll_buttons` policy. |

#### Combobox exports (v4, Base UI)

| Upstream export (TS) | Upstream base file | Fret symbol (Rust) | Fret module | Notes / drift |
|---|---|---|---|---|
| `Combobox` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx` | `Combobox` | `ecosystem/fret-ui-shadcn/src/combobox.rs` | Upstream is `@base-ui/react` render-prop driven; Fret is a recipe/adaptor (currently Popover + Command shaped). |
| `ComboboxValue` | same | `ComboboxValue` | same | In upstream this is a nested part; in Fret it is primarily used to drive chips rendering. |
| `ComboboxTrigger` | same | `ComboboxTrigger` | same | Upstream is a “trigger” element; in Fret it maps to recipe-level knobs (`variant`, `width`). |
| `ComboboxClear` | same | `ComboboxClear` | same | Upstream is a nested part with a `render` prop; in Fret it enables the clear affordance. |
| `ComboboxInput` | same | `ComboboxInput` | same | Upstream is `InputGroup`-composed and supports `showTrigger/showClear`; Fret mirrors these knobs. |
| `ComboboxContent` | same | `ComboboxContent` | same | Upstream includes `Portal + Positioner + Popup`; Fret maps to overlay placement policy (`side/align/offset`) + content parts. |
| `ComboboxList` | same | `ComboboxList` | same | Part exists; should remain the scroll container for items. |
| `ComboboxItem` | same | `ComboboxItem` | same | Part exists; indicator is explicit in Rust. |
| `ComboboxGroup` | same | `ComboboxGroup` | same | Part exists. |
| `ComboboxLabel` | same | `ComboboxLabel` | same | Part exists. |
| `ComboboxCollection` | same | `ComboboxCollection` | same | Upstream uses it for list virtualization/collection metadata; in Fret it is an adapter surface only today. |
| `ComboboxEmpty` | same | `ComboboxEmpty` | same | Part exists. |
| `ComboboxSeparator` | same | `ComboboxSeparator` | same | Part exists. |
| `ComboboxChips` | same | `ComboboxChips` | same | Part exists in Fret as an adapter; chips rendering is recipe-owned today. |
| `ComboboxChip` | same | `ComboboxChip` | same | Part exists; removal affordance is recipe-owned today. |
| `ComboboxChipsInput` | same | `ComboboxChipsInput` | same | Part exists. |
| `useComboboxAnchor` | same | `use_combobox_anchor` | same | Upstream returns a DOM ref; Fret returns a layout-only `PopoverAnchor` wrapper exposing a stable element id. |

#### Fret-only helper exports (migration ergonomics)

These do not exist upstream but reduce churn when porting examples:

- (None today.) Prefer `ComboboxItem::new(...)` and `ComboboxGroup::new().items(...)` for explicit
  list construction.

### Status legend

- `Not started`
- `In progress`
- `Done (with known gaps)`
- `Done`
- `Deferred (planned)`

### Tracker table

| Component | Target surface | Current state (Fret) | Known gaps / risks | Proposed changes (layer) | Gates | Status |
|---|---|---|---|---|---|---|
| `select` | shadcn part surface (`Select*`) + stable `test_id` | Part adapters exist (`Select::into_element_parts` + `SelectContent::with_entries`); call-site parity is mostly achieved in-tree | Composition drift, focus/keyboard edge cases, automation surfaces not uniformly documented | Extract/align shared listbox substrate (`kit`), keep shadcn defaults (`shadcn`); decide a single focus model and gate it | Unit tests: `ecosystem/fret-ui-shadcn/tests/select_test_id_stability.rs` (locks trigger + viewport `test_id`), `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs` (ArrowDown + Enter selects + closes), `ecosystem/fret-ui-shadcn/tests/select_escape_dismiss_focus_restore.rs` (Escape closes + focus restore), `ecosystem/fret-ui-shadcn/tests/select_typeahead.rs` (KeyB typeahead selects matching item) | Done (with known gaps) |
| `combobox` | shadcn part surface (`Combobox*`) + docs-aligned examples | Part adapters exist; legacy option model has been removed; known structural drift remains by design | Input-in-trigger ergonomics, Base UI-style expectations, structural adapter debt | Shared substrate (`kit`), refine part surface + adapters (`shadcn`), document explicit differences | Unit tests: `ecosystem/fret-ui-shadcn/tests/combobox_test_id_prefix_semantics.rs` (locks prefix scheme + item slugging), `ecosystem/fret-ui-shadcn/tests/combobox_keyboard_navigation.rs` (ArrowDown + Enter selects + closes), `ecosystem/fret-ui-shadcn/tests/combobox_escape_dismiss_focus_restore.rs` (Escape closes + focus restore), `ecosystem/fret-ui-shadcn/tests/combobox_filtering.rs` (TextInput filters + Enter selects), `ecosystem/fret-ui-shadcn/src/combobox_chips.rs` (chips adapter part patch gates) | In progress |

### Proposed `test_id` scheme (for gates + scripted diags)

We should commit to a stable `test_id` naming scheme before deep refactors, so we can gate behavior
without binding tests to internal structure.

#### Select

Existing in-tree anchors (already used by tests):

- Trigger: call-site provided (e.g. `Select::trigger_test_id("select-trigger")`)
- Viewport / scroll container: `select-scroll-viewport` (wired in `ecosystem/fret-ui-shadcn/src/select.rs`)

Recommended conventions for new gates:

- Trigger chrome wrapper (if needed): `select-trigger.chrome`
- Items: `select.item.<value>` (value should be stable and URL/identifier-safe)
- Group labels: `select.label.<heading>` (optional)
- Scroll buttons: `select.scroll_up`, `select.scroll_down`

#### Combobox

Combobox already supports a stable prefix-based scheme (`Combobox::test_id_prefix(...)`), and the
parts adapter forwards it to:

- Trigger: `<prefix>-trigger` / `<prefix>-trigger-icon` / `<prefix>-clear-button`
- Input: `<prefix>-input`
- Listbox: `<prefix>-listbox`
- Items: `<prefix>-item-<slug(value)>`

For multi-select (chips), prefer explicit `test_id`s on chip parts where possible.

### Immediate next steps (proposed)

1. **Decide focus model** per component:
   - active-descendant listbox model vs roving focus on items,
   - how it maps to platform semantics and automation.
2. **Lock the `test_id` scheme with one gate per component** (done):
   - `select`: `ecosystem/fret-ui-shadcn/tests/select_test_id_stability.rs`
   - `combobox`: `ecosystem/fret-ui-shadcn/tests/combobox_test_id_prefix_semantics.rs`
3. **Extract one shared helper in `ecosystem/fret-ui-kit` (reuse-first)**:
   - only if we can’t express the desired outcome with existing `kit` primitives,
   - add a unit test at the kit layer for the helper (not only recipe tests).

### Evidence checklist (fill as we implement)

For each milestone, record 1–3 evidence anchors:

- file + symbol anchors (e.g. `ecosystem/fret-ui-kit/src/...`),
- unit tests (`cargo test -p fret-ui-shadcn --lib <filter>`),
- diag scripts (if used).

#### Evidence anchors (today)

These anchors are intended to make audits and future refactors cheaper (searchable, stable names).

- Upstream sources:
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`
- Kit substrate:
  - `ecosystem/fret-ui-kit/src/primitives/select.rs:89` (`SelectRoot`)
  - `ecosystem/fret-ui-kit/src/primitives/select.rs:147` (`apply_select_trigger_a11y`)
  - `ecosystem/fret-ui-kit/src/primitives/select.rs:302` (`SelectMousePolicies`)
  - `ecosystem/fret-ui-kit/src/primitives/select.rs:1822` (`modal_select_request`)
  - `ecosystem/fret-ui-kit/src/primitives/combobox.rs:18` (`ComboboxOpenChangeReason`)
  - `ecosystem/fret-ui-kit/src/primitives/combobox.rs:48` (`ComboboxCloseAutoFocusPolicy`)
  - `ecosystem/fret-ui-kit/src/primitives/combobox.rs:213` (`commit_selection_on_activate`)
  - `ecosystem/fret-ui-kit/src/primitives/combobox.rs:280` (`on_close_auto_focus_with_reason`)
  - `ecosystem/fret-ui-kit/src/primitives/active_descendant.rs:4` (`active_descendant_for_index` re-export)
- Current shadcn recipes/adapters:
  - `ecosystem/fret-ui-shadcn/src/select.rs:1268` (`Select`)
  - `ecosystem/fret-ui-shadcn/src/select.rs:1484` (`Select::into_element_parts`)
- `ecosystem/fret-ui-shadcn/src/combobox.rs:108` (`use_combobox_anchor`)
  - `ecosystem/fret-ui-shadcn/src/combobox.rs:793` (`Combobox`)
  - `ecosystem/fret-ui-shadcn/src/combobox.rs:888` (`Combobox::into_element_parts`)
- Gates:
  - `ecosystem/fret-ui-shadcn/tests/select_test_id_stability.rs`
  - `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`
  - `ecosystem/fret-ui-shadcn/tests/select_escape_dismiss_focus_restore.rs`
  - `ecosystem/fret-ui-shadcn/tests/select_typeahead.rs`
  - `ecosystem/fret-ui-shadcn/tests/combobox_test_id_prefix_semantics.rs`
  - `ecosystem/fret-ui-shadcn/tests/combobox_keyboard_navigation.rs`
  - `ecosystem/fret-ui-shadcn/tests/combobox_escape_dismiss_focus_restore.rs`
  - `ecosystem/fret-ui-shadcn/tests/combobox_filtering.rs`
