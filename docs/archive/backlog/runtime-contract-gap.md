# Runtime Contract Gap List (ADR 0066)

This document is a practical companion to `docs/adr/0066-fret-ui-runtime-contract-surface.md`.
It answers: “Do we already meet the Accepted runtime contracts, and what is missing?”

Status legend:

- **Done**: contract is implemented and has tests.
- **Partial**: mechanism exists but gaps remain for long-term use.
- **Missing**: contract is not yet implemented (or not yet stable enough).

## Snapshot

- Authoritative contract set: `docs/adr/0066-fret-ui-runtime-contract-surface.md` (Accepted)
- Primary references:
  - APG: `docs/reference-stack-ui-behavior.md`
  - Radix/shadcn outcomes: Radix UI Primitives (upstream: <https://github.com/radix-ui/primitives>; pinned locally, see `docs/repo-ref.md`), `repo-ref/ui`
  - Placement: `repo-ref/floating-ui`
  - Virtualization: `repo-ref/virtualizer` (Rust engine; primary)
  - Ergonomics reference: `repo-ref/gpui-component`

## Contract coverage (current)

| Contract (ADR 0066) | Status | Current implementation entry points | Key gaps (if any) |
| --- | --- | --- | --- |
| Input routing + hit testing | **Done** | `crates/fret-ui/src/tree/mod.rs` | — |
| Hover tracking + geometry queries | **Done** | `crates/fret-ui/src/elements/mod.rs` (`HoverRegion`, `bounds_for_element`), `crates/fret-ui/src/declarative.rs` (bounds recording) | — |
| Focus + capture + focus-visible + traversal | **Done** | `crates/fret-ui/src/tree/mod.rs` (modal scoping + `focus.next`/`focus.previous`), `crates/fret-ui/src/focus_visible.rs` | Component-layer focus trap/restore remains policy (ADR 0067); runtime traversal is conservative until a scroll-into-view contract is formalized (ADR 0068). |
| Multi-root layers substrate | **Done** | `crates/fret-ui/src/tree/layers/impls.rs` + `crates/fret-ui/src/tree/layers/types.rs` (`push_overlay_root_with_options`, `OverlayRootOptions`, `remove_layer`, `active_input_layers`) | — |
| Placement solver | **Partial** | `crates/fret-ui/src/overlay_placement/mod.rs` | Arrow support is intentionally deferred to P1 (ADR 0066 Gate 3.2). |
| Declarative authoring | **Done** | `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/elements/mod.rs`, `crates/fret-ui/src/declarative.rs` | — |
| Layout vocabulary | **Done** | `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative.rs` | Layout defaults should remain CSS/Tailwind-like; avoid adding per-component defaults in runtime. |
| Scroll contract | **Done** | `crates/fret-ui/src/scroll.rs`, declarative scroll element in `crates/fret-ui/src/declarative.rs` | — |
| Virtualization contract (TanStack alignment) | **Done** | `crates/fret-ui/src/virtual_list.rs`, `crates/fret-ui/src/elements/mod.rs`, declarative `VirtualList` in `crates/fret-ui/src/declarative.rs` | Lanes/masonry remain P1. |
| Text input / IME engine contract | **Partial** | `crates/fret-ui/src/text_input/mod.rs` + ADR 0012/0044/0045/0046 | Single-line is usable; multiline and geometry queries must remain the Stable contract boundary (component chrome stays out of runtime). |
| Semantics tree | **Partial** | `crates/fret-ui/src/tree/mod.rs` + `crates/fret-core/src/semantics.rs` | `SemanticsSnapshot` exposes `barrier_root` and per-root flags; the eventual platform bridge must enforce “background is inert/hidden under barrier” (ADR 0066 Gate H). |

## P0 “make it usable” tasks (recommended order)

These are the highest-leverage follow-ups before scaling component work.

1) **virtualizer alignment (virtualization contract)**
   - Done in runtime substrate (`VirtualItem`, `scrollMargin`, `gap`, `rangeExtractor`, stable-key size cache).
   - Remaining P1: lanes/masonry + more scroll strategies if needed.

## MVP mapping

This is how the tasks above map into the existing MVP queue:

- MVP 62: overlay behavior + placement (Radix/Floating alignment)
  - substrate: layer uninstall API + barrier semantics tests (runtime)
  - policy: dismissal/focus trap/restore tests (components)
- MVP 63: unify scroll ergonomics (GPUI-like)
  - `ScrollHandle` contract completion and shared scroll-to vocabulary
- MVP 56 / 50: unify virtualization around composable declarative rows
  - virtualizer alignment + stable keys (ADR 0070) + shared selection/scroll-to patterns
