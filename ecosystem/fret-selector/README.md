# `fret-selector`

Derived state selector utilities for Fret apps and components.

A selector is a small, explicit derived-state primitive:

- memoize an expensive computation behind a dependency signature (`Deps: PartialEq`)
- keep dependency tracking explicit (no hidden global reactive graph)

## Status

Experimental learning project (not production-ready).

## Features

- `ui`: element-context helpers for collecting dependency tokens while registering observations

