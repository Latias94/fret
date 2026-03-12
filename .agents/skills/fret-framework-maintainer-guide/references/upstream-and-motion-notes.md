# Upstream alignment and motion notes

Use this note when the maintainer task involves upstream parity work or motion behavior in Fret’s GPU-first renderer.

## 1) Motion changes: optimize for outcomes

If you are changing motion/animation behavior (especially overlays, drawers, sidebars):

- Optimize for parity of **outcomes** (timing, sequencing, interrupt/re-target rules), not for porting a DOM runtime.
- Keep authoring in wall-time (`Duration` / `durations_ms` theme tokens), not in “60fps ticks”.
- Keep motion parameters themeable (durations/easings/springs) so ecosystems can tune hand feel without editing code.
- Require at least one deterministic diag gate under fixed delta:
  - `fretboard diag run ... --fixed-frame-delta-ms 16`
  - or `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`
- Be explicit about renderer semantics:
  - DOM `transform` affects hit-testing
  - in Fret, choose `RenderTransform` vs `VisualTransform` intentionally to match pointer semantics

## 2) Current token-shape guidance

Recommended token shapes in the current ecosystem direction:

- shadcn recipes: durations + cubic-bezier easings + duration+bounce springs (Flutter-style)
- Material 3: published tokens are damping ratio + stiffness (motion scheme)
- Unification is optional; if it happens later, keep both shapes supported and provide a bridge

## 3) Upstream reference mapping

Use `fret-shadcn-source-alignment` when you want parity work plus gates.

Practical mapping:

- **Radix**: semantics + state-machine outcomes (dismiss/focus/keyboard nav/placement)
- **shadcn**: composition + taxonomy + sizing defaults (recipes)
- **Base UI**: headless accessibility patterns and part composition (unstyled primitives, event/state flows)

Use Base UI as an additional reference when DOM-centric assumptions need translating to Fret’s GPU-first custom renderer (semantics tree, hit-testing, focus routing, text/IME).

## 4) Questions to answer before coding

- Which upstream source is defining the user-visible truth?
- Which part is mechanism, which part is policy, and which part is recipe chrome?
- What deterministic gate will prove the motion/parity change?
- Do pointer semantics depend on `RenderTransform` rather than paint-only motion?
- Does the change belong in a shared ecosystem foundation rather than one component?

## 5) Useful anchors

- `.agents/skills/fret-shadcn-source-alignment/SKILL.md`
- `.agents/skills/fret-material-source-alignment/SKILL.md`
- `docs/reference-stack-ui-behavior.md`
- `docs/workstreams/motion-foundation-v1/motion-foundation-v1.md`
