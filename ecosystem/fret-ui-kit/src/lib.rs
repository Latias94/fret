#![deny(deprecated)]
//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).
//!
//! Note: This crate is declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface.

/// Build a heterogeneous `Vec<AnyElement>` without repetitive `.into_element(cx)` boilerplate.
///
/// Intended for ergonomic authoring inside layout builders, e.g.:
/// `ui::h_flex(|cx| ui::children![cx; Button::new("OK").ui(), cx.text("...")])`.
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
                let element = $crate::UiIntoElement::into_element(child, &mut *$cx);
                children.push(element);
            }
        )+
        children
    }};
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

        impl $crate::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
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

        impl $crate::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
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

        impl $crate::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
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

/// Implement `UiIntoElement` for a stateless component authored as `RenderOnce` (ADR 0039).
///
/// Note: we intentionally avoid a blanket impl due to coherence restrictions on upstream types.
#[macro_export]
macro_rules! ui_into_element_render_once {
    ($ty:ty) => {
        impl $crate::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
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
    UiBuilder, UiExt, UiIntoElement, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
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
        ImageMetadataStore, ImageSamplingExt, LayoutRefinement, MarginEdge, MetricRef,
        OverrideSlot, Radius, ShadowPreset, SignedMetricRef, Size, Space, StyledExt, UiExt,
        UiIntoElement, WidgetState, WidgetStateProperty, WidgetStates, merge_override_slot,
        merge_slot, resolve_override_slot, resolve_override_slot_opt,
        resolve_override_slot_opt_with, resolve_override_slot_with, resolve_slot,
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
        fn assert_traits<T: UiPatchTarget + UiSupportsChrome + UiSupportsLayout + UiIntoElement>() {
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

    ui_into_element_render_once!(DummyRenderOnceComponent);

    #[test]
    fn ui_into_element_render_once_macro_compiles() {
        fn assert_into_element<T: UiIntoElement>() {}
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
