# `fret-ui-headless`

Headless interaction policies and primitives for Fret UI composition.

This crate contains deterministic state machines and small reusable helpers that can be shared
across UI kits and component ecosystems, without depending on `fret-ui` rendering/theme details.

## Status

Experimental learning project (not production-ready).

## Examples of what lives here

- roving focus + typeahead
- menu navigation
- presence/transition helpers
- hover intent / tooltip delay groups
- table helpers and small layout-adjacent utilities

## Upstream references (non-normative)

Some modules in this crate intentionally port or align with upstream interaction engines for
behavioral parity:

- cmdk (command palette scoring + selection math): https://github.com/pacocoursey/cmdk
  - See `src/cmdk_score.rs` and `src/cmdk_selection.rs`.
- Embla Carousel (carousel behavior): https://github.com/davidjerleke/embla-carousel
  - See `src/embla/*`.

See [`docs/reference-stack-ui-behavior.md`](../../docs/reference-stack-ui-behavior.md) for repo-level
guidance on which upstream references are used for which behavior classes.
