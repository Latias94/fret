# UI diagnostics timebase decoupling (v1) — Open questions

## 1) What is the authoritative “timebase” for diagnostics scripts?

Options:

- Frame-based only: steps advance only on real rendered frames; in no-frame scenarios, fail with `timeout.no_frames`
  after a bounded wall time.
- Tick-based: advance “script ticks” on a runner timer even if no frames are presented.
- Hybrid: prefer frames, but allow the keepalive timer to advance some waits/timeouts.

What we need to decide:

- Which semantics model is least surprising for authors, while still preventing hangs.
- How this interacts with `frame_clock_fixed_delta_ms` (deterministic frame clock).

## 2) Should `wait_frames` be treated as “script ticks” under no-frame conditions?

Pros:

- Avoids hangs and keeps scripts simple.

Cons:

- “Frames” stop meaning “rendered frames”, which can be surprising for authors and might hide real rendering liveness
  bugs.

If we do not want to overload `wait_frames`, we likely need a schema evolution path (`wait_ms` / `yield`) and a
deprecation strategy.

## 3) Who owns liveness while scripts are pending (not yet started)?

Today:

- Most script trigger polling and script driving happens from per-window post-paint hooks.

If the app goes idle at the wrong time:

- the script trigger may not be observed promptly.

Possible owners:

- Diagnostics runtime: arm a timer when a script is *pending* (not only active).
- Tooling: enforce a keepalive mode in `--launch` config for all script-driven runs.
- Runner: an opt-in “diag keepalive” mode when `FRET_DIAG_CONFIG_PATH` is present.

## 4) What is the CPU budget for keepalive?

Questions:

- Is a fixed 16ms repeating timer acceptable for tool-launched runs (likely yes)?
- Should the interval be adaptive (e.g. 16ms while active drag, 33–100ms while just waiting)?
- Do we need a knob for very low power devices / CI VMs?

## 5) What evidence must be present on no-frame failures?

Minimum:

- stable `reason_code=timeout.no_frames`,
- last `step_index`,
- last known snapshot age and window id,
- at least one bounded bundle around the failure (or a guarantee that a bundle dump is attempted).

