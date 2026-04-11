use fret_core::{Edges, Px};
use fret_ui::{ElementContextAccess, GlobalElementId, Invalidation, UiHost};

use crate::declarative::{
    container_queries, occlusion_queries, pointer_queries, safe_area_queries, viewport_queries,
    ContainerQueryHysteresis, ViewportQueryHysteresis,
};

/// Explicit query-source selector for adaptive policy surfaces.
///
/// This type exists so higher-level recipe/component APIs can name whether a responsive branch is
/// driven by a local container or by the window/device environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveQuerySource {
    Container,
    Viewport,
}

/// Coarse device-shell classification derived from viewport/device policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeviceAdaptiveClass {
    #[default]
    Compact,
    Regular,
    Expanded,
}

/// Coarse panel/container classification derived from container-query policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelAdaptiveClass {
    #[default]
    Compact,
    Medium,
    Wide,
}

/// Shared policy for device-shell classification.
#[derive(Debug, Clone, Copy)]
pub struct DeviceAdaptivePolicy {
    pub regular_min_width: Px,
    pub expanded_min_width: Px,
    pub hysteresis: ViewportQueryHysteresis,
    pub default_can_hover_when_unknown: bool,
    pub default_coarse_pointer_when_unknown: bool,
}

impl Default for DeviceAdaptivePolicy {
    fn default() -> Self {
        Self {
            regular_min_width: viewport_queries::tailwind::SM,
            expanded_min_width: viewport_queries::tailwind::XL,
            hysteresis: ViewportQueryHysteresis::default(),
            default_can_hover_when_unknown: true,
            default_coarse_pointer_when_unknown: false,
        }
    }
}

impl DeviceAdaptivePolicy {
    pub fn regular_min_width(mut self, width: Px) -> Self {
        self.regular_min_width = width;
        self
    }

    pub fn expanded_min_width(mut self, width: Px) -> Self {
        self.expanded_min_width = width;
        self
    }

    pub fn hysteresis(mut self, hysteresis: ViewportQueryHysteresis) -> Self {
        self.hysteresis = hysteresis;
        self
    }

    pub fn default_can_hover_when_unknown(mut self, value: bool) -> Self {
        self.default_can_hover_when_unknown = value;
        self
    }

    pub fn default_coarse_pointer_when_unknown(mut self, value: bool) -> Self {
        self.default_coarse_pointer_when_unknown = value;
        self
    }
}

/// Binary device-shell branch result for explicit desktop/mobile shell authoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceShellMode {
    Desktop,
    Mobile,
}

impl DeviceShellMode {
    pub fn is_desktop(self) -> bool {
        matches!(self, Self::Desktop)
    }

    pub fn is_mobile(self) -> bool {
        matches!(self, Self::Mobile)
    }
}

/// Shared policy for binary desktop/mobile shell switching.
#[derive(Debug, Clone, Copy)]
pub struct DeviceShellSwitchPolicy {
    pub desktop_min_width: Px,
    pub hysteresis: ViewportQueryHysteresis,
}

impl Default for DeviceShellSwitchPolicy {
    fn default() -> Self {
        Self {
            desktop_min_width: viewport_queries::tailwind::MD,
            hysteresis: ViewportQueryHysteresis::default(),
        }
    }
}

impl DeviceShellSwitchPolicy {
    pub fn desktop_min_width(mut self, width: Px) -> Self {
        self.desktop_min_width = width;
        self
    }

    pub fn hysteresis(mut self, hysteresis: ViewportQueryHysteresis) -> Self {
        self.hysteresis = hysteresis;
        self
    }
}

/// Shared policy for panel/container classification.
#[derive(Debug, Clone, Copy)]
pub struct PanelAdaptivePolicy {
    pub medium_min_width: Px,
    pub wide_min_width: Px,
    pub hysteresis: ContainerQueryHysteresis,
}

impl Default for PanelAdaptivePolicy {
    fn default() -> Self {
        Self {
            medium_min_width: container_queries::tailwind::MD,
            wide_min_width: container_queries::tailwind::XL,
            hysteresis: ContainerQueryHysteresis::default(),
        }
    }
}

impl PanelAdaptivePolicy {
    pub fn medium_min_width(mut self, width: Px) -> Self {
        self.medium_min_width = width;
        self
    }

    pub fn wide_min_width(mut self, width: Px) -> Self {
        self.wide_min_width = width;
        self
    }

    pub fn hysteresis(mut self, hysteresis: ContainerQueryHysteresis) -> Self {
        self.hysteresis = hysteresis;
        self
    }
}

/// Snapshot of the most common device-shell adaptive signals for the current window.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DeviceAdaptiveSnapshot {
    pub class: DeviceAdaptiveClass,
    pub can_hover: bool,
    pub coarse_pointer: bool,
    pub safe_area_insets: Edges,
    pub occlusion_insets: Edges,
}

/// Resolves whether the current window should use a desktop or mobile shell branch.
#[track_caller]
pub fn device_shell_mode<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    invalidation: Invalidation,
    policy: DeviceShellSwitchPolicy,
) -> DeviceShellMode
where
    Cx: ElementContextAccess<'a, H>,
{
    if viewport_queries::viewport_width_at_least(
        cx,
        invalidation,
        policy.desktop_min_width,
        policy.hysteresis,
    ) {
        DeviceShellMode::Desktop
    } else {
        DeviceShellMode::Mobile
    }
}

/// Lands exactly one explicit desktop/mobile shell branch without widening generic children APIs.
#[track_caller]
pub fn device_shell_switch<'a, H, Cx, DesktopBranch, MobileBranch, DesktopChild, MobileChild>(
    cx: &mut Cx,
    invalidation: Invalidation,
    policy: DeviceShellSwitchPolicy,
    desktop: DesktopBranch,
    mobile: MobileBranch,
) -> fret_ui::element::AnyElement
where
    H: UiHost + 'a,
    Cx: ElementContextAccess<'a, H>,
    DesktopBranch: FnOnce(&mut Cx) -> DesktopChild,
    MobileBranch: FnOnce(&mut Cx) -> MobileChild,
    DesktopChild: crate::ui_builder::IntoUiElement<H>,
    MobileChild: crate::ui_builder::IntoUiElement<H>,
{
    match device_shell_mode(cx, invalidation, policy) {
        DeviceShellMode::Desktop => {
            let child = desktop(cx);
            crate::land_child(cx, child)
        }
        DeviceShellMode::Mobile => {
            let child = mobile(cx);
            crate::land_child(cx, child)
        }
    }
}

fn normalize_device_thresholds(policy: DeviceAdaptivePolicy) -> (Px, Px) {
    if policy.regular_min_width.0 <= policy.expanded_min_width.0 {
        (policy.regular_min_width, policy.expanded_min_width)
    } else {
        (policy.expanded_min_width, policy.regular_min_width)
    }
}

fn normalize_panel_thresholds(policy: PanelAdaptivePolicy) -> (Px, Px) {
    if policy.medium_min_width.0 <= policy.wide_min_width.0 {
        (policy.medium_min_width, policy.wide_min_width)
    } else {
        (policy.wide_min_width, policy.medium_min_width)
    }
}

/// Resolves a coarse device-shell class from viewport width using explicit policy.
///
/// This is intended for device-shell decisions such as mobile/desktop branch coordination.
#[track_caller]
pub fn device_adaptive_class<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    invalidation: Invalidation,
    policy: DeviceAdaptivePolicy,
) -> DeviceAdaptiveClass
where
    Cx: ElementContextAccess<'a, H>,
{
    let (regular_min_width, expanded_min_width) = normalize_device_thresholds(policy);
    viewport_queries::viewport_breakpoints(
        cx.elements(),
        invalidation,
        DeviceAdaptiveClass::Compact,
        &[
            (regular_min_width, DeviceAdaptiveClass::Regular),
            (expanded_min_width, DeviceAdaptiveClass::Expanded),
        ],
        policy.hysteresis,
    )
}

/// Resolves a coarse panel/container class from a container-query region using explicit policy.
#[track_caller]
pub fn panel_adaptive_class<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    region: GlobalElementId,
    invalidation: Invalidation,
    default_when_unknown: PanelAdaptiveClass,
    policy: PanelAdaptivePolicy,
) -> PanelAdaptiveClass
where
    Cx: ElementContextAccess<'a, H>,
{
    let (medium_min_width, wide_min_width) = normalize_panel_thresholds(policy);
    container_queries::container_breakpoints(
        cx.elements(),
        region,
        invalidation,
        default_when_unknown,
        &[
            (medium_min_width, PanelAdaptiveClass::Medium),
            (wide_min_width, PanelAdaptiveClass::Wide),
        ],
        policy.hysteresis,
    )
}

/// Returns a bundle of common device-shell adaptive signals for the current window.
#[track_caller]
pub fn device_adaptive_snapshot<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    invalidation: Invalidation,
    policy: DeviceAdaptivePolicy,
) -> DeviceAdaptiveSnapshot
where
    Cx: ElementContextAccess<'a, H>,
{
    let class = device_adaptive_class(cx, invalidation, policy);
    let cx = cx.elements();
    DeviceAdaptiveSnapshot {
        class,
        can_hover: pointer_queries::primary_pointer_can_hover(
            cx,
            invalidation,
            policy.default_can_hover_when_unknown,
        ),
        coarse_pointer: pointer_queries::primary_pointer_is_coarse(
            cx,
            invalidation,
            policy.default_coarse_pointer_when_unknown,
        ),
        safe_area_insets: safe_area_queries::safe_area_insets_or_zero(cx, invalidation),
        occlusion_insets: occlusion_queries::occlusion_insets_or_zero(cx, invalidation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_policy_normalizes_threshold_order() {
        let policy = DeviceAdaptivePolicy::default()
            .regular_min_width(Px(1280.0))
            .expanded_min_width(Px(640.0));
        let (regular, expanded) = normalize_device_thresholds(policy);
        assert_eq!(regular, Px(640.0));
        assert_eq!(expanded, Px(1280.0));
    }

    #[test]
    fn panel_policy_normalizes_threshold_order() {
        let policy = PanelAdaptivePolicy::default()
            .medium_min_width(Px(1280.0))
            .wide_min_width(Px(768.0));
        let (medium, wide) = normalize_panel_thresholds(policy);
        assert_eq!(medium, Px(768.0));
        assert_eq!(wide, Px(1280.0));
    }

    #[test]
    fn device_shell_switch_policy_defaults_to_md_breakpoint() {
        let policy = DeviceShellSwitchPolicy::default();
        assert_eq!(policy.desktop_min_width, viewport_queries::tailwind::MD);
        assert_eq!(policy.hysteresis.up, Px(8.0));
        assert_eq!(policy.hysteresis.down, Px(8.0));
    }

    #[test]
    fn device_shell_mode_bool_helpers_match_variants() {
        assert!(DeviceShellMode::Desktop.is_desktop());
        assert!(!DeviceShellMode::Desktop.is_mobile());
        assert!(DeviceShellMode::Mobile.is_mobile());
        assert!(!DeviceShellMode::Mobile.is_desktop());
    }
}
