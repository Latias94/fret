## Select + Combobox Deep Redesign v1 (TODO + Tracker)

Last updated: 2026-03-02.

This tracker is **workstream-local**. It exists because `select` and `combobox` need deeper
structural work than the “part surface alignment” stream.

### Reference anchors (upstream)

- `select` base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx`
- `combobox` base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`

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
| `useComboboxAnchor` | same | `useComboboxAnchor` (PascalCase) | same | Upstream returns a DOM ref; Fret returns a layout-only `PopoverAnchor` wrapper exposing a stable element id. |

#### Fret-only helper exports (migration ergonomics)

These do not exist upstream but reduce churn when porting examples:

- `combobox_option(...)`
- `combobox_option_group(...)`

### Status legend

- `Not started`
- `In progress`
- `Done (with known gaps)`
- `Done`
- `Deferred (planned)`

### Tracker table

| Component | Target surface | Current state (Fret) | Known gaps / risks | Proposed changes (layer) | Gates | Status |
|---|---|---|---|---|---|---|
| `select` | shadcn v4 part surface (`Select*`) + stable `test_id` | Implemented via adapters; behavior is usable but not fully upstream-shaped | Composition drift, focus/keyboard edge cases, automation surfaces not uniformly documented | Extract/align shared listbox substrate (`kit`), keep shadcn defaults (`shadcn`) | Unit tests: open/close + focus restore + keyboard nav; optionally diag script for overlay flows | Not started |
| `combobox` | shadcn v4 part surface (`Combobox*`) + docs-aligned examples | Part adapters exist; still “known gaps” by design doc | Input-in-trigger ergonomics, Base UI-style expectations, structural adapter debt | Shared substrate (`kit`), refine part surface + adapters (`shadcn`), document explicit differences | Unit tests: filtering/typeahead + focus model + stable `test_id`; optionally diag script for overlay interactions | Not started |

### Proposed `test_id` scheme (for gates + scripted diags)

We should commit to a stable `test_id` naming scheme before deep refactors, so we can gate behavior
without binding tests to internal structure.

#### Select

- Trigger: `select.trigger`
- Content root: `select.content`
- Viewport / scroll container: `select.viewport`
- Items: `select.item.<value>` (value should be stable and URL/identifier-safe)
- Group labels: `select.label.<heading>` (optional)
- Scroll buttons: `select.scroll_up`, `select.scroll_down`

#### Combobox

- Input: `combobox.input`
- Trigger (if present): `combobox.trigger`
- Clear button: `combobox.clear`
- Content root: `combobox.content`
- List / scroll container: `combobox.list`
- Items: `combobox.item.<value>`
- Empty state: `combobox.empty`
- Chips (multiple): `combobox.chip.<value>` and remove affordance `combobox.chip_remove.<value>`

### Immediate next steps (proposed)

1. **Audit current APIs**:
   - enumerate current public parts/functions in `ecosystem/fret-ui-shadcn/src/select.rs` and
     `ecosystem/fret-ui-shadcn/src/combobox.rs`,
   - map them to upstream part names and call-site expectations.
2. **Decide focus model** per component:
   - active-descendant listbox model vs roving focus on items,
   - how it maps to platform semantics and automation.
3. **Define stable `test_id` scheme**:
   - trigger id,
   - content viewport id,
   - option/item ids (including groups).
4. **Extract shared substrate** in `ecosystem/fret-ui-kit` and gate it before rewriting recipes.

### Evidence checklist (fill as we implement)

For each milestone, record 1–3 evidence anchors:

- file + symbol anchors (e.g. `ecosystem/fret-ui-kit/src/...`),
- unit tests (`cargo test -p fret-ui-shadcn --lib <filter>`),
- diag scripts (if used).
