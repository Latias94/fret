## Select + Combobox Deep Redesign v1 (TODO + Tracker)

Last updated: 2026-03-02.

This tracker is **workstream-local**. It exists because `select` and `combobox` need deeper
structural work than the “part surface alignment” stream.

### Reference anchors (upstream)

- `select` base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx`
- `combobox` base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`

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

