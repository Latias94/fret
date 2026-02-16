# Mind model: Design direction (frontend-design → Fret)

Goal: get a **distinctive, cohesive UI** without drifting into “default component library” aesthetics.

This is adapted from the web-centric `frontend-design` skill, but rewritten for Fret’s constraints:
typed tokens, declarative composition, and strong correctness/perf contracts.

## The 1-minute design brief

Write this before touching layout code:

- **Purpose + audience**: what problem does this surface solve? who uses it daily?
- **Tone**: pick one *extreme*, then execute it precisely.
  - Examples: brutally minimal, soft/diffuse, neubrutal/outlined, HUD/sci‑fi, editorial/magazine,
    industrial/utilitarian, playful/toy-like.
- **Constraints**: accessibility, performance, reduced motion, brand colors, font requirements,
  localization/RTL, code-heavy text.
- **Differentiation hook**: the one thing someone remembers after 3 seconds.
  - Examples: a signature focus ring, a distinctive panel chrome, a unique overlay treatment,
    a consistent density rhythm, a bold accent only used for “primary intent”.
- **Baseline style pick**: pick a `style_catalog.json` entry and commit to it.

Why it matters:

- Without a brief, teams “style by accident” and end up with inconsistent tokens, spacing rhythms,
  and overlay treatments.

## Mapping “design” to Fret mechanisms

### Typography

- Prefer a small type scale (usually 3 sizes): title / body / helper.
- Limit weights (often 2): regular + semibold.
- Pick fonts intentionally, but keep coverage in mind (icons, CJK, symbols).
- Treat font choices as **inputs** to token decisions (line height, control height, padding).

### Color & theme

- Commit to a dominant background + sharp accent, rather than a timid “everything evenly colored” palette.
- Use tokens for everything that can vary across scheme/density/platform.
- Keep overrides centralized in `ThemeConfig` (avoid per-component ad-hoc colors).

### Motion

- Motion should support intent (open/close, emphasis), not become ambient noise.
- Prefer a small number of high-impact moments over many tiny micro-animations.
- Respect reduced motion requirements; avoid continuous frames unless explicitly leased.

### Spatial composition

- Decide whether the surface is **dense** (editor) or **spacious** (settings/content) and keep it consistent.
- Use one spacing rhythm per surface class; avoid “almost the same but different” gaps.
- Reserve elevation/shadows for meaning (dialogs/menus), not as a default decoration.

### Backgrounds & visual details (optional, careful)

- Avoid “visual flair” that undermines legibility in an editor workflow.
- If you add texture/glass/blur, scope it to overlays or specific surfaces (not globally everywhere).

## “Avoid generic” without breaking Fret contracts

The web skill’s “avoid AI slop” principle maps cleanly to Fret if you translate it into constraints:

- Don’t scatter magic numbers; express identity via tokens + a small set of overrides.
- Don’t ship a default baseline with no differentiation hook.
- Don’t bake policy into `crates/*` to chase a look; keep policy in `ecosystem/*` and recipes.
- Always leave a gate for interaction-heavy work (diag script + stable `test_id`).

## Quick tool loop

- Pick a style quickly:
  - `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --suggest "<keywords>"`
- Apply one baseline + small override, then do a polish pass:
  - `references/polish/polish-pass.md`

