# Shadcn Component Surface Audit v1

Last updated: 2026-03-02.

## Goal

Maintain an explicit, reviewable tracker for **component surface completeness** when aligning
`ecosystem/fret-ui-shadcn` to upstream shadcn/ui v4 (Radix + Base UI).

This workstream is about:

- Part/slot **splits** (e.g. `CardHeader`, `SelectTrigger`, `ComboboxContent`)
- Public API naming (no `v4` in exported symbols)
- Stable automation surfaces (`test_id` / `test_id_prefix`)
- Identifying where “missing parity” is a **mechanism** issue vs a **kit primitive** gap vs a
  **recipe** default

This is *not* a pixel-perfect tracker. Visual parity gets its own gates (web-vs-fret harness and/or
diag scripts).

## Sources of truth

- Upstream docs index: `repo-ref/ui/apps/v4/content/docs/components/radix/`
- Upstream base sources: `repo-ref/ui/apps/v4/registry/bases/radix/ui/*.tsx`
- Fret shadcn crate: `ecosystem/fret-ui-shadcn/src/*.rs`
- Contract boundaries: `docs/architecture.md`, `docs/adr/0066-fret-ui-runtime-contract-surface.md`

## Audit method (repeatable)

For each component:

1. Read upstream base file and list its `export { ... }` surface.
2. Compare with the Rust module’s public structs/builders/parts.
   - Note: In Fret some components are *re-export adapters* (e.g. `aspect_ratio`, `drawer`), so
     “module-local `pub struct ...`” is not the only signal. Verify that the symbols are available
     from the crate root `fret_ui_shadcn::*` before calling it a gap.
3. Classify drift:
   - **Recipe default drift**: sizing/spacing/tokens/composition; fix in `fret-ui-shadcn`.
   - **Policy / infra drift**: dismiss/focus/typeahead/active-descendant; fix in `fret-ui-kit`.
   - **Mechanism drift**: layout, hit-testing, semantics snapshot, overlay routing; fix in `crates/*`.
4. Lock at least one gate before deep refactors:
   - `test_id` stability tests (cheap)
   - targeted semantics behavior tests (keyboard/focus)
   - diag scripts for motion/overlay scenarios when determinism matters

## Non-goals (v1)

- Exhaustive per-token parity tracking
- Re-implementing upstream DOM APIs; we only need outcome parity
- Component catalog redesign (this workstream is about alignment, not taxonomy)

## Current baseline (as of 2026-03-02)

- Upstream Radix base export surfaces have corresponding Rust symbols across the shadcn crate.
- Remaining parity work is primarily about **behavior outcomes** (dismiss/focus/keyboard) and
  **composition defaults** (layout constraints), not missing part splits.
