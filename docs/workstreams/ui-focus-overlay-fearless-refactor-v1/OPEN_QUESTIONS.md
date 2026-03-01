# Open Questions

- Touch semantics: should delayed (touch-up) outside-press dismissal apply the same focus clearing
  rules as mouse outside press? (Current intent: align unless a platform-specific reason exists.)
- Should `prevent_default()` for outside press suppress *only* focus clearing, or also other default
  effects in the future (e.g. hover clearing / active-state clearing)?
- What is the long-term source of truth for “branch containment” under portals/tears-out/multi-root?
- Phase C: where should the dispatch snapshot live (tree vs runtime service), and how should it be
  exposed for diagnostics without locking us into unstable internal types?

