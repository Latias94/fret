# Mind model: widget state surfaces in app code

This note helps app authors decide when a shadcn-style widget should use plain local state, a narrow
bridge, or an explicit `Model<T>` boundary.

The goal is to avoid accidental boilerplate inflation in app code.

## Default rule

Choose the narrowest public surface that matches the product need:

- plain local snapshot + typed action for small local collections
- narrow interop bridge for widgets with model-backed internals
- explicit `Model<T>` only when state is intentionally shared, externally synchronized, or runtime-owned

## Practical checklist

### 1) Is the widget value just rendered from local view-owned data?

Examples:

- todo row checked state
- per-row expanded/collapsed flag in a local list
- simple on/off flags stored inside `LocalState<Vec<Row>>`

Preferred shape:

- render from a snapshot constructor if available (`from_checked(...)`, similar)
- mutate through typed actions / payload actions
- do not introduce per-row models unless the component contract truly requires it

### 2) Does the widget edit live text with IME/caret semantics?

Examples:

- `Input`
- `Textarea`

Preferred shape:

- use the narrow local bridge (`Input::new(&local_text)`)
- let the widget keep model-backed internals if needed

### 3) Does the widget need external synchronization or shared ownership?

Examples:

- app-wide settings models
- shared filters/query parameters
- complex selection state consumed outside the current view

Preferred shape:

- use explicit `Model<T>`
- make the shared boundary intentional and visible

### 4) Does the widget already provide an uncontrolled path?

Examples:

- `default_open(...)`
- `new_controllable(cx, None, default)`

Preferred shape:

- prefer the uncontrolled path when app code does not need to observe the state elsewhere
- avoid creating local models purely to satisfy an available controlled constructor

## Escalation rule

If a simple local-authoring case still forces `Model<T>` boilerplate, treat it as a component parity
question before you normalize the boilerplate in app code.

Ask:

1. is the missing surface a narrow snapshot/action path?
2. is the remaining gap actually label/control or a11y parity?
3. should this be handled by `fret-shadcn-source-alignment` instead of app-level helpers?

## Anti-patterns

Avoid these unless you have a clear shared-state requirement:

- per-row `Model<bool>` in a small local list just to satisfy a checkbox recipe
- converting every widget request into another generic helper
- widening to a broad `IntoModel<T>` story when a narrow widget-specific bridge is enough
