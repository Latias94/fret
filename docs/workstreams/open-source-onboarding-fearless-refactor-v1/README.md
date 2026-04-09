# Open-source onboarding (fearless refactor v1)

Goal: make the repository feel “boring to onboard” for first-time users, without changing core
contracts. This workstream focuses on:

- Examples and runnable entry points (templates / cookbook / UI gallery / labs)
- Dependency and feature surfaces that affect cold build + “first run”
- README + docs navigation (clickable, GitHub-friendly links; minimal overload)

Non-goals:

- Rewriting the UI gallery architecture (we will gate/hide unfinished content instead)
- Committing to a stable hotpatch story (devloop is maintainer-focused and may change)

## Current stance (baseline)

- **Primary onboarding path**: templates + in-tree cookbook (small files, one concept each).
  - See: [docs/examples/README.md](../../examples/README.md)
  - See: [docs/first-hour.md](../../first-hour.md)
  - See: [apps/fret-cookbook/README.md](../../../apps/fret-cookbook/README.md)
- **UI gallery**: keep as a comprehensive catalog, but default to a lite surface and hide dev/material3
  behind features.
  - See: [apps/fret-ui-gallery/README.md](../../../apps/fret-ui-gallery/README.md)
- **Diagnostics (`fretboard-dev diag`)**: optional. Teach via cookbook with a single “hello” script instead
  of introducing “gate lists” up front.
  - See: [apps/fret-cookbook/README.md#diagnostics-optional](../../../apps/fret-cookbook/README.md#diagnostics-optional)

## Documents

- TODO list: [docs/workstreams/open-source-onboarding-fearless-refactor-v1/TODO.md](./TODO.md)
- Milestones: [docs/workstreams/open-source-onboarding-fearless-refactor-v1/MILESTONES.md](./MILESTONES.md)

