# ADR 0329: Direction and Writing-Mode Boundary (Logical LTR/RTL Only)

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret already has a shared logical direction contract that is used by RTL-sensitive recipes,
overlay placement, and logical text alignment. The mechanism harness lane needs that boundary to be
explicit so it can prove logical mirroring without accidentally turning recipe policy into
framework truth.

The current implementation already provides:

- `LayoutDirection::{Ltr,Rtl}` as the shared logical direction enum.
- an inherited direction helper in `fret-ui-kit`.
- explicit direction threading across portal/overlay roots.
- logical `Start` / `End` text alignment resolution.

What Fret does **not** have is a general CSS `writing-mode` framework contract. Direction and
writing-mode are related, but they are not the same thing. Treating them as one substrate would
make future layout oracles ambiguous.

## Decision

1. `LayoutDirection` is the framework's logical inline-flow direction contract.
   - It is limited to `Ltr` and `Rtl`.
   - It resolves by local override, then inherited provider state, then `Ltr`.

2. Logical start/end mapping may flip under RTL only when a helper explicitly consumes direction.
   - `TextAlign::Start` and `TextAlign::End` are logical.
   - Physical left/right/top/bottom remain physical unless a helper explicitly translates them.

3. Provider state is recipe-owned and must be reinstalled across explicit portal/root boundaries.
   - Overlay roots reset inherited scope.
   - If a subtree needs direction inside a portal, the caller must thread it into the portal root
     explicitly.

4. `writing-mode` is not a general framework contract yet.
   - Fret does not infer vertical text flow or general CSS writing-mode behavior from
     `LayoutDirection`.
   - Any future writing-mode substrate needs its own ADR, its own explicit type, and dedicated
     proof surfaces.

5. Mechanism harnesses may use RTL to verify logical mirroring, but they must not encode
   recipe-specific padding or component layout choices as layout-engine truth.

## Consequences

- RTL mechanism oracles can proceed against a stable contract.
- Recipe and mechanism boundaries stay explicit.
- General writing-mode work remains intentionally blocked until a separate contract exists.

## References

- `crates/fret-core/src/layout_direction.rs`
- `ecosystem/fret-ui-kit/src/primitives/direction.rs`
- `ecosystem/fret-ui-kit/src/primitives/portal_inherited.rs`
- `crates/fret-ui/src/overlay_placement/types.rs`
- `ecosystem/fret-ui-kit/src/ui.rs`
- `docs/audits/radix-direction.md`
- `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
