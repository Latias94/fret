//! Curated Fret overlay tokens layered on top of the generated Material Web v30 baseline.
//!
//! This Module owns Fret-specific markers/aliases plus hand-authored upstream backfills that should
//! stay reviewable without editing the generated `material_web_v30` baseline.

use fret_core::{Corners, Px};
use fret_ui::theme::ThemeConfig;

pub(crate) fn inject_system_layout_defaults(cfg: &mut ThemeConfig) {
    // Compose `minimumInteractiveComponentSize()` default (48dp).
    cfg.metrics
        .insert("md.sys.layout.minimum-touch-target.size".to_string(), 48.0);

    // Fret-owned layout direction marker (0 = LTR, 1 = RTL).
    //
    // This is not a Material Web token. It represents app-level directionality and can be
    // overridden at the theme or subtree level.
    cfg.numbers
        .entry("md.sys.fret.layout.is-rtl".to_string())
        .or_insert(0.0);
}

pub(crate) fn inject_dynamic_variant_marker(cfg: &mut ThemeConfig, is_expressive: bool) {
    // Fret-owned marker token: allow Material3 components to switch to expressive component token
    // variants when the dynamic scheme uses the expressive palette variant.
    cfg.numbers.insert(
        "md.sys.fret.material.is-expressive".to_string(),
        if is_expressive { 1.0 } else { 0.0 },
    );
}

pub(crate) fn inject_expressive_motion_tokens(cfg: &mut ThemeConfig) {
    // Compose baseline: ExpressiveMotionTokens (v0_14_0).
    //
    // We keep these as Fret-owned tokens so:
    // - The component layer can converge on a stable `MotionScheme` API.
    // - Downstream apps can override values at the theme level.
    //
    // Material Web v30 currently provides only a single `md.sys.motion.spring.*` set.
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.default.spatial.damping".to_string(),
        0.8,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.default.spatial.stiffness".to_string(),
        380.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.fast.spatial.damping".to_string(),
        0.6,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.fast.spatial.stiffness".to_string(),
        800.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.slow.spatial.damping".to_string(),
        0.8,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.slow.spatial.stiffness".to_string(),
        200.0,
    );

    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.default.effects.damping".to_string(),
        1.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.default.effects.stiffness".to_string(),
        1600.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.fast.effects.damping".to_string(),
        1.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.fast.effects.stiffness".to_string(),
        3800.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.slow.effects.damping".to_string(),
        1.0,
    );
    cfg.numbers.insert(
        "md.sys.fret.material.motion.spring.slow.effects.stiffness".to_string(),
        800.0,
    );
}

pub(crate) fn override_navigation_drawer_scrim_opacity(cfg: &mut ThemeConfig) {
    // Material Web v30 notes that the navigation drawer scrim tokens are deprecated and do not
    // represent the intended M3 defaults. Prefer Neutral-Variant10 at 50% opacity for scrims.
    cfg.numbers
        .insert("md.comp.navigation-drawer.scrim.opacity".to_string(), 0.5);
}

pub(crate) fn inject_button_scalar_aliases(cfg: &mut ThemeConfig) {
    // Material Web omits explicit zero-elevation scalars for non-elevated button variants.
    // Fret keeps variant-shaped keys so token callers do not need to know that omission.
    for key in [
        "md.comp.button.outlined.container.elevation",
        "md.comp.button.outlined.disabled.container.elevation",
        "md.comp.button.outlined.focused.container.elevation",
        "md.comp.button.outlined.hovered.container.elevation",
        "md.comp.button.outlined.pressed.container.elevation",
        "md.comp.button.text.container.elevation",
        "md.comp.button.text.disabled.container.elevation",
        "md.comp.button.text.focused.container.elevation",
        "md.comp.button.text.hovered.container.elevation",
        "md.comp.button.text.pressed.container.elevation",
    ] {
        cfg.metrics.entry(key.to_string()).or_insert(0.0);
    }
}

pub(crate) fn inject_radio_button_scalar_backfills(cfg: &mut ThemeConfig) {
    // Material Web v30 emits the 40dp state layer size but omits the matching full state-layer
    // shape that other circular controls expose.
    cfg.corners
        .entry("md.comp.radio-button.state-layer.shape".to_string())
        .or_insert(Corners::all(Px(9999.0)));
}

pub(crate) fn inject_date_picker_docked_scalars(cfg: &mut ThemeConfig) {
    cfg.metrics
        .entry("md.sys.fret.material.date-picker.calendar.horizontal-padding".to_string())
        .or_insert(12.0);
}

pub(crate) fn inject_date_picker_modal_scalars(cfg: &mut ThemeConfig) {
    cfg.numbers
        .entry("md.sys.fret.material.date-picker.modal.scrim.opacity".to_string())
        .or_insert(0.32);
}

pub(crate) fn inject_time_picker_scalars(cfg: &mut ThemeConfig) {
    cfg.metrics
        .entry("md.sys.fret.material.time-picker.display-separator.width".to_string())
        .or_insert(24.0);
    cfg.numbers
        .entry("md.sys.fret.material.time-picker.scrim.opacity".to_string())
        .or_insert(0.32);
}

pub(crate) fn inject_primary_navigation_tab_scalars(cfg: &mut ThemeConfig) {
    cfg.metrics
        .entry(
            "md.comp.primary-navigation-tab.with-stacked-icon-and-label-text.container.height"
                .to_string(),
        )
        .or_insert(72.0);
    cfg.metrics
        .entry("md.comp.primary-navigation-tab.active-indicator.min-width".to_string())
        .or_insert(24.0);
    cfg.metrics
        .entry("md.comp.primary-navigation-tab.scrollable.edge-padding".to_string())
        .or_insert(52.0);
    cfg.metrics
        .entry("md.comp.primary-navigation-tab.scrollable.min-tab-width".to_string())
        .or_insert(90.0);
}

pub(crate) fn inject_secondary_navigation_tab_scalars(cfg: &mut ThemeConfig) {
    // Source: Compose Material3 `SecondaryNavigationTabTokens` and `TabRowDefaults`.
    cfg.metrics
        .entry("md.comp.secondary-navigation-tab.container.height".to_string())
        .or_insert(48.0);
    cfg.metrics
        .entry("md.comp.secondary-navigation-tab.divider.height".to_string())
        .or_insert(1.0);
    cfg.metrics
        .entry("md.comp.secondary-navigation-tab.scrollable.edge-padding".to_string())
        .or_insert(52.0);
    cfg.metrics
        .entry("md.comp.secondary-navigation-tab.scrollable.min-tab-width".to_string())
        .or_insert(90.0);
    cfg.metrics
        .entry("md.comp.secondary-navigation-tab.with-icon.icon.size".to_string())
        .or_insert(24.0);
    cfg.metrics
        .entry(
            "md.comp.secondary-navigation-tab.with-stacked-icon-and-label-text.container.height"
                .to_string(),
        )
        .or_insert(72.0);
    cfg.numbers
        .entry("md.comp.secondary-navigation-tab.with-label-text.label-text.weight".to_string())
        .or_insert(500.0);

    for key in [
        "md.comp.secondary-navigation-tab.active.focus.state-layer.opacity",
        "md.comp.secondary-navigation-tab.active.hover.state-layer.opacity",
        "md.comp.secondary-navigation-tab.active.pressed.state-layer.opacity",
        "md.comp.secondary-navigation-tab.inactive.focus.state-layer.opacity",
        "md.comp.secondary-navigation-tab.inactive.hover.state-layer.opacity",
        "md.comp.secondary-navigation-tab.inactive.pressed.state-layer.opacity",
    ] {
        let sys_key = if key.contains(".focus.") {
            "md.sys.state.focus.state-layer-opacity"
        } else if key.contains(".hover.") {
            "md.sys.state.hover.state-layer-opacity"
        } else {
            "md.sys.state.pressed.state-layer-opacity"
        };
        if let Some(value) = cfg.numbers.get(sys_key).copied() {
            cfg.numbers.entry(key.to_string()).or_insert(value);
        }
    }
}

pub(crate) fn inject_navigation_bar_scalars(cfg: &mut ThemeConfig) {
    for (key, value) in [
        ("md.comp.navigation-bar.active-indicator.top-offset", 12.0),
        ("md.comp.navigation-bar.item.gap", 8.0),
    ] {
        cfg.metrics.entry(key.to_string()).or_insert(value);
    }
}

pub(crate) fn inject_navigation_rail_scalars(cfg: &mut ThemeConfig) {
    for (key, value) in [
        ("md.comp.navigation-rail.item.width", 80.0),
        ("md.comp.navigation-rail.item.height", 56.0),
    ] {
        cfg.metrics.entry(key.to_string()).or_insert(value);
    }
}

pub(crate) fn inject_menu_scalars(cfg: &mut ThemeConfig) {
    for (key, value) in [
        ("md.comp.menu.container.max-height", 320.0),
        ("md.comp.menu.container.vertical-padding", 8.0),
        ("md.comp.menu.list-item.container.max-width", 280.0),
        ("md.comp.menu.list-item.container.min-width", 112.0),
        ("md.comp.menu.list-item.content.gap", 12.0),
        ("md.comp.menu.list-item.content.horizontal-padding", 12.0),
        ("md.comp.menu.list-item.icon.size", 24.0),
        ("md.comp.menu.list-item.leading-icon.size", 24.0),
        ("md.comp.menu.list-item.leading-icon.trailing-space", 12.0),
        (
            "md.sys.fret.material.selectable-menu-item.content-horizontal-padding",
            12.0,
        ),
        (
            "md.sys.fret.material.selectable-menu-item.icon-text-gap",
            8.0,
        ),
        (
            "md.sys.fret.material.selectable-menu-item.outer-horizontal-padding",
            4.0,
        ),
        (
            "md.sys.fret.material.selectable-menu-item.outer-vertical-padding",
            0.0,
        ),
        (
            "md.sys.fret.material.selectable-menu-item.with-secondary.outer-vertical-padding",
            2.0,
        ),
        ("md.sys.fret.material.dropdown-menu.collision-padding", 8.0),
        (
            "md.sys.fret.material.dropdown-menu.divider-margin-total",
            8.0,
        ),
        (
            "md.comp.menu.list-item.supporting-text.container.height",
            64.0,
        ),
        ("md.comp.menu.list-item.two-line-container.height", 64.0),
        ("md.comp.menu.section-label.container.height", 32.0),
    ] {
        cfg.metrics.entry(key.to_string()).or_insert(value);
    }

    for (key, value) in [
        ("md.comp.menu.list-item.disabled.leading-icon.opacity", 0.38),
        (
            "md.comp.menu.list-item.disabled.supporting-text.opacity",
            0.38,
        ),
        (
            "md.comp.menu.list-item.disabled.trailing-text.opacity",
            0.38,
        ),
        ("md.comp.menu.list-item.supporting-text.weight", 400.0),
        ("md.comp.menu.list-item.trailing-text.weight", 500.0),
        ("md.comp.menu.section-label.label-text.weight", 500.0),
    ] {
        cfg.numbers.entry(key.to_string()).or_insert(value);
    }

    cfg.corners
        .entry("md.comp.menu.list-item.container.shape".to_string())
        .or_insert(Corners::all(Px(4.0)));
    cfg.corners
        .entry("md.comp.menu.list-item.selected.container.shape".to_string())
        .or_insert(Corners::all(Px(12.0)));
}

pub(crate) fn inject_sheet_bottom_scalars(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-sheet-bottom.scss

    cfg.metrics.insert(
        "md.comp.sheet.bottom.docked.drag-handle.height".to_string(),
        4.0,
    );
    cfg.metrics.insert(
        "md.comp.sheet.bottom.docked.drag-handle.width".to_string(),
        32.0,
    );
    cfg.numbers.insert(
        "md.comp.sheet.bottom.docked.drag-handle.opacity".to_string(),
        0.4,
    );

    cfg.corners.insert(
        "md.comp.sheet.bottom.docked.container.shape".to_string(),
        Corners {
            top_left: Px(28.0),
            top_right: Px(28.0),
            bottom_right: Px(0.0),
            bottom_left: Px(0.0),
        },
    );
    cfg.corners.insert(
        "md.comp.sheet.bottom.docked.minimized.container.shape".to_string(),
        Corners::all(Px(0.0)),
    );

    // Both modal and standard use level1 in Material Web v30.
    cfg.metrics.insert(
        "md.comp.sheet.bottom.docked.modal.container.elevation".to_string(),
        1.0,
    );
    cfg.metrics.insert(
        "md.comp.sheet.bottom.docked.standard.container.elevation".to_string(),
        1.0,
    );

    cfg.metrics.insert(
        "md.comp.sheet.bottom.focus.indicator.outline.offset".to_string(),
        2.0,
    );
    cfg.metrics.insert(
        "md.comp.sheet.bottom.focus.indicator.thickness".to_string(),
        3.0,
    );

    // Material guidance defaults around ~0.32 for modal scrims.
    cfg.numbers
        .entry("md.sys.fret.material.sheet.bottom.docked.modal.scrim.opacity".to_string())
        .or_insert(0.32);
}

pub(crate) fn inject_search_bar_scalars(cfg: &mut ThemeConfig) {
    cfg.metrics
        .entry("md.sys.fret.material.search-bar.container.min-width".to_string())
        .or_insert(360.0);
    cfg.metrics
        .entry("md.sys.fret.material.search-bar.container.max-width".to_string())
        .or_insert(720.0);
}

pub(crate) fn inject_dialog_scalars(cfg: &mut ThemeConfig) {
    // Material guidance defaults around ~0.32 for modal scrims.
    cfg.numbers
        .entry("md.sys.fret.material.dialog.scrim.opacity".to_string())
        .or_insert(0.32);
}
