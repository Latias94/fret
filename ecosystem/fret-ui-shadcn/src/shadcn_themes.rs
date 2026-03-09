//! Built-in shadcn/ui theme presets.
//!
//! The upstream shadcn/ui project ships theme definitions as CSS variable sets (HSL/OKLCH).
//! Fret's runtime theme system is token-based (see `fret_ui::ThemeConfig`), so we convert those
//! CSS variables into `ThemeConfig` maps and rely on `Theme::apply_config` to parse/alias them.

use std::collections::HashMap;

use fret_core::window::ColorScheme;
use fret_ui::theme::CubicBezier;
use fret_ui::{Theme, ThemeConfig, UiHost};
use fret_ui_kit::theme_tokens;
use serde::Deserialize;

fn seed_shadcn_motion_tokens(cfg: &mut ThemeConfig) {
    // Keep these values aligned with the motion workstream doc and the UI kit defaults.
    // The goal is to make "web-like" motion outcomes explicit and theme-tunable.

    cfg.durations_ms
        .entry("duration.shadcn.motion.100".to_string())
        .or_insert(100);
    cfg.durations_ms
        .entry("duration.shadcn.motion.150".to_string())
        .or_insert(150);
    cfg.durations_ms
        .entry("duration.shadcn.motion.200".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.shadcn.motion.300".to_string())
        .or_insert(300);
    cfg.durations_ms
        .entry("duration.shadcn.motion.500".to_string())
        .or_insert(500);

    // Semantic duration keys (preferred by recipes; numeric scale remains as fallback).
    cfg.durations_ms
        .entry("duration.shadcn.motion.overlay.open".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.shadcn.motion.overlay.close".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.shadcn.motion.sidebar.toggle".to_string())
        .or_insert(200);

    // Toast/Sonner: align with upstream `sonner` defaults.
    //
    // - Toast enter/stack transitions use 400ms `ease` by default.
    // - Toast unmount after exit uses 200ms (`TIME_BEFORE_UNMOUNT` in `sonner`).
    cfg.durations_ms
        .entry("duration.shadcn.motion.toast.enter".to_string())
        .or_insert(400);
    cfg.durations_ms
        .entry("duration.shadcn.motion.toast.exit".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.shadcn.motion.toast.stack.shift".to_string())
        .or_insert(400);
    // Upstream `sonner` does not stagger stack reflow; keep the default at 0ms.
    cfg.durations_ms
        .entry("duration.shadcn.motion.toast.stack.shift.stagger".to_string())
        .or_insert(0);

    // Drawer springs (duration + bounce) are read from theme tokens when present.
    cfg.durations_ms
        .entry("duration.shadcn.motion.spring.drawer.settle".to_string())
        .or_insert(240);
    cfg.numbers
        .entry("number.shadcn.motion.spring.drawer.settle.bounce".to_string())
        .or_insert(0.0);
    cfg.durations_ms
        .entry("duration.shadcn.motion.spring.drawer.inertia_bounce".to_string())
        .or_insert(240);
    cfg.numbers
        .entry("number.shadcn.motion.spring.drawer.inertia_bounce.bounce".to_string())
        .or_insert(0.25);

    let shadcn_ease = CubicBezier {
        x1: 0.22,
        y1: 1.0,
        x2: 0.36,
        y2: 1.0,
    };
    let linear = CubicBezier {
        x1: 0.0,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };
    cfg.easings
        .entry("easing.shadcn.motion".to_string())
        .or_insert(shadcn_ease);
    cfg.easings
        .entry("easing.shadcn.motion.overlay".to_string())
        .or_insert(shadcn_ease);
    cfg.easings
        .entry("easing.shadcn.motion.sidebar".to_string())
        .or_insert(linear);
    // CSS `ease` (used by upstream `sonner` toast transitions).
    let css_ease = CubicBezier {
        x1: 0.25,
        y1: 0.1,
        x2: 0.25,
        y2: 1.0,
    };
    cfg.easings
        .entry("easing.shadcn.motion.toast".to_string())
        .or_insert(css_ease);
    cfg.easings
        .entry("easing.shadcn.motion.toast.stack.shift".to_string())
        .or_insert(css_ease);

    // Canonical cross-ecosystem semantic motion keys (preferred for long-term authoring).
    //
    // shadcn recipes still prefer `*.shadcn.motion.*` keys first, but the UI kit supports these as
    // a fallback so other ecosystems can share a common vocabulary without importing shadcn
    // namespaces.
    cfg.durations_ms
        .entry("duration.motion.presence.enter".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.motion.presence.exit".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.motion.collapsible.toggle".to_string())
        .or_insert(200);
    cfg.durations_ms
        .entry("duration.motion.layout.expand".to_string())
        .or_insert(200);

    cfg.durations_ms
        .entry("duration.motion.spring.drag_release_settle".to_string())
        .or_insert(240);
    cfg.numbers
        .entry("number.motion.spring.drag_release_settle.bounce".to_string())
        .or_insert(0.25);

    cfg.easings
        .entry("easing.motion.standard".to_string())
        .or_insert(shadcn_ease);
    cfg.easings
        .entry("easing.motion.emphasized".to_string())
        .or_insert(shadcn_ease);
    cfg.easings
        .entry("easing.motion.layout.expand".to_string())
        .or_insert(shadcn_ease);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadcnColorScheme {
    Light,
    Dark,
}

impl ShadcnColorScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            ShadcnColorScheme::Light => "light",
            ShadcnColorScheme::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadcnBaseColor {
    Neutral,
    Zinc,
    Slate,
    Stone,
    Gray,
}

impl ShadcnBaseColor {
    pub const ALL: &'static [ShadcnBaseColor] = &[
        ShadcnBaseColor::Neutral,
        ShadcnBaseColor::Zinc,
        ShadcnBaseColor::Slate,
        ShadcnBaseColor::Stone,
        ShadcnBaseColor::Gray,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ShadcnBaseColor::Neutral => "neutral",
            ShadcnBaseColor::Zinc => "zinc",
            ShadcnBaseColor::Slate => "slate",
            ShadcnBaseColor::Stone => "stone",
            ShadcnBaseColor::Gray => "gray",
        }
    }
}

#[derive(Debug, Deserialize)]
struct ShadcnRegistryTheme {
    #[serde(rename = "cssVars")]
    css_vars: ShadcnCssVars,
}

#[derive(Debug, Deserialize)]
struct ShadcnCssVars {
    light: HashMap<String, String>,
    dark: HashMap<String, String>,
}

/// Load a shadcn v4 "new-york-v4" theme preset.
///
/// Theme source:
/// - Base palette: `repo-ref/ui/apps/v4/public/r/styles/new-york-v4/theme-*.json` (vendored).
/// - App default overrides: `repo-ref/ui/apps/v4/styles/globals.css` (the web golden harness uses
///   these as the effective runtime values).
pub fn shadcn_new_york_config(base: ShadcnBaseColor, scheme: ShadcnColorScheme) -> ThemeConfig {
    let raw = match base {
        ShadcnBaseColor::Neutral => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-neutral.json")
        }
        ShadcnBaseColor::Zinc => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-zinc.json")
        }
        ShadcnBaseColor::Slate => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-slate.json")
        }
        ShadcnBaseColor::Stone => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-stone.json")
        }
        ShadcnBaseColor::Gray => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-gray.json")
        }
    };

    let theme: ShadcnRegistryTheme =
        serde_json::from_str(raw).expect("vendored shadcn theme JSON is valid");

    let mut colors = match scheme {
        ShadcnColorScheme::Light => theme.css_vars.light,
        ShadcnColorScheme::Dark => theme.css_vars.dark,
    };

    // The upstream shadcn registry v4 theme JSONs focus on base palette tokens.
    // Some `*-foreground` variables are still relied upon by component recipes (e.g. Button,
    // Badge). Seed missing ones to match the upstream CSS defaults.
    colors
        .entry("destructive-foreground".to_string())
        .or_insert_with(|| match scheme {
            // Source: `repo-ref/ui/apps/v4/styles/globals.css`.
            ShadcnColorScheme::Light => "oklch(0.97 0.01 17)".to_string(),
            ShadcnColorScheme::Dark => "oklch(0.58 0.22 27)".to_string(),
        });

    // Menu rows use `data-[variant=destructive]:focus:bg-destructive/10` (and `/20` on dark) in the
    // upstream shadcn v4 recipes.
    if let Some(destructive) = colors.get("destructive").cloned() {
        // Button / Badge destructive background:
        // - light: `bg-destructive`
        // - dark: `dark:bg-destructive/60`
        let destructive_bg = match scheme {
            ShadcnColorScheme::Light => destructive.clone(),
            // Note: CSS alpha blending is not perceptually identical across renderers.
            // Our GPU pipeline blends in linear space, which can make `*/60` backgrounds appear
            // slightly brighter than upstream web screenshots. Nudge the derived token darker to
            // keep destructive chrome readable (white label + icons) in zinc dark.
            ShadcnColorScheme::Dark => with_oklch_alpha(&destructive, 0.3)
                .expect("shadcn new-york-v4 destructive token is oklch"),
        };
        colors.insert(
            "component.button.destructive.bg".to_string(),
            destructive_bg.clone(),
        );
        colors.insert("component.badge.destructive.bg".to_string(), destructive_bg);

        let alpha = match scheme {
            ShadcnColorScheme::Light => 0.1,
            ShadcnColorScheme::Dark => 0.2,
        };
        let destructive_focus_bg = with_oklch_alpha(&destructive, alpha)
            .expect("shadcn new-york-v4 destructive token is oklch");
        colors.insert(
            "component.menu.destructive_focus_bg".to_string(),
            destructive_focus_bg,
        );

        // shadcn new-york-v4 invalid control ring color:
        // - light: `destructive/20`
        // - dark: `destructive/40`
        let invalid_alpha = match scheme {
            ShadcnColorScheme::Light => 0.2,
            ShadcnColorScheme::Dark => 0.4,
        };
        let invalid_ring = with_oklch_alpha(&destructive, invalid_alpha)
            .expect("shadcn new-york-v4 destructive token is oklch");
        colors.insert("component.control.invalid_ring".to_string(), invalid_ring);
    }

    // shadcn new-york-v4 `TabsTrigger` inactive foreground:
    // - light: `text-foreground`
    // - dark: `text-muted-foreground`
    let tabs_inactive_key = match scheme {
        ShadcnColorScheme::Light => "foreground",
        ShadcnColorScheme::Dark => "muted-foreground",
    };
    if let Some(fg) = colors.get(tabs_inactive_key).cloned() {
        colors.insert("component.tabs.trigger.fg_inactive".to_string(), fg);
    }

    // shadcn new-york-v4 `TabsTrigger` active chrome:
    // - light: `data-[state=active]:bg-background` (border stays transparent)
    // - dark: `dark:data-[state=active]:bg-input/30 dark:data-[state=active]:border-input`
    if !colors.contains_key("component.tabs.trigger.bg_active") {
        let v = match scheme {
            ShadcnColorScheme::Light => colors.get("background").cloned(),
            ShadcnColorScheme::Dark => colors
                .get("input")
                .and_then(|input| with_oklch_alpha(input, 0.3))
                .or_else(|| colors.get("background").cloned()),
        };
        if let Some(v) = v {
            colors.insert("component.tabs.trigger.bg_active".to_string(), v);
        }
    }
    if !colors.contains_key("component.tabs.trigger.border_active") {
        let v = match scheme {
            ShadcnColorScheme::Light => Some("#00000000".to_string()),
            ShadcnColorScheme::Dark => colors
                .get("input")
                .cloned()
                .or_else(|| colors.get("border").cloned()),
        };
        if let Some(v) = v {
            colors.insert("component.tabs.trigger.border_active".to_string(), v);
        }
    }

    // shadcn new-york-v4 `Switch` unchecked track background:
    // - light: `data-[state=unchecked]:bg-input`
    // - dark: `dark:data-[state=unchecked]:bg-input/80`
    if !colors.contains_key("component.switch.track.bg_off") {
        let v = match scheme {
            ShadcnColorScheme::Light => colors.get("input").cloned(),
            ShadcnColorScheme::Dark => colors
                .get("input")
                .and_then(|input| with_oklch_alpha(input, 0.8))
                .or_else(|| colors.get("input").cloned()),
        };
        if let Some(v) = v {
            colors.insert("component.switch.track.bg_off".to_string(), v);
        }
    }

    // shadcn new-york-v4 `RadioGroup` choice-card checked background:
    // - light: `bg-primary/5`
    // - dark: `bg-primary/10`
    if let Some(primary) = colors.get("primary").cloned() {
        let alpha = match scheme {
            ShadcnColorScheme::Light => 0.05,
            ShadcnColorScheme::Dark => 0.10,
        };
        let checked_bg =
            with_oklch_alpha(&primary, alpha).expect("shadcn new-york-v4 primary token is oklch");
        colors.insert(
            "component.radio_group.choice_card.checked_bg".to_string(),
            checked_bg,
        );

        // Docking (in-tree) overlays use a small set of primary-derived alphas.
        // Keep them tokenized so presets can tune contrast without touching docking internals.
        //
        // Note: these are seeded from `primary` so all shadcn base colors inherit the correct hue.
        let mut seed_primary = |key: &str, alpha: f32| {
            if colors.contains_key(key) {
                return;
            }
            let v = with_oklch_alpha(&primary, alpha)
                .expect("shadcn new-york-v4 primary token is oklch");
            colors.insert(key.to_string(), v);
        };

        seed_primary("component.docking.drop_overlay.float.bg", 0.10);
        seed_primary("component.docking.drop_overlay.float.border", 0.85);
        seed_primary("component.docking.drop_overlay.empty.bg", 0.08);
        seed_primary("component.docking.drop_overlay.empty.border", 0.75);
        seed_primary("component.docking.drop_overlay.center.content.bg", 0.12);
        seed_primary("component.docking.drop_overlay.center.content.border", 0.65);
        seed_primary("component.docking.drop_overlay.center.tab_bar.bg", 0.14);
        seed_primary("component.docking.drop_overlay.center.tab_bar.border", 0.45);
        seed_primary("component.docking.drop_overlay.zone.bg", 0.16);
        seed_primary("component.docking.drop_overlay.zone.border", 0.85);

        seed_primary("component.docking.tab_insert.preview.bg", 0.22);
        seed_primary("component.docking.tab_insert.preview.border", 0.85);
        seed_primary("component.docking.tab_insert.marker.bg", 0.85);
        seed_primary("component.docking.tab_insert.marker.border", 1.0);
        seed_primary("component.docking.tab_insert.marker.cap.bg", 0.92);
    }

    // The upstream v4 registry theme JSONs do not fully match the values used by the upstream
    // web app's `styles/globals.css`. Our web-vs-fret goldens are generated from that app, so we
    // patch the delta here to keep the Rust runtime aligned.
    if base == ShadcnBaseColor::Neutral && scheme == ShadcnColorScheme::Dark {
        // Source: `repo-ref/ui/apps/v4/styles/globals.css`.
        colors.insert("popover".to_string(), "oklch(0.269 0 0)".to_string());
        colors.insert("accent".to_string(), "oklch(0.371 0 0)".to_string());
        colors.insert("sidebar-ring".to_string(), "oklch(0.439 0 0)".to_string());
    }

    // shadcn new-york-v4 `Skeleton`:
    // - `bg-accent`
    if !colors.contains_key("component.skeleton.bg")
        && let Some(accent) = colors.get("accent").cloned()
    {
        colors.insert("component.skeleton.bg".to_string(), accent);
    }

    // shadcn new-york-v4 `NavigationMenuTrigger` open background:
    // - `data-[state=open]:bg-accent/50`
    if !colors.contains_key("component.navigation_menu.trigger.bg_open")
        && let Some(accent) = colors.get("accent").cloned()
    {
        let v = with_oklch_alpha(&accent, 0.5).unwrap_or(accent);
        colors.insert("component.navigation_menu.trigger.bg_open".to_string(), v);
    }

    // Editor-like ecosystem surfaces (node graph, code editors) consume Fret viewport selection
    // tokens from the typed theme baseline.
    //
    // Seed them from shadcn's `ring` to avoid the default Fret blue selection when a shadcn theme
    // is installed.
    if let Some(ring) = colors.get("ring").cloned() {
        let mut seed_ring_alpha = |key: &str, alpha: f32| {
            if colors.contains_key(key) {
                return;
            }
            let v = with_oklch_alpha(&ring, alpha).unwrap_or_else(|| ring.clone());
            colors.insert(key.to_string(), v);
        };

        seed_ring_alpha("color.viewport.selection.fill", 0.16);
        seed_ring_alpha("color.viewport.selection.stroke", 0.80);

        if !colors.contains_key("color.viewport.marker") {
            colors.insert("color.viewport.marker".to_string(), ring);
        }
    }

    // `fret-chart` accepts shadcn `chart-*` tokens, but some retained paths use
    // `chart.palette.<n>` directly (tests and fixed-style overrides). Seed a small alias set so
    // chart demos do not fall back to the default theme when the palette keys are absent.
    for (idx, shadcn_key) in ["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"]
        .into_iter()
        .enumerate()
    {
        let key = format!("chart.palette.{idx}");
        if colors.contains_key(&key) {
            continue;
        }
        if let Some(v) = colors.get(shadcn_key).cloned() {
            colors.insert(key, v);
        }
    }

    // `fret-plot` historically reads `plot.palette.<n>` / `fret.plot.palette.<n>`.
    // Seed a small alias set from shadcn's chart tokens so plot demos match the same palette
    // baseline as `fret-chart` when a shadcn theme preset is installed.
    for (idx, shadcn_key) in ["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"]
        .into_iter()
        .enumerate()
    {
        if let Some(v) = colors.get(shadcn_key).cloned() {
            let key = format!("plot.palette.{idx}");
            colors.entry(key).or_insert_with(|| v.clone());
            let key = format!("fret.plot.palette.{idx}");
            colors.entry(key).or_insert_with(|| v.clone());
        }
    }

    // AI Elements + some shadcn recipes use `dark:hover:bg-accent/50` to soften hover in dark
    // schemes (e.g. attachment chips/rows). Our baseline theme defaults `color.menu.item.hover` to
    // `accent` when not explicitly configured, so we seed an explicit dark alpha here to keep
    // zinc/dark parity consistent.
    if scheme == ShadcnColorScheme::Dark
        && !colors.contains_key("color.menu.item.hover")
        && let Some(accent) = colors.get("accent").cloned()
    {
        let v = with_oklch_alpha(&accent, 0.5).unwrap_or(accent);
        colors.insert("color.menu.item.hover".to_string(), v);
    }

    let mut metrics: HashMap<String, f32> = HashMap::new();
    if let Some(radius) = colors.remove("radius") {
        if let Some(px) = parse_css_length_px(&radius) {
            // Match shadcn's default border-radius recipe:
            // lg = var(--radius), md = var(--radius) - 2px, sm = var(--radius) - 4px.
            metrics.insert("metric.radius.lg".to_string(), px);
            metrics.insert("metric.radius.md".to_string(), (px - 2.0).max(0.0));
            metrics.insert("metric.radius.sm".to_string(), (px - 4.0).max(0.0));
        }
    }

    // new-york-v4 component defaults are expressed as Tailwind classes in the upstream registry
    // (e.g. `h-9`, `px-3`, `gap-2`, `focus-visible:ring-[3px]`). Our component library consumes
    // theme metrics, so we seed those defaults here to reduce per-component drift.
    //
    // Note: keep this small and scoped to ergonomic, high-signal tokens; component-specific
    // deviations should live in the component implementations and audit doc.
    metrics
        .entry("metric.padding.sm".to_string())
        .or_insert(8.0);
    metrics
        .entry("metric.padding.md".to_string())
        .or_insert(10.0);

    // NavigationMenu:
    // Upstream `NavigationMenuViewport` uses a 0.375rem gap (`mt-1.5`) between the trigger row and
    // the viewport panel.
    metrics.insert(
        "component.navigation_menu.viewport.side_offset".to_string(),
        6.0,
    );

    // Component-specific overrides in the upstream registry.
    // - Checkbox uses `rounded-[4px]` (not `rounded-sm`, which would be `radius - 4px`).
    metrics
        .entry("component.checkbox.radius".to_string())
        .or_insert(4.0);
    metrics
        .entry("metric.font.size".to_string())
        .or_insert(14.0);
    metrics
        .entry("metric.font.line_height".to_string())
        .or_insert(20.0);

    // Default typography scales used across shadcn recipes (via fret-ui-kit helpers).
    // These are also accessed directly by some components (e.g. Calendar) via `metric_required`.
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_XS_PX.to_string())
        .or_insert(12.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT.to_string())
        .or_insert(16.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_SM_PX.to_string())
        .or_insert(14.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT.to_string())
        .or_insert(20.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_BASE_PX.to_string())
        .or_insert(15.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT.to_string())
        .or_insert(20.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_PROSE_PX.to_string())
        .or_insert(16.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_PROSE_LINE_HEIGHT.to_string())
        .or_insert(24.0);

    // Kbd (new-york-v4): `text-xs` inside a fixed `h-5` keycap.
    metrics
        .entry("component.kbd.text_px".to_string())
        .or_insert(12.0);
    metrics
        .entry("component.kbd.line_height".to_string())
        .or_insert(16.0);

    // Calendar (shadcn `Calendar` uses `h-8 w-8` day cells with `space-y-2` between week rows).
    metrics
        .entry("component.calendar.day_size".to_string())
        .or_insert(32.0);
    metrics
        .entry("component.calendar.week_row_gap".to_string())
        .or_insert(8.0);

    metrics
        .entry("component.ring.width".to_string())
        .or_insert(3.0);
    metrics
        .entry("component.ring.offset".to_string())
        .or_insert(0.0);

    metrics
        .entry("component.size.md.input.h".to_string())
        .or_insert(36.0);
    metrics
        .entry("component.size.md.input.px".to_string())
        .or_insert(12.0);
    metrics
        .entry("component.size.md.input.py".to_string())
        .or_insert(4.0);

    metrics
        .entry("component.size.md.button.h".to_string())
        .or_insert(36.0);
    metrics
        .entry("component.size.sm.button.h".to_string())
        .or_insert(32.0);
    metrics
        .entry("component.size.lg.button.h".to_string())
        .or_insert(40.0);
    metrics
        .entry("component.size.md.icon_button.size".to_string())
        .or_insert(36.0);
    metrics
        .entry("component.size.sm.icon_button.size".to_string())
        .or_insert(32.0);
    metrics
        .entry("component.size.lg.icon_button.size".to_string())
        .or_insert(40.0);

    // Legacy generic size tokens used by some components/tests.
    // Prefer `component.size.*` tokens in new code.
    metrics.entry("metric.size.sm".to_string()).or_insert(32.0);
    metrics.entry("metric.size.md".to_string()).or_insert(36.0);
    metrics.entry("metric.size.lg".to_string()).or_insert(40.0);

    // new-york-v4 `Slider` defaults:
    // - Track uses `h-1.5` (6px) via `data-[orientation=horizontal]:h-1.5`.
    // - Thumb uses `size-4` (16px).
    metrics
        .entry("component.slider.track_height".to_string())
        .or_insert(6.0);
    metrics
        .entry("component.slider.thumb_size".to_string())
        .or_insert(16.0);

    // new-york-v4 `Badge` defaults:
    // - `text-xs` (12px) with Tailwind default leading (16px).
    metrics
        .entry("component.badge.text_px".to_string())
        .or_insert(12.0);
    metrics
        .entry("component.badge.line_height".to_string())
        .or_insert(16.0);

    // new-york-v4 `Label` defaults:
    // - `text-sm` (14px) and `leading-none` (line-height = font-size).
    metrics
        .entry("component.label.text_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.label.line_height".to_string())
        .or_insert(14.0);

    // new-york-v4 `AlertDialog` defaults:
    // - Title uses `text-lg` (18px) and Tailwind default leading (28px).
    // - Description uses `text-sm` (14px) and Tailwind default leading (20px).
    metrics
        .entry("component.alert_dialog.title_px".to_string())
        .or_insert(18.0);
    metrics
        .entry("component.alert_dialog.title_line_height".to_string())
        .or_insert(28.0);
    metrics
        .entry("component.alert_dialog.description_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.alert_dialog.description_line_height".to_string())
        .or_insert(20.0);

    // new-york-v4 `Empty` defaults:
    // - Title uses `text-lg` (18px) and Tailwind default leading (28px).
    // - Description uses `text-sm/relaxed` (14px, 22.75px line-height).
    metrics.insert("component.empty.title_px".to_string(), 18.0);
    metrics.insert("component.empty.title_line_height".to_string(), 28.0);
    metrics.insert("component.empty.description_px".to_string(), 14.0);
    metrics.insert("component.empty.description_line_height".to_string(), 22.75);

    // new-york-v4 `Resizable` defaults:
    // - Handle uses `w-px` / `h-px` (1px layout gap), with a larger hit area.
    metrics
        .entry("component.resizable.gap".to_string())
        .or_insert(1.0);

    // new-york-v4 `Field` defaults:
    // - `FieldGroup` uses `gap-7` (28px).
    // - `FieldLabel` uses `text-sm` with `leading-snug` (14px * 1.375 = 19.25px).
    // - `FieldDescription` uses `text-sm` with `leading-normal` (14px * 1.5 = 21px).
    metrics
        .entry("component.field.group_gap".to_string())
        .or_insert(28.0);
    metrics
        .entry("component.field.label_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.field.label_line_height".to_string())
        .or_insert(19.25);
    metrics
        .entry("component.field.description_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.field.description_line_height".to_string())
        .or_insert(21.0);

    // Tooltip defaults in the upstream new-york-v4 registry:
    // - `TooltipContent` uses `sideOffset={0}` by default.
    // - Arrow uses `size-2.5` (10px).
    metrics
        .entry("component.tooltip.side_offset".to_string())
        .or_insert(0.0);
    metrics
        .entry("component.tooltip.arrow_size".to_string())
        .or_insert(10.0);

    if let Some(ring) = colors.get("ring").cloned() {
        if let Some(ring_50) = with_oklch_alpha(&ring, 0.5) {
            colors.insert("ring/50".to_string(), ring_50);
        }
    }
    if let Some(border) = colors.get("border").cloned() {
        if let Some(border_50) = with_oklch_alpha(&border, 0.5) {
            colors.insert("border/50".to_string(), border_50);
        }
    }
    if let Some(destructive) = colors.get("destructive").cloned() {
        if let Some(v) = with_oklch_alpha(&destructive, 0.1) {
            colors.insert("destructive/10".to_string(), v);
        }
        if let Some(v) = with_oklch_alpha(&destructive, 0.2) {
            colors.insert("destructive/20".to_string(), v);
        }
        if let Some(v) = with_oklch_alpha(&destructive, 0.4) {
            colors.insert("destructive/40".to_string(), v);
        }
    }

    // new-york-v4 `ScrollArea` uses `bg-border` for the thumb.
    if let Some(border) = colors.get("border").cloned() {
        colors.insert("scrollbar.thumb.background".to_string(), border.clone());
        colors.insert("scrollbar.thumb.hover.background".to_string(), border);
        colors
            .entry("scrollbar.background".to_string())
            .or_insert("#00000000".to_string());
    }
    match scheme {
        ShadcnColorScheme::Light => {
            // `bg-transparent` for inputs in light mode.
            colors.insert("component.input.bg".to_string(), "#00000000".to_string());
        }
        ShadcnColorScheme::Dark => {
            // `dark:bg-input/30` in the upstream Input component.
            if let Some(input) = colors.get("input").cloned() {
                colors.insert(
                    "component.input.bg".to_string(),
                    with_oklch_alpha(&input, 0.3).unwrap_or(input),
                );
            }
        }
    }

    // new-york-v4: select triggers (and native select) use `dark:hover:bg-input/50` while staying
    // `bg-transparent` in light mode.
    if !colors.contains_key("component.input.bg_hover") {
        let v = match scheme {
            ShadcnColorScheme::Light => Some("#00000000".to_string()),
            ShadcnColorScheme::Dark => colors
                .get("input")
                .and_then(|input| with_oklch_alpha(input, 0.5)),
        };
        if let Some(v) = v {
            colors.insert("component.input.bg_hover".to_string(), v);
        }
    }

    // Button (outline) dark deltas in shadcn new-york-v4:
    // - base: `bg-background border-border hover:bg-accent hover:text-accent-foreground`
    // - dark: `dark:bg-input/30 dark:border-input dark:hover:bg-input/50`
    if !colors.contains_key("component.button.outline.bg") {
        let v = match scheme {
            ShadcnColorScheme::Light => colors.get("background").cloned(),
            ShadcnColorScheme::Dark => colors.get("component.input.bg").cloned().or_else(|| {
                colors
                    .get("input")
                    .and_then(|input| with_oklch_alpha(input, 0.3))
            }),
        };
        if let Some(v) = v {
            colors.insert("component.button.outline.bg".to_string(), v);
        }
    }

    if !colors.contains_key("component.button.outline.bg_hover") {
        let v = match scheme {
            ShadcnColorScheme::Light => colors.get("accent").cloned(),
            ShadcnColorScheme::Dark => colors
                .get("input")
                .and_then(|input| with_oklch_alpha(input, 0.5))
                .or_else(|| colors.get("accent").cloned()),
        };
        if let Some(v) = v {
            colors.insert("component.button.outline.bg_hover".to_string(), v);
        }
    }

    if !colors.contains_key("component.button.outline.border") {
        let v = match scheme {
            ShadcnColorScheme::Light => colors.get("border").cloned(),
            ShadcnColorScheme::Dark => colors
                .get("input")
                .cloned()
                .or_else(|| colors.get("border").cloned()),
        };
        if let Some(v) = v {
            colors.insert("component.button.outline.border".to_string(), v);
        }
    }

    seed_workspace_shell_colors(&mut colors);
    seed_syntax_colors(&mut colors);

    let mut cfg = ThemeConfig {
        name: format!("shadcn/new-york-v4/{}/{}", base.as_str(), scheme.as_str()),
        author: Some("shadcn/ui".to_string()),
        url: Some("https://ui.shadcn.com".to_string()),
        color_scheme: Some(match scheme {
            ShadcnColorScheme::Light => ColorScheme::Light,
            ShadcnColorScheme::Dark => ColorScheme::Dark,
        }),
        colors,
        metrics,
        ..Default::default()
    };
    seed_shadcn_motion_tokens(&mut cfg);
    cfg
}

fn seed_syntax_colors(colors: &mut HashMap<String, String>) {
    fn pick(colors: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
        for &key in keys {
            if let Some(value) = colors.get(key) {
                return Some(value.clone());
            }
        }
        None
    }

    fn insert_if_missing(colors: &mut HashMap<String, String>, key: &str, value: Option<String>) {
        if colors.contains_key(key) {
            return;
        }
        if let Some(value) = value {
            colors.insert(key.to_string(), value);
        }
    }

    // These keys are consumed by `fret-code-view` for tree-sitter highlight tags (ADR 0099).
    // We derive a small palette from the base shadcn `chart-*` tokens.
    insert_if_missing(
        colors,
        "color.syntax.comment",
        pick(colors, &["muted-foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.keyword",
        pick(colors, &["chart-3", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.function",
        pick(colors, &["chart-1", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.type",
        pick(colors, &["chart-4", "accent-foreground", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.string",
        pick(colors, &["chart-2", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.constant",
        pick(colors, &["chart-5", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.number",
        pick(colors, &["chart-5", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.operator",
        pick(colors, &["muted-foreground", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.punctuation",
        pick(colors, &["muted-foreground", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.variable",
        pick(colors, &["foreground"]),
    );
}

fn seed_workspace_shell_colors(colors: &mut HashMap<String, String>) {
    fn pick(colors: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
        for &key in keys {
            if let Some(value) = colors.get(key) {
                return Some(value.clone());
            }
        }
        None
    }

    fn insert_if_missing(colors: &mut HashMap<String, String>, key: &str, value: Option<String>) {
        if colors.contains_key(key) {
            return;
        }
        if let Some(value) = value {
            colors.insert(key.to_string(), value);
        }
    }

    insert_if_missing(colors, "workspace.frame.bg", pick(colors, &["background"]));
    insert_if_missing(
        colors,
        "workspace.top_bar.bg",
        pick(colors, &["muted", "background"]),
    );
    insert_if_missing(
        colors,
        "workspace.top_bar.border",
        pick(colors, &["border"]),
    );
    insert_if_missing(
        colors,
        "workspace.status_bar.bg",
        pick(colors, &["muted", "background"]),
    );
    insert_if_missing(
        colors,
        "workspace.status_bar.border",
        pick(colors, &["border"]),
    );
    insert_if_missing(
        colors,
        "workspace.tabstrip.bg",
        pick(colors, &["muted", "background"]),
    );
    insert_if_missing(
        colors,
        "workspace.tabstrip.border",
        pick(colors, &["border"]),
    );
    insert_if_missing(
        colors,
        "workspace.tabstrip.scroll_fg",
        pick(colors, &["muted-foreground", "foreground"]),
    );
}

/// Apply a shadcn preset into the global `Theme`.
pub fn apply_shadcn_new_york<H: UiHost>(
    app: &mut H,
    base: ShadcnBaseColor,
    scheme: ShadcnColorScheme,
) {
    let cfg = shadcn_new_york_config(base, scheme);
    Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
}

fn parse_css_length_px(s: &str) -> Option<f32> {
    let s = s.trim();
    if let Some(rem) = s.strip_suffix("rem") {
        let v: f32 = rem.trim().parse().ok()?;
        return Some(v * 16.0);
    }
    if let Some(px) = s.strip_suffix("px") {
        let v: f32 = px.trim().parse().ok()?;
        return Some(v);
    }
    None
}

fn with_oklch_alpha(raw: &str, alpha: f32) -> Option<String> {
    let alpha = alpha.clamp(0.0, 1.0);
    let raw = raw.trim();
    let inner = raw.strip_prefix("oklch(")?.strip_suffix(')')?.trim();

    let (main, base_alpha) = if let Some((main, alpha_part)) = inner.split_once('/') {
        let alpha_part = alpha_part.trim();
        let base_alpha = if alpha_part.ends_with('%') {
            let pct: f32 = alpha_part.trim_end_matches('%').trim().parse().ok()?;
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            alpha_part.parse::<f32>().ok()?.clamp(0.0, 1.0)
        };
        (main.trim(), base_alpha)
    } else {
        (inner, 1.0)
    };

    // Tailwind-style opacity modifiers (e.g. `bg-input/30`) should multiply alpha when the base
    // token already includes one (shadcn v4 dark themes often do).
    let out_alpha = (base_alpha * alpha).clamp(0.0, 1.0);
    let pct = ((out_alpha * 1000.0).round() / 10.0).clamp(0.0, 100.0);
    if (pct.fract() - 0.0).abs() < f32::EPSILON {
        Some(format!("oklch({main} / {}%)", pct as u32))
    } else {
        Some(format!("oklch({main} / {pct:.1}%)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::window::ColorScheme;
    use fret_ui::Theme;

    #[test]
    fn new_york_v4_seeds_component_text_metrics() {
        for &base in ShadcnBaseColor::ALL {
            for scheme in [ShadcnColorScheme::Light, ShadcnColorScheme::Dark] {
                let cfg = shadcn_new_york_config(base, scheme);
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
                );
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
                );
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_BASE_PX)
                );
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT)
                );
            }
        }
    }

    #[test]
    fn new_york_v4_seeds_kbd_metrics() {
        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        assert_eq!(
            cfg.metrics.get("component.kbd.text_px").copied(),
            Some(12.0)
        );
        assert_eq!(
            cfg.metrics.get("component.kbd.line_height").copied(),
            Some(16.0)
        );

        let mut app = fret_app::App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let theme = Theme::global(&app);
        assert_eq!(
            theme.metric_by_key("component.kbd.text_px"),
            Some(fret_core::Px(12.0))
        );
        assert_eq!(
            theme.metric_by_key("component.kbd.line_height"),
            Some(fret_core::Px(16.0))
        );
    }

    #[test]
    fn new_york_v4_seeds_control_sizing_metrics() {
        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        assert_eq!(
            cfg.metrics.get("component.size.md.button.h").copied(),
            Some(36.0)
        );
        assert_eq!(
            cfg.metrics.get("component.size.sm.button.h").copied(),
            Some(32.0)
        );
        assert_eq!(
            cfg.metrics.get("component.size.lg.button.h").copied(),
            Some(40.0)
        );
        assert_eq!(
            cfg.metrics
                .get("component.size.md.icon_button.size")
                .copied(),
            Some(36.0)
        );
        assert_eq!(
            cfg.metrics
                .get("component.size.sm.icon_button.size")
                .copied(),
            Some(32.0)
        );
        assert_eq!(
            cfg.metrics
                .get("component.size.lg.icon_button.size")
                .copied(),
            Some(40.0)
        );

        let mut app = fret_app::App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let theme = Theme::global(&app);
        assert_eq!(
            theme.metric_by_key("component.size.md.icon_button.size"),
            Some(fret_core::Px(36.0))
        );
        assert_eq!(
            theme.metric_by_key("component.size.sm.button.h"),
            Some(fret_core::Px(32.0))
        );
    }

    #[test]
    fn new_york_v4_sets_color_scheme_metadata() {
        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        assert_eq!(cfg.color_scheme, Some(ColorScheme::Light));

        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Dark);
        assert_eq!(cfg.color_scheme, Some(ColorScheme::Dark));

        let mut app = fret_app::App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Dark);
        let theme = Theme::global(&app);
        assert_eq!(theme.color_scheme, Some(ColorScheme::Dark));
    }

    #[test]
    fn new_york_v4_seeds_component_variant_colors() {
        for &base in ShadcnBaseColor::ALL {
            let cfg_light = shadcn_new_york_config(base, ShadcnColorScheme::Light);
            let cfg_dark = shadcn_new_york_config(base, ShadcnColorScheme::Dark);

            for cfg in [&cfg_light, &cfg_dark] {
                for key in ["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"] {
                    assert!(
                        cfg.colors.contains_key(key),
                        "expected shadcn new-york-v4 preset to include `{key}`",
                    );
                }

                assert_eq!(
                    cfg.colors.get("color.syntax.comment").cloned(),
                    cfg.colors.get("muted-foreground").cloned(),
                    "expected syntax comment to match muted-foreground",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.keyword").cloned(),
                    cfg.colors.get("chart-3").cloned(),
                    "expected syntax keyword to be chart-3 derived",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.function").cloned(),
                    cfg.colors.get("chart-1").cloned(),
                    "expected syntax function to be chart-1 derived",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.type").cloned(),
                    cfg.colors.get("chart-4").cloned(),
                    "expected syntax type to be chart-4 derived",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.string").cloned(),
                    cfg.colors.get("chart-2").cloned(),
                    "expected syntax string to be chart-2 derived",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.constant").cloned(),
                    cfg.colors.get("chart-5").cloned(),
                    "expected syntax constant to be chart-5 derived",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.number").cloned(),
                    cfg.colors.get("chart-5").cloned(),
                    "expected syntax number to be chart-5 derived",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.operator").cloned(),
                    cfg.colors.get("muted-foreground").cloned(),
                    "expected syntax operator to match muted-foreground",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.punctuation").cloned(),
                    cfg.colors.get("muted-foreground").cloned(),
                    "expected syntax punctuation to match muted-foreground",
                );
                assert_eq!(
                    cfg.colors.get("color.syntax.variable").cloned(),
                    cfg.colors.get("foreground").cloned(),
                    "expected syntax variable to match foreground",
                );
            }

            let destructive_light = cfg_light
                .colors
                .get("destructive")
                .cloned()
                .expect("missing destructive");
            let destructive_dark = cfg_dark
                .colors
                .get("destructive")
                .cloned()
                .expect("missing destructive");

            assert_eq!(
                cfg_light
                    .colors
                    .get("component.button.destructive.bg")
                    .cloned(),
                Some(destructive_light.clone()),
                "expected destructive button bg to match destructive in light scheme"
            );
            assert_eq!(
                cfg_light
                    .colors
                    .get("component.badge.destructive.bg")
                    .cloned(),
                Some(destructive_light),
                "expected destructive badge bg to match destructive in light scheme"
            );

            let expected_destructive_dark_bg = with_oklch_alpha(&destructive_dark, 0.3)
                .expect("shadcn new-york-v4 destructive token is oklch");
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.button.destructive.bg")
                    .cloned(),
                Some(expected_destructive_dark_bg.clone()),
                "expected destructive button bg to match destructive-derived dark token"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.badge.destructive.bg")
                    .cloned(),
                Some(expected_destructive_dark_bg),
                "expected destructive badge bg to match destructive-derived dark token"
            );

            let outline_bg_light = cfg_light
                .colors
                .get("background")
                .cloned()
                .expect("missing background");
            let outline_border_light = cfg_light
                .colors
                .get("border")
                .cloned()
                .expect("missing border");
            let outline_bg_hover_light = cfg_light
                .colors
                .get("accent")
                .cloned()
                .expect("missing accent");
            assert_eq!(
                cfg_light.colors.get("component.skeleton.bg").cloned(),
                Some(outline_bg_hover_light.clone()),
                "expected skeleton bg to match accent in light scheme"
            );
            let ring_light = cfg_light.colors.get("ring").cloned().expect("missing ring");
            assert_eq!(
                cfg_light
                    .colors
                    .get("color.viewport.selection.fill")
                    .cloned(),
                Some(
                    with_oklch_alpha(&ring_light, 0.16)
                        .expect("shadcn new-york-v4 ring token is oklch")
                ),
                "expected viewport selection fill to be ring-derived in light scheme"
            );
            assert_eq!(
                cfg_light
                    .colors
                    .get("color.viewport.selection.stroke")
                    .cloned(),
                Some(
                    with_oklch_alpha(&ring_light, 0.80)
                        .expect("shadcn new-york-v4 ring token is oklch")
                ),
                "expected viewport selection stroke to be ring-derived in light scheme"
            );
            assert_eq!(
                cfg_light.colors.get("color.viewport.marker").cloned(),
                Some(ring_light),
                "expected viewport marker to match ring in light scheme"
            );
            for (idx, shadcn_key) in ["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"]
                .into_iter()
                .enumerate()
            {
                let palette_key = format!("chart.palette.{idx}");
                assert_eq!(
                    cfg_light.colors.get(&palette_key).cloned(),
                    cfg_light.colors.get(shadcn_key).cloned(),
                    "expected {palette_key} to match {shadcn_key} in light scheme"
                );
                let palette_key = format!("plot.palette.{idx}");
                assert_eq!(
                    cfg_light.colors.get(&palette_key).cloned(),
                    cfg_light.colors.get(shadcn_key).cloned(),
                    "expected {palette_key} to match {shadcn_key} in light scheme"
                );
                let palette_key = format!("fret.plot.palette.{idx}");
                assert_eq!(
                    cfg_light.colors.get(&palette_key).cloned(),
                    cfg_light.colors.get(shadcn_key).cloned(),
                    "expected {palette_key} to match {shadcn_key} in light scheme"
                );
            }
            assert_eq!(
                cfg_light.colors.get("component.button.outline.bg").cloned(),
                Some(outline_bg_light),
                "expected outline button bg to match background in light scheme"
            );
            assert_eq!(
                cfg_light
                    .colors
                    .get("component.button.outline.border")
                    .cloned(),
                Some(outline_border_light),
                "expected outline button border to match border in light scheme"
            );
            assert_eq!(
                cfg_light
                    .colors
                    .get("component.button.outline.bg_hover")
                    .cloned(),
                Some(outline_bg_hover_light),
                "expected outline button hover bg to match accent in light scheme"
            );

            let input_dark = cfg_dark
                .colors
                .get("input")
                .cloned()
                .expect("missing input");
            let outline_bg_dark = cfg_dark
                .colors
                .get("component.input.bg")
                .cloned()
                .expect("missing component.input.bg");
            let accent_dark = cfg_dark
                .colors
                .get("accent")
                .cloned()
                .expect("missing accent");
            let outline_bg_hover_dark = with_oklch_alpha(&input_dark, 0.5)
                .expect("shadcn new-york-v4 input token is oklch");
            assert_eq!(
                cfg_dark.colors.get("component.skeleton.bg").cloned(),
                Some(accent_dark),
                "expected skeleton bg to match accent in dark scheme"
            );
            let ring_dark = cfg_dark.colors.get("ring").cloned().expect("missing ring");
            assert_eq!(
                cfg_dark
                    .colors
                    .get("color.viewport.selection.fill")
                    .cloned(),
                Some(
                    with_oklch_alpha(&ring_dark, 0.16)
                        .expect("shadcn new-york-v4 ring token is oklch")
                ),
                "expected viewport selection fill to be ring-derived in dark scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("color.viewport.selection.stroke")
                    .cloned(),
                Some(
                    with_oklch_alpha(&ring_dark, 0.80)
                        .expect("shadcn new-york-v4 ring token is oklch")
                ),
                "expected viewport selection stroke to be ring-derived in dark scheme"
            );
            assert_eq!(
                cfg_dark.colors.get("color.viewport.marker").cloned(),
                Some(ring_dark),
                "expected viewport marker to match ring in dark scheme"
            );
            for (idx, shadcn_key) in ["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"]
                .into_iter()
                .enumerate()
            {
                let palette_key = format!("chart.palette.{idx}");
                assert_eq!(
                    cfg_dark.colors.get(&palette_key).cloned(),
                    cfg_dark.colors.get(shadcn_key).cloned(),
                    "expected {palette_key} to match {shadcn_key} in dark scheme"
                );
                let palette_key = format!("plot.palette.{idx}");
                assert_eq!(
                    cfg_dark.colors.get(&palette_key).cloned(),
                    cfg_dark.colors.get(shadcn_key).cloned(),
                    "expected {palette_key} to match {shadcn_key} in dark scheme"
                );
                let palette_key = format!("fret.plot.palette.{idx}");
                assert_eq!(
                    cfg_dark.colors.get(&palette_key).cloned(),
                    cfg_dark.colors.get(shadcn_key).cloned(),
                    "expected {palette_key} to match {shadcn_key} in dark scheme"
                );
            }
            assert_eq!(
                cfg_dark.colors.get("component.button.outline.bg").cloned(),
                Some(outline_bg_dark),
                "expected outline button bg to match input/30 in dark scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.button.outline.border")
                    .cloned(),
                Some(input_dark.clone()),
                "expected outline button border to match input in dark scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.button.outline.bg_hover")
                    .cloned(),
                Some(outline_bg_hover_dark),
                "expected outline button hover bg to match input/50 in dark scheme"
            );
            assert_eq!(
                cfg_dark.colors.get("component.input.bg_hover").cloned(),
                Some(
                    with_oklch_alpha(&input_dark, 0.5)
                        .expect("shadcn new-york-v4 input token is oklch",)
                ),
                "expected input hover bg to match input/50 in dark scheme"
            );

            assert_eq!(
                cfg_light
                    .colors
                    .get("component.control.invalid_ring")
                    .cloned(),
                cfg_light.colors.get("destructive/20").cloned(),
                "expected component.control.invalid_ring to match destructive/20"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.control.invalid_ring")
                    .cloned(),
                cfg_dark.colors.get("destructive/40").cloned(),
                "expected component.control.invalid_ring to match destructive/40"
            );

            assert_eq!(
                cfg_light
                    .colors
                    .get("component.tabs.trigger.fg_inactive")
                    .cloned(),
                cfg_light.colors.get("foreground").cloned(),
                "expected tabs inactive fg to match foreground in light scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.tabs.trigger.fg_inactive")
                    .cloned(),
                cfg_dark.colors.get("muted-foreground").cloned(),
                "expected tabs inactive fg to match muted-foreground in dark scheme"
            );

            assert_eq!(
                cfg_light
                    .colors
                    .get("component.tabs.trigger.bg_active")
                    .cloned(),
                cfg_light.colors.get("background").cloned(),
                "expected tabs active bg to match background in light scheme"
            );
            assert_eq!(
                cfg_light
                    .colors
                    .get("component.tabs.trigger.border_active")
                    .cloned(),
                Some("#00000000".to_string()),
                "expected tabs active border to remain transparent in light scheme"
            );
            let input_dark = cfg_dark
                .colors
                .get("input")
                .cloned()
                .expect("missing input");
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.tabs.trigger.border_active")
                    .cloned(),
                Some(input_dark.clone()),
                "expected tabs active border to match input in dark scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.tabs.trigger.bg_active")
                    .cloned(),
                Some(
                    with_oklch_alpha(&input_dark, 0.3)
                        .expect("shadcn new-york-v4 input token is oklch"),
                ),
                "expected tabs active bg to match input/30 in dark scheme"
            );

            assert_eq!(
                cfg_light
                    .colors
                    .get("component.switch.track.bg_off")
                    .cloned(),
                cfg_light.colors.get("input").cloned(),
                "expected switch off bg to match input in light scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.switch.track.bg_off")
                    .cloned(),
                Some(
                    with_oklch_alpha(&input_dark, 0.8)
                        .expect("shadcn new-york-v4 input token is oklch"),
                ),
                "expected switch off bg to match input/80 in dark scheme"
            );

            let accent_light = cfg_light
                .colors
                .get("accent")
                .cloned()
                .expect("missing accent");
            let accent_dark = cfg_dark
                .colors
                .get("accent")
                .cloned()
                .expect("missing accent");
            assert_eq!(
                cfg_light
                    .colors
                    .get("component.navigation_menu.trigger.bg_open")
                    .cloned(),
                Some(
                    with_oklch_alpha(&accent_light, 0.5)
                        .expect("shadcn new-york-v4 accent token is oklch"),
                ),
                "expected nav-menu open bg to match accent/50 in light scheme"
            );
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.navigation_menu.trigger.bg_open")
                    .cloned(),
                Some(
                    with_oklch_alpha(&accent_dark, 0.5)
                        .expect("shadcn new-york-v4 accent token is oklch"),
                ),
                "expected nav-menu open bg to match accent/50 in dark scheme"
            );

            let primary_light = cfg_light
                .colors
                .get("primary")
                .cloned()
                .expect("missing primary");
            let expected_light = with_oklch_alpha(&primary_light, 0.05)
                .expect("shadcn new-york-v4 primary token is oklch");
            assert_eq!(
                cfg_light
                    .colors
                    .get("component.radio_group.choice_card.checked_bg")
                    .cloned(),
                Some(expected_light),
                "expected choice-card checked bg to match primary/5 in light scheme"
            );

            let primary_dark = cfg_dark
                .colors
                .get("primary")
                .cloned()
                .expect("missing primary");
            let expected_dark = with_oklch_alpha(&primary_dark, 0.10)
                .expect("shadcn new-york-v4 primary token is oklch");
            assert_eq!(
                cfg_dark
                    .colors
                    .get("component.radio_group.choice_card.checked_bg")
                    .cloned(),
                Some(expected_dark),
                "expected choice-card checked bg to match primary/10 in dark scheme"
            );

            for &(key, alpha) in &[
                ("component.docking.drop_overlay.float.bg", 0.10),
                ("component.docking.drop_overlay.float.border", 0.85),
                ("component.docking.drop_overlay.empty.bg", 0.08),
                ("component.docking.drop_overlay.empty.border", 0.75),
                ("component.docking.drop_overlay.center.content.bg", 0.12),
                ("component.docking.drop_overlay.center.content.border", 0.65),
                ("component.docking.drop_overlay.center.tab_bar.bg", 0.14),
                ("component.docking.drop_overlay.center.tab_bar.border", 0.45),
                ("component.docking.drop_overlay.zone.bg", 0.16),
                ("component.docking.drop_overlay.zone.border", 0.85),
                ("component.docking.tab_insert.preview.bg", 0.22),
                ("component.docking.tab_insert.preview.border", 0.85),
                ("component.docking.tab_insert.marker.bg", 0.85),
                ("component.docking.tab_insert.marker.border", 1.0),
                ("component.docking.tab_insert.marker.cap.bg", 0.92),
            ] {
                let expected_light = with_oklch_alpha(&primary_light, alpha)
                    .expect("shadcn new-york-v4 primary token is oklch");
                assert_eq!(
                    cfg_light.colors.get(key).cloned(),
                    Some(expected_light),
                    "expected {key} to match primary-derived value in light scheme"
                );

                let expected_dark = with_oklch_alpha(&primary_dark, alpha)
                    .expect("shadcn new-york-v4 primary token is oklch");
                assert_eq!(
                    cfg_dark.colors.get(key).cloned(),
                    Some(expected_dark),
                    "expected {key} to match primary-derived value in dark scheme"
                );
            }
        }
    }

    #[test]
    fn new_york_v4_seeds_menu_item_hover_in_dark_scheme() {
        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Dark);
        let accent = cfg.colors.get("accent").cloned().expect("missing accent");
        let expected = with_oklch_alpha(&accent, 0.5).expect("accent token is oklch");
        assert_eq!(
            cfg.colors.get("color.menu.item.hover").cloned(),
            Some(expected)
        );
    }

    #[test]
    fn new_york_v4_seeds_navigation_menu_viewport_gap_metric() {
        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        assert_eq!(
            cfg.metrics
                .get("component.navigation_menu.viewport.side_offset")
                .copied(),
            Some(6.0)
        );

        let mut app = fret_app::App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let theme = Theme::global(&app);
        assert_eq!(
            theme.metric_by_key("component.navigation_menu.viewport.side_offset"),
            Some(fret_core::Px(6.0))
        );
    }

    #[test]
    fn new_york_v4_seeds_workspace_shell_tokens() {
        for &base in ShadcnBaseColor::ALL {
            for scheme in [ShadcnColorScheme::Light, ShadcnColorScheme::Dark] {
                let cfg = shadcn_new_york_config(base, scheme);

                assert_eq!(
                    cfg.colors.get("workspace.frame.bg").cloned(),
                    cfg.colors.get("background").cloned(),
                    "expected workspace.frame.bg to alias background for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.top_bar.bg").cloned(),
                    cfg.colors.get("muted").cloned(),
                    "expected workspace.top_bar.bg to alias muted for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.top_bar.border").cloned(),
                    cfg.colors.get("border").cloned(),
                    "expected workspace.top_bar.border to alias border for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.status_bar.bg").cloned(),
                    cfg.colors.get("muted").cloned(),
                    "expected workspace.status_bar.bg to alias muted for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.status_bar.border").cloned(),
                    cfg.colors.get("border").cloned(),
                    "expected workspace.status_bar.border to alias border for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.tabstrip.bg").cloned(),
                    cfg.colors.get("muted").cloned(),
                    "expected workspace.tabstrip.bg to alias muted for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.tabstrip.border").cloned(),
                    cfg.colors.get("border").cloned(),
                    "expected workspace.tabstrip.border to alias border for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    cfg.colors.get("workspace.tabstrip.scroll_fg").cloned(),
                    cfg.colors.get("muted-foreground").cloned(),
                    "expected workspace.tabstrip.scroll_fg to alias muted-foreground for {base:?}/{scheme:?}"
                );

                let mut app = fret_app::App::new();
                apply_shadcn_new_york(&mut app, base, scheme);
                let theme = Theme::global(&app);
                assert_eq!(
                    theme.color_by_key("workspace.frame.bg"),
                    theme.color_by_key("background"),
                    "expected applied workspace.frame.bg to resolve for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    theme.color_by_key("workspace.top_bar.bg"),
                    theme.color_by_key("muted"),
                    "expected applied workspace.top_bar.bg to resolve for {base:?}/{scheme:?}"
                );
                assert_eq!(
                    theme.color_by_key("workspace.tabstrip.scroll_fg"),
                    theme.color_by_key("muted-foreground"),
                    "expected applied workspace.tabstrip.scroll_fg to resolve for {base:?}/{scheme:?}"
                );
            }
        }
    }

    #[test]
    fn new_york_v4_seeds_canonical_motion_tokens() {
        let cfg = shadcn_new_york_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        assert_eq!(
            cfg.durations_ms
                .get("duration.motion.presence.enter")
                .copied(),
            Some(200)
        );
        assert_eq!(
            cfg.durations_ms
                .get("duration.motion.presence.exit")
                .copied(),
            Some(200)
        );
        assert_eq!(
            cfg.durations_ms
                .get("duration.motion.collapsible.toggle")
                .copied(),
            Some(200)
        );
        assert_eq!(
            cfg.durations_ms
                .get("duration.motion.layout.expand")
                .copied(),
            Some(200)
        );

        assert_eq!(
            cfg.durations_ms
                .get("duration.motion.spring.drag_release_settle")
                .copied(),
            Some(240)
        );
        assert_eq!(
            cfg.numbers
                .get("number.motion.spring.drag_release_settle.bounce")
                .copied(),
            Some(0.25)
        );

        assert!(cfg.easings.contains_key("easing.motion.standard"));
        assert!(cfg.easings.contains_key("easing.motion.emphasized"));
    }
}
