# Material3 Non-Color Token Governance v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M0 - Lane Opened

Status: Complete

- Non-color direct reads are split from the closed color fallback lanes.
- Workstream scope distinguishes fallback-chain debt from acceptable component scalar defaults.

## M1 - Typography Weight Ownership

Status: Complete

- Chip-family and Slider label weight reads are centralized through `tokens::typography`.

## M2 - Selection Numeric Residuals

Status: Complete

- Radio and adjacent selection-control numeric fallback paths use resolver vocabulary where they
  form component/system fallback chains.

## M3 - Motion Policy Classified

Status: Complete

- Dialog, Snackbar, and ModalNavigationDrawer motion token reads are either routed through shared
  helpers or documented as intentional motion-scheme policy.

## M4 - Time Picker/Input Classified

Status: Complete

- TimeInput and TimePicker numeric fallback chains are migrated or split into a field-family
  follow-on with exact evidence.

## M5 - Lane Verified

Status: Complete

- Formatting, token fixtures, targeted component tests, check/clippy, catalog, layering, and diff
  hygiene pass.
- Residual direct reads are limited to token-key registration, `MaterialTokenResolver`,
  `tokens::typography`, context flags, and fixture utilities.
