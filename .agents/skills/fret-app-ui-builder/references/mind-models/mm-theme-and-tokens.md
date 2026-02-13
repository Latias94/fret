# Mind model: Theme and tokens (shadcn → Fret)

Goal: keep the UI consistent across light/dark, DPI, and platforms by using tokens instead of hardcoded colors/sizes.

## Token-first rule

If a value is part of shadcn “style vocabulary”, treat it as a token:

- Colors: `background`, `foreground`, `muted`, `muted-foreground`, `border`, `ring`, `primary`, `destructive`, `input`, ...
- Metrics: control heights, paddings, radii, row heights, icon sizes, ...

In Fret, prefer `Theme` lookups and token references (`ColorRef`, `MetricRef`, `Radius`, `Space`) over ad-hoc `Px(…)` / `Color { … }`.

## Where to define tokens

- App-level design decisions (theme selection, overrides) belong in app/bootstrap layers.
- Reusable shadcn recipes should resolve through theme keys so user apps can swap themes without patching recipes.

## Practical tips

- Avoid scattering “magic numbers” in recipes; route them through theme metrics.
- If you must adjust alpha/opacity for hovered/pressed states, keep it localized and deterministic (avoid random/per-frame effects).
- When a pixel-perfect value is required for parity, document the upstream source (shadcn class or Radix behavior) and gate with a test/repro.

## See also

- `fret-shadcn-source-alignment` (upstream parity workflow)
- Token cheat sheet: `references/theme/token-groups.md`
- Starter presets: `references/theme/editor-presets.md`
