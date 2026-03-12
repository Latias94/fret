# UI Assets image loading v1 — TODO

## P0 (unblock users)

- Add a UI Gallery page-level hint for image loading logs (where to set `RUST_LOG`).
- Ensure the Card preview debug overlay includes:
  - `status`
  - `intrinsic_size_px`
  - `error` (if any)
  - `path_exists` / `source present`
- Add one stable `test_id` target for the Card event cover container to support future diag scripts.

## P1 (parity + ergonomics)

- Shadcn parity check for Card cover presentation:
  - Confirm `ViewportFit::Cover` matches shadcn expectation for "event cover".
  - Validate clip/overflow behavior for rounded corners (Card chrome).
- Add an invariant test for `ViewportFit::Cover` math (crop window + UV mapping) if the mismatch is mechanism-level.
- Optional: add a small `tools/diag-scripts/` repro once the visual target is stable (only after the current bug is fixed).

## P2 (ecosystem integration)

- `query-integration` ergonomics:
  - Provide a `Query`-friendly wrapper that yields `ImageSourceState` without forcing query usage.
  - Keep the base path/bytes API usable without query.
- Consider an app-level asset resolver abstraction:
  - `ImageSource::from_path` is fine for native dev, but long-term packaging may want a virtual path layer.

## Performance & memory

- Budget tuning guides:
  - Document recommended defaults for image budgets per app class (gallery/editor).
- Validate decoded-byte retention:
  - Ensure decoded bytes are dropped as soon as GPU-ready is observed.

