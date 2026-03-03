# Open Source Readiness (Fearless Refactor v1) — Milestones

## Milestone 0 — Public entry points are obvious

Outcome:

- A new GitHub user can find the right “first run” command quickly.

Exit criteria:

- Root `README.md` recommends:
  - cookbook (lightweight),
  - templates ladder,
  - optionally gallery (deep dive).

## Milestone 1 — Cookbook is curated (not a dump)

Outcome:

- Cookbook has a short recommended order and a clear separation of Official vs Lab examples.
- Advanced examples are feature-gated.

Exit criteria:

- The first 5–8 cookbook examples are boring, stable, and copy/paste-friendly.

## Milestone 2 — `fret` defaults are smooth, and opt-outs are real

Outcome:

- Depending on `fret` with `default-features = false` does not pull selector/query unless requested.
- Docs describe feature profiles clearly.

Exit criteria:

- App authors can choose between “minimal” and “batteries” without guessing.

## Milestone 3 — UI Gallery is approachable

Outcome:

- UI Gallery has a lite mode suitable for onboarding.

Exit criteria:

- Lite mode loads quickly and provides a meaningful “what is shadcn in Fret?” tour.

## Milestone 4 — `fret-demo` reads as maintainer/labs

Outcome:

- Lesson-shaped demos live in cookbook; `fret-demo` retains only labs/harnesses.

Exit criteria:

- The official native demo list remains intentionally small and stable.

