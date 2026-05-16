# Fret Context

Fret is the domain language glossary for the framework and its surrounding ecosystem.
It exists to keep product, architecture, and code discussions aligned on the same terms.

## Language

**Fret**:
A GPU-first Rust application UI framework for desktop-first apps that can scale to editor-grade UI, embedded viewports, multi-window workflows, and wasm/WebGPU.
_Avoid_: game editor, engine editor, rendering engine

**Editor-grade UI**:
Application UI with long-lived state, docking, overlays, multi-window workflows, embedded viewports, and precise input routing.
_Avoid_: game editor, editor app, engine

**Editor Product Domain**:
Product-specific editor or engine responsibilities such as assets, scenes, tools, gizmos, build pipelines, and undo policy.
_Avoid_: editor-grade UI, runtime substrate, framework scope

**App Author**:
A developer who builds an application on Fret through the recommended high-level app and tooling surfaces.
_Avoid_: end user, framework user, consumer

**Declarative App Authoring**:
The default Fret authoring model where App Authors describe UI through composable views, actions, state, and component surfaces rather than manually managing the runtime tree.
_Avoid_: retained widget programming, manual tree management, renderer programming

**Framework Integrator**:
A developer who manually composes Fret runtime, platform, runner, renderer, or host integration layers for specialized products.
_Avoid_: app author, backend maintainer, engine owner

**Golden Path**:
The recommended high-level Fret entry route for App Authors to start quickly while preserving escape hatches to lower-level layers.
_Avoid_: everything crate, only supported path, beginner mode

**Runtime Substrate**:
The mechanism-only Fret UI layer that provides reusable interaction, layout, rendering, and semantics foundations without owning component policy.
_Avoid_: component library, design system, widget kit

**Portable Framework Contracts**:
Fret's stable, backend-agnostic framework contract layer for identity, runtime boundaries, UI mechanisms, scene semantics, input, and portable host-facing types.
_Avoid_: core crates, backend, launcher

**Backend Integration**:
Fret's platform or renderer-specific integration layer that connects portable contracts to concrete windowing, rendering, accessibility, or browser/native backends.
_Avoid_: portable contract, component ecosystem, app product

**Launch Integration**:
Fret's default wiring layer that composes runtime, backend, renderer, and effect draining into runnable application entry paths.
_Avoid_: app framework, component surface, product shell

**Policy Layer**:
The ecosystem layer that composes runtime mechanisms into opinionated component behavior, visual recipes, and app-facing defaults.
_Avoid_: runtime substrate, kernel, backend

**Behavior Reference**:
An upstream source used to define interaction, accessibility, placement, or component behavior outcomes without becoming a Fret implementation dependency.
_Avoid_: runtime dependency, source port, visual style

**Reference Source**:
An external project, specification, or design system used to inform Fret vocabulary, architecture, behavior, visuals, or ergonomics without becoming a normative implementation dependency.
_Avoid_: dependency, upstream authority, copied implementation

**Component Ecosystem**:
The set of Fret ecosystem crates that provide component libraries, design-system surfaces, recipes, and domain UI packages above the Runtime Substrate.
_Avoid_: shadcn only, standard library, runtime

**Ecosystem Crate**:
An official Fret crate whose responsibility is policy, defaults, reusable integration, component surfaces, or domain UI above the portable framework contracts.
_Avoid_: directory member, third-party package, kernel crate

**Component Surface**:
A named component library or design-system layer within the Component Ecosystem with its own taxonomy, recipes, maturity, and upstream references.
_Avoid_: behavior reference, runtime substrate, backend

**Default Component Surface**:
The Component Surface recommended by the Golden Path for App Authors starting a general-purpose Fret app.
_Avoid_: only component surface, component system, runtime default

**Incubating Component Surface**:
An official Component Surface with a clear direction and upstream references that has not yet reached golden-path maturity, coverage, or stability.
_Avoid_: toy, plugin, third-party, secondary component system

**Maturity Gate**:
A documented evidence threshold used to decide whether an Incubating Component Surface is ready for broader recommendation or default-path consideration.
_Avoid_: preference, popularity, maintainer taste

**Language Alignment Pass**:
A focused documentation pass that updates existing Fret docs to use the canonical glossary terms without changing framework contracts.
_Avoid_: rewrite, rebrand, contract change

**Domain UI Package**:
An Ecosystem Crate that provides UI for a specialized product domain or workflow rather than a general design-system component surface.
_Avoid_: component surface, app product, runtime feature

**App-Owned State**:
Application state that belongs to the App Author's product model while Fret provides handles, update lanes, and UI integration points.
_Avoid_: framework-owned state, global store, runtime state

**Action**:
A typed intent that app or component code can dispatch without requiring menu, palette, or keymap metadata.
_Avoid_: command, key binding, event

**Command**:
A discoverable user operation with routing, availability, metadata, and optional menu or command-palette presence.
_Avoid_: action, key binding, callback

**Key Binding**:
A configurable input gesture mapping that invokes a Command or Action through Fret's routing model.
_Avoid_: command, shortcut handler, event listener

**State Helper**:
A Fret-provided utility for local, derived, asynchronous, or cached state that supports App-Owned State without owning the product model.
_Avoid_: app architecture, product model, mandatory state manager

**Portable Style Vocabulary**:
Author-facing style and layout language borrowed from mature UI ecosystems while remaining independent of DOM or CSS runtime semantics.
_Avoid_: CSS compatibility, DOM styling, custom-only terminology

**Typed Style Token**:
A strongly typed Fret style value used to express design-system choices such as spacing, color, radius, typography, or elevation.
_Avoid_: CSS variable, stringly style, raw design value

**GPU-First Rendering**:
Fret's commitment that the primary app presentation path is designed around GPU-backed rendering while non-presentational surfaces may remain headless or diagnostic.
_Avoid_: GPU-only, renderer-owned UI, graphics engine

**Headless Surface**:
A Fret surface that can exercise behavior, layout, semantics, state, or diagnostics without requiring a real GPU-backed presentation path.
_Avoid_: fake UI, unsupported mode, non-runtime

**UI Render Asset**:
A resource used by Fret UI presentation such as fonts, icons, images, themes, or renderer-registered visual resources.
_Avoid_: project asset, engine asset, import pipeline

**Project Asset**:
An application or editor-domain asset with product-specific identity, import, dependency, build, or persistence rules.
_Avoid_: UI render asset, icon, theme token

**Desktop-First Platform Strategy**:
Fret's platform stance that desktop app workflows are the primary maturity target while portable contracts avoid native-only assumptions.
_Avoid_: desktop-only, mobile-ready, browser framework

**WebGPU Path**:
The wasm/WebGPU route that keeps Fret portable to browser-hosted demos and future web-facing app surfaces without making Fret a DOM framework.
_Avoid_: DOM runtime, SSR path, native fallback

**App-Facing Diagnostics**:
Diagnostics that App Authors can run on their own Fret applications to capture, package, inspect, and share reproducible evidence.
_Avoid_: maintainer suite, internal script, debug-only tool

**Maintainer Diagnostics**:
Repository maintainer diagnostics for campaigns, registries, promoted suites, performance gates, and framework regression matrices.
_Avoid_: app-facing CLI, public starter workflow, product feature

**App Author Tooling**:
Public Fret tooling for App Authors to create, run, configure, inspect, and package evidence for their applications.
_Avoid_: runtime, library API, maintainer workflow

**Maintainer Tooling**:
Repository-only or maintainer-focused tooling for framework development, campaigns, registries, release checks, and regression control.
_Avoid_: app author tooling, public golden path, runtime feature

## Relationships

- **Fret** provides the foundation for **Editor-grade UI** without owning editor or engine domain logic.
- **Editor-grade UI** is a capability level; the **Editor Product Domain** remains owned by applications or specialized ecosystem layers.
- An **App Author** uses Fret's golden path before reaching for **Framework Integrator** surfaces.
- **Declarative App Authoring** is the default model for **App Authors** on the **Golden Path**.
- A **Framework Integrator** may build **Editor-grade UI** products without making Fret own the product domain.
- The **Golden Path** is optimized for **App Authors** and must not collapse the lower-level surfaces used by **Framework Integrators**.
- The **Golden Path** may choose a **Default Component Surface** without making it the whole **Component Ecosystem**.
- The **Runtime Substrate** enables component behavior; the **Policy Layer** decides component defaults.
- **Portable Framework Contracts** define stable framework boundaries; **Backend Integration** and **Launch Integration** make those contracts runnable on concrete targets.
- The **Policy Layer** may target **Editor-grade UI** patterns without moving product domain logic into **Fret**.
- A **Behavior Reference** is a **Reference Source** focused on interaction or accessibility outcomes.
- A **Behavior Reference** informs a **Component Surface** but does not define the whole **Component Ecosystem**.
- A **Reference Source** informs Fret decisions, but accepted ADRs remain the project-level decision record.
- The **Component Ecosystem** contains multiple **Component Surfaces**; a golden-path surface does not exclude incubating or specialized surfaces.
- The **Component Ecosystem** is defined by responsibility; an **Ecosystem Crate** is not defined only by its repository directory.
- An **Incubating Component Surface** is official but not yet the **Default Component Surface**.
- A **Maturity Gate** determines when an **Incubating Component Surface** is ready for broader recommendation.
- A **Language Alignment Pass** updates wording to match the glossary while leaving ADR-backed contracts unchanged.
- A **Domain UI Package** may depend on a **Component Surface** without becoming one itself.
- **App-Owned State** may use **State Helpers**, but the application remains the owner of product state and policy.
- An **Action** expresses typed intent; a **Command** adds discoverability and routing metadata; a **Key Binding** maps input to either operation path.
- **Portable Style Vocabulary** gives App Authors familiar language; **Typed Style Tokens** keep Fret style contracts explicit and non-DOM-bound.
- **GPU-First Rendering** defines the main app presentation direction; **Headless Surfaces** remain valid for behavior, semantics, diagnostics, and tests.
- Fret may help load and register **UI Render Assets**; **Project Assets** belong to applications or the **Editor Product Domain**.
- The **Desktop-First Platform Strategy** is the current maturity focus; the **WebGPU Path** preserves portability without turning Fret into a DOM framework.
- **App-Facing Diagnostics** are part of Fret's product surface; **Maintainer Diagnostics** serve framework development and regression control.
- **App Author Tooling** supports the **Golden Path**; **Maintainer Tooling** supports framework development and should not leak into default app workflows.

## Example Dialogue

> **Dev:** "Are we building a game editor?"
> **Domain expert:** "No. **Fret** is the UI framework that can support **Editor-grade UI**; the editor app remains a separate product layer."
>
> **Dev:** "Does supporting **Editor-grade UI** mean Fret owns asset databases and tool modes?"
> **Domain expert:** "No. Those belong to the **Editor Product Domain**, not the framework core."
>
> **Dev:** "Should the first tutorial explain runner and renderer wiring?"
> **Domain expert:** "No. Start with the **App Author** path; expose **Framework Integrator** surfaces when the app needs custom integration."
>
> **Dev:** "Should app authors manually manage runtime tree nodes?"
> **Domain expert:** "No. Teach **Declarative App Authoring** first; retained runtime details are lower-level mechanisms."
>
> **Dev:** "Is the **Golden Path** the only supported way to use Fret?"
> **Domain expert:** "No. It is the recommended starting route for **App Authors**, with escape hatches for **Framework Integrators**."
>
> **Dev:** "Should dialog dismissal rules live in the runtime?"
> **Domain expert:** "No. The **Runtime Substrate** exposes the mechanisms; the **Policy Layer** composes the dialog behavior."
>
> **Dev:** "Are all crates under `crates/` portable core?"
> **Domain expert:** "No. Separate **Portable Framework Contracts** from **Backend Integration** and **Launch Integration**."
>
> **Dev:** "Is shadcn the whole component story?"
> **Domain expert:** "No. It is one maintained **Component Surface** in the broader **Component Ecosystem**; other surfaces can use different **Behavior References** and design-system sources."
>
> **Dev:** "Should we copy Radix's DOM portal implementation?"
> **Domain expert:** "No. Radix is a **Reference Source**; Fret ports behavior outcomes through its own ADR-backed runtime model."
>
> **Dev:** "Can the tutorial use shadcn by default?"
> **Domain expert:** "Yes. shadcn can be the **Default Component Surface** for the **Golden Path** without becoming the only component surface."
>
> **Dev:** "Is Material 3 outside the official ecosystem?"
> **Domain expert:** "No. It is an **Incubating Component Surface** until it reaches golden-path maturity."
>
> **Dev:** "When can Material 3 become a default recommendation?"
> **Domain expert:** "When it passes the relevant **Maturity Gate**, not just because it is strategically interesting."
>
> **Dev:** "Should old docs be rewritten immediately?"
> **Domain expert:** "No. Use a **Language Alignment Pass** when touching docs, or run a focused pass when wording drift becomes costly."
>
> **Dev:** "If a crate moves out of `ecosystem/`, is it no longer ecosystem?"
> **Domain expert:** "No. An **Ecosystem Crate** is classified by responsibility, not by the current directory layout."
>
> **Dev:** "Does Fret own my product state?"
> **Domain expert:** "No. Fret supports **App-Owned State** with **State Helpers**, but your application owns its product model."
>
> **Dev:** "Is every button click a command?"
> **Domain expert:** "No. Most component callbacks dispatch an **Action**; use a **Command** when the operation needs routing, metadata, or discoverability."
>
> **Dev:** "Does Tailwind-like spacing mean Fret is a CSS runtime?"
> **Domain expert:** "No. It is **Portable Style Vocabulary** backed by **Typed Style Tokens**."
>
> **Dev:** "Do all tests need to run a real GPU renderer?"
> **Domain expert:** "No. **GPU-First Rendering** guides the app presentation path, while **Headless Surfaces** keep behavior and diagnostics testable."
>
> **Dev:** "Does the asset system import game assets?"
> **Domain expert:** "No. Fret handles **UI Render Assets**; **Project Assets** remain app or editor-domain responsibility."
>
> **Dev:** "Does cross-platform mean mobile is already a first-class target?"
> **Domain expert:** "No. Fret follows a **Desktop-First Platform Strategy** while keeping the **WebGPU Path** open."
>
> **Dev:** "Should a normal app author run the full parity campaign?"
> **Domain expert:** "No. Use **App-Facing Diagnostics** for shareable evidence; keep campaign orchestration in **Maintainer Diagnostics**."
>
> **Dev:** "Should component crates depend on the CLI?"
> **Domain expert:** "No. **App Author Tooling** and **Maintainer Tooling** are workflow surfaces, not runtime dependencies."

## Flagged Ambiguities

- "Rust UI framework" was too broad; resolved: use **Fret** as a GPU-first Rust application UI framework with editor-grade scalability.
- "Editor-grade" could imply game/editor product ownership; resolved: distinguish **Editor-grade UI** from the **Editor Product Domain**.
- "User" was ambiguous; resolved: distinguish **App Author** from **Framework Integrator**.
- "Authoring model" could imply retained widget programming; resolved: use **Declarative App Authoring** for the default app-facing model.
- "Golden path" could imply a mandatory one-crate abstraction; resolved: **Golden Path** is recommended, not exclusive.
- "Component behavior" was ambiguous; resolved: distinguish **Runtime Substrate** mechanisms from **Policy Layer** defaults.
- "Core crates" could blur portability and integration boundaries; resolved: distinguish **Portable Framework Contracts**, **Backend Integration**, and **Launch Integration**.
- "Reference" could imply source copying or runtime dependency; resolved: use **Reference Source** and port outcomes/contracts instead.
- "shadcn" could imply the whole component ecosystem; resolved: shadcn is one **Component Surface** inside the broader **Component Ecosystem**.
- "Default components" could imply runtime ownership; resolved: the **Default Component Surface** belongs to the **Component Ecosystem**.
- "Experimental component surface" could sound unofficial; resolved: use **Incubating Component Surface** for official but not-yet-golden-path surfaces.
- "Maturity" could imply subjective preference; resolved: use **Maturity Gate** for evidence-based readiness.
- "Terminology cleanup" could imply broad rewrites or contract changes; resolved: use **Language Alignment Pass** for wording-only documentation alignment.
- "Ecosystem" could mean the `ecosystem/` directory only; resolved: use **Ecosystem Crate** for responsibility-based official crates above portable contracts.
- "State management" could imply a mandatory framework-owned store; resolved: use **App-Owned State** plus optional **State Helpers**.
- "Command" was overloaded; resolved: distinguish **Action**, **Command**, and **Key Binding**.
- "Tailwind-like" could imply CSS compatibility; resolved: use **Portable Style Vocabulary** backed by **Typed Style Tokens**.
- "GPU-first" could imply GPU-only; resolved: **GPU-First Rendering** still allows **Headless Surfaces**.
- "Asset" could mean UI presentation resources or editor product assets; resolved: distinguish **UI Render Asset** from **Project Asset**.
- "Cross-platform" could imply equal maturity on every target; resolved: use **Desktop-First Platform Strategy** plus **WebGPU Path**.
- "Diagnostics" could mean either user-facing evidence capture or internal campaigns; resolved: distinguish **App-Facing Diagnostics** from **Maintainer Diagnostics**.
- "Tooling" could imply runtime ownership; resolved: distinguish **App Author Tooling** and **Maintainer Tooling** from library/runtime surfaces.
