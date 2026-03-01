# UI Gallery Fearless Refactor (Workstream Plan)

## Context

UI Gallery is our “component gallery” surface. It is used for:

- Quick manual verification (visual + interaction).
- Demonstrating shadcn-aligned recipes (copy/paste friendly).
- Diagnostics anchors (`test_id`) for scripted runs and parity gates.

Today, many pages render a preview with real Rust code, but the “Code” tab shows a separately
maintained string literal. This creates drift (the preview is correct, the copyable code is not),
especially when layout constraints must be explicit (e.g. `flex_1()` / `min_w_0()`).

This workstream aims to make UI Gallery **fearlessly refactorable** by enforcing a single source of
truth for examples.

## Goals

1. **Single-source examples**: the preview and copyable snippet are generated from the same code.
2. **Copyability**: code shown in UI Gallery is “correct by construction” (compiles, matches preview).
3. **Scalable maintenance**: refactors in recipes/components do not require hunting down duplicated
   string literals.
4. **Shadcn parity tracking**: every shadcn component has a tracked gallery/refactor status.

## Non-goals

- Redesigning the UI Gallery look & feel (layout/theme) as part of this pass.
- Changing core runtime mechanisms (`crates/fret-ui`) to emulate browser intrinsic sizing.
- Achieving full 1:1 DOM API parity (React slot-level composition) for every component in this pass.

## Key decision: “Layout constraint translation” lives in examples/recipes

In a GPU-first explicit-layout framework, many web examples rely on implicit browser sizing
heuristics. We align **outcomes**, not implicit browser behaviors. Therefore, UI Gallery examples
must encode the correct explicit constraints (e.g. `flex_1().min_w_0()`) and the copyable code must
match exactly.

Reference precedent:

- Zed and gpui-component routinely encode `min_w_0()` / `flex_1()` explicitly in UI composition.

## Proposed architecture

### 0) Pick the upstream doc variant explicitly (Base vs Radix)

Upstream shadcn v4 docs are split into two parallel trees:

- Base UI: `repo-ref/ui/apps/v4/content/docs/components/base/*.mdx`
- Radix UI: `repo-ref/ui/apps/v4/content/docs/components/radix/*.mdx`

For this workstream:

- Use **Base MDX** as the *primary* source of truth for composition, sizing, and recipe structure
  (what ends up in the copy/paste snippet).
- Use **Radix MDX** as a *cross-check* for interaction semantics and overlay expectations (dismiss,
  focus restore, keyboard navigation), together with APG where applicable.

The tracker table in `todo.md` records both paths so the review surface stays explicit.

### 1) Introduce “snippet modules” as the single source of truth

For each example, create a snippet file containing real Rust code that returns an `AnyElement`:

```
apps/fret-ui-gallery/src/ui/snippets/<component>/<example>.rs
```

Then:

- **Preview** imports and executes the snippet module (compiled code).
- **Code tab** displays the snippet file content via `include_str!`.

This makes drift impossible.

#### Snippet template (recommended)

Each snippet is real, compiled Rust code. Prefer snippets that are **self-contained** (they create
their own models/state) so the copy/paste outcome is usable without hidden page-level plumbing.

```rust
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    // Create models/state here (or in a small helper) so the snippet is standalone.
    // Ensure any layout constraints required for parity are explicit (e.g. flex_1 + min_w_0).
    todo!()
}
```

If a snippet must integrate with page-level models (e.g. to keep existing diag scripts stable),
prefer passing explicit inputs and still displaying the same file in the Code tab.

#### Import policy (user-facing by default)

Snippet files are the *copy/paste surface* for end users, so avoid internal-only imports like
`super::*` or `crate::ui::*` in displayed code.

Default to the “golden path” import stanza:

- `use fret_ui_shadcn::prelude::*;`
- plus small std imports as needed (`Arc`, `Rc`, etc.)

If an example genuinely needs additional crates (e.g. `fret_icons` icon packs), prefer importing the
public, user-facing symbols rather than reaching into UI Gallery internals.

### 2) Optional region slicing (for nicer displayed code)

Snippet files can optionally include regions:

```rust
// region: example
// endregion: example
```

UI Gallery can display only the region (string slicing), while still compiling the full file.

### 3) Add a tiny “gallery example” helper API (no macros required)

Prefer a small helper in UI Gallery (e.g. `doc_layout::code_from_file(...)`) over macros.

Macros (`stringify!`) tend to:

- Produce unformatted output (not rustfmt’d).
- Break down for multi-function examples.

### 4) Tracking + enforcement

- Add a component tracker table (see `todo.md`) and update it as we migrate pages.
- Add a lightweight check (test or build-time check) that ensures “Code tabs” are sourced from
  files, not raw string literals, for migrated pages.
  - For migrated pages, the desired end-state is “no raw code literals”, only file-backed snippets.

## Current UI Gallery issues (to address during refactor)

### A) Code/Preview drift

- The same example is expressed twice: real UI code + separate string literal.
- Layout fixes land in one place and not the other.

### B) Fragmented page taxonomy

- Some content lives under `apps/fret-ui-gallery/src/ui/pages/*`.
- Some content lives under `apps/fret-ui-gallery/src/ui/previews/pages/components/*`.
- Navigation/IDs can be difficult to map to upstream shadcn docs.

### C) Inconsistent “example boundaries”

- “Demo” sections sometimes include additional diagnostics-only hooks.
- Some examples depend on page-level models/state that are not visible in the code snippet.
- Some examples rely on implicit layout heuristics; in Fret these must be expressed as explicit
  constraints in the snippet to match upstream outcomes.

### D) Weak regression protection for examples

- Most example drift is only caught manually.
- We should treat migrated examples as compile-checked and (where valuable) diag-scripted.

## Migration strategy (incremental, low risk)

1. Add snippet infrastructure (helpers + directory layout).
2. Migrate the highest-drift pages first:
   - Button Group (contains nested layout + composite examples)
   - Select (overlay + scroll + alignment policy)
3. Continue migrating component pages in small batches.
4. Once a page is fully migrated, forbid new `r#"...` code literals on that page.

## Definition of done

- For each migrated example:
  - Preview is rendered by calling compiled snippet code.
  - Code tab is loaded from the snippet file content.
  - The snippet contains the explicit constraints required for correct rendering.
- Tracker table updated for the component(s).
