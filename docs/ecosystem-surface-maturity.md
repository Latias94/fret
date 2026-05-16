# Ecosystem Surface Maturity Gates

This document defines the evidence used to decide whether an Incubating Component Surface is ready
for broader recommendation or default-path consideration.

`CONTEXT.md` defines the language. This document defines the operating checklist.

## Scope

These gates apply to official Fret Component Surfaces such as `fret-ui-shadcn`,
`fret-ui-material3`, IMUI-style surfaces, and future design-system surfaces.

They do not apply to every Domain UI Package. A domain package may define a narrower gate when it
is not trying to become a general-purpose component surface.

## Gate A: Official Incubating Surface

An official Incubating Component Surface should have:

- A clear README describing what the surface is for and when to use it.
- Named Reference Sources or Behavior References.
- A stable crate identity and explicit relationship to the Runtime Substrate and Policy Layer.
- At least one runnable example or gallery route.
- A documented status that says it is not yet the Default Component Surface.

## Gate B: Broad Recommendation Candidate

Before recommending a surface broadly to App Authors, it should have:

- Coverage for the core app-building families expected by the surface.
- Documented theme or token strategy with stable public names.
- Interaction behavior mapped to explicit Behavior References where relevant.
- Focus, keyboard, disabled, hover, active, and selected states covered for interactive families.
- Screenshot or diagnostics evidence for representative states.
- Focused tests for non-trivial behavior and regressions.
- Onboarding examples that an App Author can follow without reading runtime internals.
- Clear escape hatches for Framework Integrators.

## Gate C: Default Component Surface Candidate

Before a surface can replace or compete with the Default Component Surface, it should have:

- First-hour onboarding quality comparable to the existing Golden Path.
- Broad coverage for forms, layout, overlays, navigation, feedback, data display, and command-like workflows.
- Stable public imports, naming, and recipe conventions.
- Documented accessibility baseline and known gaps.
- Diagnostics scripts or repeatable harnesses for representative components.
- Visual regression evidence for key states and density/theme variants.
- A support plan for keeping examples, templates, and documentation current.

## Review Rule

Promotion is evidence-based. A surface should not be promoted because it is strategically appealing
unless the relevant gate evidence exists or the gaps are explicitly accepted in a workstream.
