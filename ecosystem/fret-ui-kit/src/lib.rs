#![deny(deprecated)]
//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).
//!
//! Note: This crate is declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface.

/// Build a heterogeneous `Vec<AnyElement>` without repetitive child landing boilerplate.
///
/// Intended for ergonomic authoring inside layout builders, including host-bound late builders,
/// e.g.: `ui::h_flex(|cx| ui::children![cx; Button::new("OK").ui(), cx.text("...")])`.
#[macro_export]
macro_rules! children {
    ($cx:ident;) => {
        ::std::vec::Vec::new()
    };
    ($cx:ident; $($child:expr),+ $(,)?) => {{
        let mut children = ::std::vec::Vec::new();
        $(
            {
                let child = $child;
                let element = $crate::IntoUiElement::into_element(child, &mut *$cx);
                children.push(element);
            }
        )+
        children
    }};
}

/// Land typed child values at the last possible moment inside wrapper-style helpers.
pub(crate) fn collect_children<H, I>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    children: I,
) -> Vec<fret_ui::element::AnyElement>
where
    H: fret_ui::UiHost,
    I: IntoIterator,
    I::Item: crate::ui_builder::IntoUiElement<H>,
{
    let mut out = Vec::new();
    for child in children {
        out.push(crate::ui_builder::IntoUiElement::into_element(child, cx));
    }
    out
}

/// Implement the `UiBuilder` patch + render glue for a component that supports both chrome and
/// layout refinements.
///
/// The type must provide:
/// - `fn refine_style(self, ChromeRefinement) -> Self`
/// - `fn refine_layout(self, LayoutRefinement) -> Self`
/// - `fn into_element(self, &mut ElementContext<'_, H>) -> AnyElement`
#[macro_export]
macro_rules! ui_component_chrome_layout {
    ($ty:ty) => {
        impl $crate::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: $crate::UiPatch) -> Self {
                self.refine_style(patch.chrome).refine_layout(patch.layout)
            }
        }

        impl $crate::UiSupportsChrome for $ty {}
        impl $crate::UiSupportsLayout for $ty {}

        impl<H: ::fret_ui::UiHost> $crate::IntoUiElement<H> for $ty {
            #[track_caller]
            fn into_element(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                <$ty>::into_element(self, cx)
            }
        }
    };
}

/// Implement the `UiBuilder` patch + render glue for a component that supports layout refinements
/// only.
///
/// The type must provide:
/// - `fn refine_layout(self, LayoutRefinement) -> Self`
/// - `fn into_element(self, &mut ElementContext<'_, H>) -> AnyElement`
#[macro_export]
macro_rules! ui_component_layout_only {
    ($ty:ty) => {
        impl $crate::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: $crate::UiPatch) -> Self {
                self.refine_layout(patch.layout)
            }
        }

        impl $crate::UiSupportsLayout for $ty {}

        impl<H: ::fret_ui::UiHost> $crate::IntoUiElement<H> for $ty {
            #[track_caller]
            fn into_element(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                <$ty>::into_element(self, cx)
            }
        }
    };
}

/// Implement `UiPatchTarget` + `UiSupports*` for a component that supports both chrome and layout
/// refinements, without implementing rendering glue.
#[macro_export]
macro_rules! ui_component_chrome_layout_patch_only {
    ($ty:ty) => {
        impl $crate::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: $crate::UiPatch) -> Self {
                self.refine_style(patch.chrome).refine_layout(patch.layout)
            }
        }

        impl $crate::UiSupportsChrome for $ty {}
        impl $crate::UiSupportsLayout for $ty {}
    };
}

/// Implement `UiPatchTarget` + `UiSupportsLayout` for a component that supports layout refinements
/// only, without implementing rendering glue.
#[macro_export]
macro_rules! ui_component_layout_only_patch_only {
    ($ty:ty) => {
        impl $crate::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: $crate::UiPatch) -> Self {
                self.refine_layout(patch.layout)
            }
        }

        impl $crate::UiSupportsLayout for $ty {}
    };
}

/// Implement patch + render glue for a component that does not accept any `UiPatch`, but still
/// wants to opt into the `.ui()` surface (e.g. purely cosmetic elements).
#[macro_export]
macro_rules! ui_component_passthrough {
    ($ty:ty) => {
        impl $crate::UiPatchTarget for $ty {
            fn apply_ui_patch(self, _patch: $crate::UiPatch) -> Self {
                self
            }
        }

        impl<H: ::fret_ui::UiHost> $crate::IntoUiElement<H> for $ty {
            #[track_caller]
            fn into_element(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                <$ty>::into_element(self, cx)
            }
        }
    };
}

/// Implement `UiPatchTarget` for a component that does not accept any `UiPatch`, without
/// implementing rendering glue.
#[macro_export]
macro_rules! ui_component_passthrough_patch_only {
    ($ty:ty) => {
        impl $crate::UiPatchTarget for $ty {
            fn apply_ui_patch(self, _patch: $crate::UiPatch) -> Self {
                self
            }
        }
    };
}

/// Implement internal landing glue for a stateless component authored as `RenderOnce` (ADR 0039).
///
/// Note: we intentionally avoid a blanket impl due to coherence restrictions on upstream types.
#[macro_export]
macro_rules! ui_component_render_once {
    ($ty:ty) => {
        impl<H: ::fret_ui::UiHost> $crate::IntoUiElement<H> for $ty {
            #[track_caller]
            fn into_element(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                ::fret_ui::element::RenderOnce::render_once(self, cx)
            }
        }
    };
}

pub mod activate;
pub mod colors;
pub mod command;
mod corners4;
pub mod custom_effects;
pub mod declarative;
#[cfg(feature = "dnd")]
pub mod dnd;
mod edges4;
pub mod headless;
pub mod image_metadata;
pub mod image_sampling;
#[cfg(feature = "imui")]
pub mod imui;
pub mod node_graph;
pub mod overlay;
pub mod overlay_controller;
pub mod primitives;
pub mod recipes;
pub mod theme_tokens;
pub mod tooltip_provider;
pub mod tree;
pub mod typography;
pub mod ui;
pub mod ui_builder;
pub mod viewport_tooling;
#[cfg(feature = "unstable-internals")]
pub mod window_overlays;
#[cfg(not(feature = "unstable-internals"))]
mod window_overlays;

mod ui_builder_impls;

mod sizing;
mod style;
mod styled;

pub use activate::{
    on_activate, on_activate_notify, on_activate_request_redraw, on_activate_request_redraw_notify,
};
pub use corners4::Corners4;
pub use edges4::{Edges4, MarginEdge};
pub use image_metadata::{ImageMetadata, ImageMetadataStore, with_image_metadata_store_mut};
pub use image_sampling::ImageSamplingExt;
pub use sizing::{Sizable, Size};
pub use style::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, LengthRefinement,
    MetricRef, OverflowRefinement, OverrideSlot, PaddingRefinement, Radius, ShadowPreset,
    SignedMetricRef, Space, WidgetState, WidgetStateProperty, WidgetStates, merge_override_slot,
    merge_slot, resolve_override_slot, resolve_override_slot_opt, resolve_override_slot_opt_with,
    resolve_override_slot_with, resolve_slot,
};
pub use styled::{RefineStyle, Stylable, Styled, StyledExt};
pub use ui_builder::{
    IntoUiElement, UiBuilder, UiExt, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
};

pub use overlay_controller::{
    OverlayArbitrationSnapshot, OverlayController, OverlayKind, OverlayPresence, OverlayRequest,
    OverlayStackEntryKind, ToastLayerSpec, WindowOverlayStackEntry, WindowOverlayStackSnapshot,
};
pub use window_overlays::{
    DEFAULT_MAX_TOASTS, DEFAULT_TOAST_DURATION, DEFAULT_VISIBLE_TOASTS, ToastAction, ToastAsyncMsg,
    ToastAsyncQueueHandle, ToastButtonStyle, ToastDescription, ToastDuration, ToastIconButtonStyle,
    ToastIconOverride, ToastIconOverrides, ToastId, ToastLayerStyle, ToastOffset, ToastPosition,
    ToastRequest, ToastStore, ToastSwipeConfig, ToastSwipeDirection, ToastSwipeDirections,
    ToastTextStyle, ToastVariant, ToastVariantColors, ToastVariantPalette, toast_async_queue,
};

pub use window_overlays::TOAST_VIEWPORT_FOCUS_COMMAND;
pub use window_overlays::TOAST_VIEWPORT_RESTORE_COMMAND;

// Diagnostics-only exports: used by `fret-bootstrap` to export bundle.json fields.
#[doc(hidden)]
pub use window_overlays::{
    OverlaySynthesisEvent, OverlaySynthesisKind, OverlaySynthesisOutcome, OverlaySynthesisSource,
    WindowOverlaySynthesisDiagnosticsStore,
};

/// Common imports for component/app code using `fret-ui-kit`.
///
/// Recommended: `use fret_ui_kit::prelude::*;`
pub mod prelude {
    pub use crate::IntoUiElement as _;
    pub use crate::command::ElementCommandGatingExt as _;
    pub use crate::declarative::prelude::*;
    pub use crate::declarative::style;
    pub use crate::declarative::{CachedSubtreeExt, CachedSubtreeProps};
    pub use crate::ui;
    pub use crate::ui::UiElementSinkExt as _;
    pub use crate::{
        on_activate, on_activate_notify, on_activate_request_redraw,
        on_activate_request_redraw_notify,
    };

    #[cfg(feature = "imui")]
    pub use crate::imui::UiWriterUiKitExt as _;

    #[cfg(feature = "imui")]
    pub use crate::imui::UiWriterImUiFacadeExt as _;

    #[cfg(feature = "icons")]
    pub use crate::declarative::icon;
    #[cfg(feature = "icons")]
    pub use fret_icons::IconId;

    pub use crate::{
        ChromeRefinement, ColorFallback, ColorRef, Corners4, Edges4, ImageMetadata,
        ImageMetadataStore, ImageSamplingExt, IntoUiElement, LayoutRefinement, MarginEdge,
        MetricRef, OverrideSlot, Radius, ShadowPreset, SignedMetricRef, Size, Space, StyledExt,
        UiExt, WidgetState, WidgetStateProperty, WidgetStates, merge_override_slot, merge_slot,
        resolve_override_slot, resolve_override_slot_opt, resolve_override_slot_opt_with,
        resolve_override_slot_with, resolve_slot,
    };
    pub use crate::{OverlayArbitrationSnapshot, OverlayController, OverlayKind, OverlayPresence};
    pub use crate::{OverlayRequest, OverlayStackEntryKind};
    pub use crate::{WindowOverlayStackEntry, WindowOverlayStackSnapshot};

    pub use fret_core::scene::ImageSamplingHint;
    pub use fret_core::{AppWindowId, Px, TextOverflow, TextWrap, UiServices};
    pub use fret_runtime::{ActionId, CommandId, Model, TypedAction};
    pub use fret_ui::element::{AnyElement, AnyElementIterExt as _, TextProps};
    pub use fret_ui::{ElementContext, Invalidation, Theme, UiHost, UiTree};
}

/// Attempts to handle a window-scoped command that targets `fret-ui-kit` overlay substrates.
///
/// This is intended to be called by app drivers after `UiTree::dispatch_command` returns `false`.
pub fn try_handle_window_overlays_command<H: fret_ui::UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    app: &mut H,
    window: fret_core::AppWindowId,
    command: &fret_runtime::CommandId,
) -> bool {
    window_overlays::try_handle_window_command(ui, app, window, command)
}

pub use tree::{
    TreeEntry, TreeItem, TreeItemId, TreeRowRenderer, TreeRowState, TreeState, flatten_tree,
};

#[cfg(test)]
mod ui_component_macro_tests {
    use super::*;
    use fret_ui::element::TextProps;

    #[derive(Debug, Clone, Copy)]
    struct DummyComponent;

    impl DummyComponent {
        fn refine_style(self, _chrome: ChromeRefinement) -> Self {
            self
        }

        fn refine_layout(self, _layout: LayoutRefinement) -> Self {
            self
        }

        fn into_element<H: fret_ui::UiHost>(
            self,
            cx: &mut fret_ui::ElementContext<'_, H>,
        ) -> fret_ui::element::AnyElement {
            cx.text_props(TextProps::new("dummy"))
        }
    }

    ui_component_chrome_layout!(DummyComponent);

    #[test]
    fn ui_component_chrome_layout_macro_compiles() {
        fn assert_traits<
            T: UiPatchTarget + UiSupportsChrome + UiSupportsLayout + IntoUiElement<fret_app::App>,
        >() {
        }
        assert_traits::<DummyComponent>();
    }

    #[derive(Debug, Clone, Copy)]
    struct DummyRenderOnceComponent;

    impl fret_ui::element::RenderOnce for DummyRenderOnceComponent {
        fn render_once<H: fret_ui::UiHost>(
            self,
            cx: &mut fret_ui::ElementContext<'_, H>,
        ) -> fret_ui::element::AnyElement {
            cx.text_props(TextProps::new("dummy-render-once"))
        }
    }

    ui_component_render_once!(DummyRenderOnceComponent);

    #[test]
    fn ui_component_render_once_macro_compiles() {
        fn assert_into_element<T: IntoUiElement<fret_app::App>>() {}
        assert_into_element::<DummyRenderOnceComponent>();
    }
}

#[cfg(test)]
mod default_semantics_tests {
    #[test]
    fn text_box_presets_have_expected_wrap_defaults() {
        let sm = crate::ui::TextBox::new("sm", crate::ui::TextPreset::Sm);
        assert_eq!(sm.wrap, fret_core::TextWrap::Word);
        assert_eq!(sm.overflow, fret_core::TextOverflow::Clip);

        let label = crate::ui::TextBox::new("label", crate::ui::TextPreset::Label);
        assert_eq!(label.wrap, fret_core::TextWrap::None);
        assert_eq!(label.overflow, fret_core::TextOverflow::Clip);
    }
}

#[cfg(test)]
mod source_policy_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const README: &str = include_str!("../README.md");
    const DECLARATIVE_BLOOM_RS: &str = include_str!("declarative/bloom.rs");
    const DECLARATIVE_CACHED_SUBTREE_RS: &str = include_str!("declarative/cached_subtree.rs");
    const DECLARATIVE_CHROME_RS: &str = include_str!("declarative/chrome.rs");
    const DECLARATIVE_CONTAINER_QUERIES_RS: &str = include_str!("declarative/container_queries.rs");
    const DECLARATIVE_DISMISSIBLE_RS: &str = include_str!("declarative/dismissible.rs");
    const DECLARATIVE_GLASS_RS: &str = include_str!("declarative/glass.rs");
    const DECLARATIVE_LIST_RS: &str = include_str!("declarative/list.rs");
    const DECLARATIVE_MOD_RS: &str = include_str!("declarative/mod.rs");
    const DECLARATIVE_MODEL_WATCH_RS: &str = include_str!("declarative/model_watch.rs");
    const DECLARATIVE_PRELUDE_RS: &str = include_str!("declarative/prelude.rs");
    const DECLARATIVE_SCROLL_RS: &str = include_str!("declarative/scroll.rs");
    const DECLARATIVE_SEMANTICS_RS: &str = include_str!("declarative/semantics.rs");
    const DECLARATIVE_TABLE_RS: &str = include_str!("declarative/table.rs");
    const DECLARATIVE_VISUALLY_HIDDEN_RS: &str = include_str!("declarative/visually_hidden.rs");
    const DECLARATIVE_PIXELATE_RS: &str = include_str!("declarative/pixelate.rs");
    const IMUI_RS: &str = include_str!("imui.rs");
    const PRIMITIVES_DISMISSABLE_LAYER_RS: &str = include_str!("primitives/dismissable_layer.rs");
    const PRIMITIVES_ALERT_DIALOG_RS: &str = include_str!("primitives/alert_dialog.rs");
    const PRIMITIVES_DIALOG_RS: &str = include_str!("primitives/dialog.rs");
    const PRIMITIVES_FOCUS_SCOPE_RS: &str = include_str!("primitives/focus_scope.rs");
    const PRIMITIVES_ACCORDION_RS: &str = include_str!("primitives/accordion.rs");
    const PRIMITIVES_MENU_CONTENT_PANEL_RS: &str = include_str!("primitives/menu/content_panel.rs");
    const PRIMITIVES_MENU_CONTENT_RS: &str = include_str!("primitives/menu/content.rs");
    const PRIMITIVES_MENU_SUB_CONTENT_RS: &str = include_str!("primitives/menu/sub_content.rs");
    const PRIMITIVES_POPPER_CONTENT_RS: &str = include_str!("primitives/popper_content.rs");
    const PRIMITIVES_POPOVER_RS: &str = include_str!("primitives/popover.rs");
    const PRIMITIVES_ROVING_FOCUS_GROUP_RS: &str = include_str!("primitives/roving_focus_group.rs");
    const PRIMITIVES_SELECT_RS: &str = include_str!("primitives/select.rs");
    const PRIMITIVES_TABS_RS: &str = include_str!("primitives/tabs.rs");
    const PRIMITIVES_TOGGLE_RS: &str = include_str!("primitives/toggle.rs");
    const PRIMITIVES_TOOLBAR_RS: &str = include_str!("primitives/toolbar.rs");
    const PRIMITIVES_TOOLTIP_RS: &str = include_str!("primitives/tooltip.rs");
    const RECIPES_SORTABLE_DND_RS: &str = include_str!("recipes/sortable_dnd.rs");
    const UI_RS: &str = include_str!("ui.rs");
    const UI_BUILDER_RS: &str = include_str!("ui_builder.rs");

    fn visit_rust_files(dir: &std::path::Path, f: &mut impl FnMut(&std::path::Path, &str)) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|err| {
            panic!(
                "failed to read source-policy directory {}: {err}",
                dir.display()
            )
        }) {
            let entry = entry.unwrap_or_else(|err| {
                panic!(
                    "failed to read source-policy entry in {}: {err}",
                    dir.display()
                )
            });
            let path = entry.path();
            if path.is_dir() {
                visit_rust_files(&path, f);
                continue;
            }
            if path.extension().and_then(std::ffi::OsStr::to_str) != Some("rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!(
                    "failed to read source-policy file {}: {err}",
                    path.display()
                )
            });
            f(&path, &source);
        }
    }

    #[test]
    fn root_surface_omits_host_bound_conversion_alias() {
        let tests_start = LIB_RS.find("#[cfg(test)]").unwrap_or(LIB_RS.len());
        let public_surface = &LIB_RS[..tests_start];
        assert!(!public_surface.contains("UiHostBoundIntoElement"));
    }

    #[test]
    fn root_surface_omits_legacy_conversion_exports() {
        let tests_start = LIB_RS.find("#[cfg(test)]").unwrap_or(LIB_RS.len());
        let public_surface = &LIB_RS[..tests_start];
        assert!(!public_surface.contains("pub use ui::UiChildIntoElement;"));
        assert!(!public_surface.contains("pub use ui_builder::UiIntoElement;"));
        assert!(!public_surface.contains("pub(crate) use ui_builder::UiIntoElement;"));

        let export_start = public_surface
            .find("pub use ui_builder::{")
            .expect("ui_builder export block should exist");
        let export_tail = &public_surface[export_start..];
        let export_end = export_tail
            .find("};")
            .expect("ui_builder export block should terminate");
        let export_block = &export_tail[..export_end];
        assert!(!export_block.contains("UiIntoElement"));
        assert!(export_block.contains("IntoUiElement"));
    }

    #[test]
    fn legacy_ui_into_element_bridge_name_is_deleted_from_ui_builder() {
        assert!(!UI_BUILDER_RS.contains("trait UiIntoElement"));
        assert!(!UI_BUILDER_RS.contains("T: UiIntoElement"));
        assert!(!UI_BUILDER_RS.contains("UiIntoElement::into_element"));
        assert!(UI_BUILDER_RS.contains("impl<H: UiHost> IntoUiElement<H> for AnyElement"));
    }

    #[test]
    fn exported_component_macros_attach_public_conversion_trait_directly() {
        let tests_start = LIB_RS.find("#[cfg(test)]").unwrap_or(LIB_RS.len());
        let public_surface = &LIB_RS[..tests_start];
        assert!(!public_surface.contains("impl $crate::ui_builder::UiIntoElement for $ty"));
        assert!(
            public_surface.contains("impl<H: ::fret_ui::UiHost> $crate::IntoUiElement<H> for $ty")
        );
        assert!(public_surface.contains("macro_rules! ui_component_render_once"));
        assert!(!public_surface.contains("macro_rules! ui_into_element_render_once"));
    }

    #[test]
    fn child_pipeline_stays_on_unified_component_conversion_trait() {
        assert!(!UI_RS.contains("trait UiChildIntoElement"));
        assert!(!IMUI_RS.contains("UiChildIntoElement"));
        assert!(UI_RS.contains("I::Item: IntoUiElement<H>"));
        assert!(UI_RS.contains("IntoUiElement::into_element(child, cx)"));
        assert!(IMUI_RS.contains("B: IntoUiElement<H>"));
    }

    #[test]
    fn ui_builtin_text_primitives_land_through_public_conversion_trait() {
        let tests_start = UI_RS.find("#[cfg(test)]").unwrap_or(UI_RS.len());
        let public_surface = &UI_RS[..tests_start];

        assert!(
            !public_surface.contains("UiIntoElement"),
            "ui.rs production surface should not depend on UiIntoElement"
        );
        assert!(public_surface.contains("IntoUiElement<H> for TextBox"));
        assert!(public_surface.contains("IntoUiElement<H> for RawTextBox"));
    }

    #[test]
    fn declarative_semantics_ext_names_drop_legacy_conversion_prefix() {
        for (label, source) in [
            ("declarative/mod.rs", DECLARATIVE_MOD_RS),
            ("declarative/prelude.rs", DECLARATIVE_PRELUDE_RS),
        ] {
            assert!(
                source.contains("UiElementA11yExt"),
                "{label} should export UiElementA11yExt"
            );
            assert!(
                source.contains("UiElementKeyContextExt"),
                "{label} should export UiElementKeyContextExt"
            );
            assert!(
                source.contains("UiElementTestIdExt"),
                "{label} should export UiElementTestIdExt"
            );
            assert!(
                !source.contains("UiIntoElementA11yExt"),
                "{label} reintroduced UiIntoElementA11yExt"
            );
            assert!(
                !source.contains("UiIntoElementKeyContextExt"),
                "{label} reintroduced UiIntoElementKeyContextExt"
            );
            assert!(
                !source.contains("UiIntoElementTestIdExt"),
                "{label} reintroduced UiIntoElementTestIdExt"
            );
        }
    }

    #[test]
    fn declarative_semantics_wrappers_land_through_public_conversion_trait() {
        let tests_start = DECLARATIVE_SEMANTICS_RS
            .find("#[cfg(test)]")
            .unwrap_or(DECLARATIVE_SEMANTICS_RS.len());
        let public_surface = &DECLARATIVE_SEMANTICS_RS[..tests_start];

        assert!(
            !public_surface.contains("UiIntoElement"),
            "declarative/semantics.rs production surface should not depend on UiIntoElement"
        );
        assert!(public_surface.contains("IntoUiElement<H> for UiElementWithTestId<T>"));
        assert!(public_surface.contains("IntoUiElement<H> for UiElementWithA11y<T>"));
        assert!(public_surface.contains("IntoUiElement<H> for UiElementWithKeyContext<T>"));
        assert!(public_surface.contains("pub trait UiElementTestIdExt: Sized"));
        assert!(public_surface.contains("pub trait UiElementA11yExt: Sized"));
        assert!(public_surface.contains("pub trait UiElementKeyContextExt: Sized"));
    }

    #[test]
    fn wrapper_helpers_prefer_typed_child_inputs() {
        for (label, source, landing_snippet) in [
            (
                "declarative/bloom.rs",
                DECLARATIVE_BLOOM_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/cached_subtree.rs",
                DECLARATIVE_CACHED_SUBTREE_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/chrome.rs",
                DECLARATIVE_CHROME_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/container_queries.rs",
                DECLARATIVE_CONTAINER_QUERIES_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/dismissible.rs",
                DECLARATIVE_DISMISSIBLE_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/glass.rs",
                DECLARATIVE_GLASS_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/list.rs",
                DECLARATIVE_LIST_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/pixelate.rs",
                DECLARATIVE_PIXELATE_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/scroll.rs",
                DECLARATIVE_SCROLL_RS,
                "collect_children(cx,",
            ),
            (
                "declarative/table.rs",
                DECLARATIVE_TABLE_RS,
                "collect_children(",
            ),
            (
                "declarative/visually_hidden.rs",
                DECLARATIVE_VISUALLY_HIDDEN_RS,
                "collect_children(cx,",
            ),
            (
                "primitives/accordion.rs",
                PRIMITIVES_ACCORDION_RS,
                "collect_children(cx, items)",
            ),
            (
                "primitives/dismissable_layer.rs",
                PRIMITIVES_DISMISSABLE_LAYER_RS,
                "render_dismissible_root_with_hooks(",
            ),
            (
                "primitives/focus_scope.rs",
                PRIMITIVES_FOCUS_SCOPE_RS,
                "collect_children(cx,",
            ),
            (
                "primitives/menu/content.rs",
                PRIMITIVES_MENU_CONTENT_RS,
                "roving_focus_group::roving_focus_group_apg_entry_fallback(",
            ),
            (
                "primitives/menu/content_panel.rs",
                PRIMITIVES_MENU_CONTENT_PANEL_RS,
                "collect_children(cx,",
            ),
            (
                "primitives/menu/sub_content.rs",
                PRIMITIVES_MENU_SUB_CONTENT_RS,
                "collect_children(cx,",
            ),
            (
                "primitives/popper_content.rs",
                PRIMITIVES_POPPER_CONTENT_RS,
                "collect_children(cx,",
            ),
            (
                "primitives/roving_focus_group.rs",
                PRIMITIVES_ROVING_FOCUS_GROUP_RS,
                "collect_children(cx,",
            ),
            (
                "primitives/tabs.rs",
                PRIMITIVES_TABS_RS,
                "collect_children(cx, items)",
            ),
            (
                "primitives/toggle.rs",
                PRIMITIVES_TOGGLE_RS,
                "collect_children(cx, items)",
            ),
            (
                "primitives/toolbar.rs",
                PRIMITIVES_TOOLBAR_RS,
                "roving_focus_group::roving_focus_group_apg(",
            ),
            (
                "recipes/sortable_dnd.rs",
                RECIPES_SORTABLE_DND_RS,
                "collect_children(cx, items)",
            ),
        ] {
            assert!(
                source.contains("IntoUiElement<"),
                "{label} should accept typed child values on the public wrapper surface"
            );
            assert!(
                !source.contains("IntoIterator<Item = AnyElement>"),
                "{label} reintroduced raw AnyElement child items on the public surface"
            );
            assert!(
                source.contains(landing_snippet),
                "{label} should only land typed child values behind a typed wrapper seam"
            );
        }
    }

    #[test]
    fn overlay_wrapper_helpers_land_typed_children_before_request_seams() {
        for (label, source, typed_signature, landing_snippet, raw_request_snippet) in [
            (
                "primitives/alert_dialog.rs",
                PRIMITIVES_ALERT_DIALOG_RS,
                "pub fn alert_dialog_modal_barrier<H: UiHost, I, T>(",
                "collect_children(cx, children)",
                "children: impl IntoIterator<Item = AnyElement>",
            ),
            (
                "primitives/dialog.rs",
                PRIMITIVES_DIALOG_RS,
                "pub fn modal_barrier<H: UiHost, I, T>(",
                "let children = collect_children(cx, children);",
                "children: impl IntoIterator<Item = AnyElement>",
            ),
            (
                "primitives/popover.rs",
                PRIMITIVES_POPOVER_RS,
                "pub fn popover_dialog_wrapper<H: UiHost, I, T>(",
                "collect_children(cx, items)",
                "children: impl IntoIterator<Item = AnyElement>",
            ),
            (
                "primitives/select.rs",
                PRIMITIVES_SELECT_RS,
                "pub fn select_modal_barrier<H: UiHost, I, T>(",
                "collect_children(cx, barrier_children)",
                "children: impl IntoIterator<Item = AnyElement>",
            ),
            (
                "primitives/tooltip.rs",
                PRIMITIVES_TOOLTIP_RS,
                "pub fn request<H: UiHost, I, T>(",
                "collect_children(cx, children)",
                "children: impl IntoIterator<Item = AnyElement>",
            ),
        ] {
            assert!(
                source.contains(typed_signature),
                "{label} should expose typed child wrappers where an ElementContext is available"
            );
            assert!(
                source.contains("IntoUiElement<H>"),
                "{label} should accept typed child values on wrapper helpers"
            );
            assert!(
                source.contains(landing_snippet),
                "{label} should land typed child values behind collect_children(...)"
            );
            assert!(
                source.contains(raw_request_snippet),
                "{label} should still document the raw AnyElement overlay-request landing seam"
            );
        }
    }

    #[test]
    fn primitives_and_base_recipes_stay_state_stack_agnostic() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let markers = [
            "use fret_query",
            "fret_query::",
            "use fret_selector",
            "fret_selector::",
        ];

        for root in [
            manifest_dir.join("src/primitives"),
            manifest_dir.join("src/recipes"),
            manifest_dir.join("../fret-ui-headless/src"),
        ] {
            visit_rust_files(&root, &mut |path, source| {
                let label = path.strip_prefix(manifest_dir).unwrap_or(path);
                for marker in markers {
                    assert!(
                        !source.contains(marker),
                        "{} reintroduced state-stack marker `{marker}` into a base primitive/recipe seam",
                        label.display()
                    );
                }
            });
        }
    }

    #[test]
    fn query_watch_helpers_stay_opt_in_and_out_of_default_declarative_prelude() {
        assert!(
            DECLARATIVE_MODEL_WATCH_RS.contains("#[cfg(feature = \"state-query\")]"),
            "declarative/model_watch.rs should keep query-watch helpers behind the `state-query` feature"
        );
        assert!(
            DECLARATIVE_MODEL_WATCH_RS.contains("pub trait QueryHandleWatchExt<T: 'static>"),
            "declarative/model_watch.rs should keep the opt-in query-watch helper explicit"
        );
        assert!(
            DECLARATIVE_MOD_RS.contains("pub use model_watch::QueryHandleWatchExt;"),
            "declarative/mod.rs should keep query-watch helpers on the explicit declarative root"
        );
        assert!(
            !DECLARATIVE_PRELUDE_RS.contains("QueryHandleWatchExt"),
            "declarative/prelude.rs should not make query-watch helpers part of the default prelude"
        );
        assert!(
            !DECLARATIVE_PRELUDE_RS.contains("fret_query"),
            "declarative/prelude.rs should remain free of direct query-crate imports"
        );
    }

    #[test]
    fn readme_keeps_icon_provider_installation_explicit_for_ui_kit() {
        assert!(README.contains("it does not install a default icon pack"));
        assert!(README.contains("`fret_icons_lucide::app::install`"));
        assert!(README.contains("`fret_icons_radix::app::install`"));
        assert!(README.contains("semantic `IconId`"));
        assert!(README.contains("semantic `IconId` / `ui.*`"));
        assert!(README.contains("one named installer/bundle surface"));
        assert!(README.contains("use fret_icons::ids;"));
        assert!(README.contains("use fret_ui_kit::prelude::*;"));
        assert!(README.contains("fret_icons_lucide::app::install(app);"));
        assert!(README.contains("let _icon = icon(ids::ui::SEARCH);"));
    }
}
