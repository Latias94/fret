/// Canonical app-hosted component/snippet context alias.
pub use crate::AppComponentCx;
/// Canonical app-facing single-element alias for extracted helper functions.
pub use crate::AppElement;
/// Canonical app-facing concrete helper context alias for closure-local or inline helpers.
pub use crate::AppRenderCx;
/// Canonical app-facing view trait on the explicit app lane.
pub use crate::view::View;
/// Explicit helper types/traits for app helper signatures that intentionally name them.
pub use crate::view::{
    AppLocalStateExt, AppLocalStateTxnExt, AppRenderActionsExt, AppRenderContext, AppRenderDataExt,
    LocalState, LocalStateTxn, RenderContextAccess, TrackedStateExt, UiActionHostLocalStateTxnExt,
    view_child, view_child_with,
};
/// Canonical app-facing runtime handle on the default `fret` surface.
///
/// This is the same underlying runtime type as the raw kernel alias exposed on
/// `fret::advanced::kernel`; prefer this name in ordinary app code and keep the raw alias for
/// advanced/manual integration seams.
pub use fret_app::App;
/// Explicit context-access capability for helper signatures that should not hard-code raw
/// `ElementContext` ownership.
pub use fret_ui::ElementContextAccess;

/// App-facing frame-pipeline observations for harnesses and diagnostics.
///
/// These types expose ordered stage metadata without handing app code the retained tree or raw
/// frame context.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use fret_bootstrap::ui_app_driver::{
    UiAppFrameObservation, UiAppFrameStage, UiAppFrameStageSink,
};

/// App-facing editor bindings for `LocalState`-owned inspector controls.
///
/// This lane adapts editor controls to the default app state surface without moving editor
/// policy into the core UI crates or exposing raw `ModelStore` ownership at app call sites.
#[cfg(feature = "editor")]
pub mod editor {
    pub use crate::view::{
        EditorThemePresetPickerLocalStateExt, InspectorTextFieldBinding, InspectorTextFieldOutcome,
        InspectorTextFieldSnapshot, TextFieldLocalStateExt,
    };
    pub use fret_ui_editor::controls::{
        EditorTextCancelBehavior, EditorTextSelectionBehavior, EditorThemePresetPicker,
        EditorThemePresetPickerOptions, TextField, TextFieldAssistiveSemantics,
        TextFieldBlurBehavior, TextFieldDraftSnapshot, TextFieldMode, TextFieldOptions,
        TextFieldOutcome,
    };
    pub use fret_ui_editor::theme::EditorThemePreset;
}

/// Request that the runner close a window.
pub fn close_window(app: &mut App, window: crate::WindowId) {
    app.push_effect(fret_app::Effect::Window(fret_app::WindowRequest::Close(
        window,
    )));
}

/// Build an activation handler that closes the active window.
pub fn close_window_activate() -> fret_ui::action::OnActivate {
    std::sync::Arc::new(|host, acx, _reason| {
        host.push_effect(fret_app::Effect::Window(fret_app::WindowRequest::Close(
            acx.window,
        )));
    })
}

/// App-facing text helpers for the default render lane.
///
/// These are thin wrappers over `fret-ui-kit` text recipes. They keep first-contact app code
/// on `AppUi` / `AppRenderContext` instead of teaching raw `ElementContext` or `AnyElement`
/// boundaries for ordinary labels, readouts, and paragraphs.
pub mod text {
    use std::sync::Arc;

    /// Compact control/status readout text.
    pub fn control_readout<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_control_readout(cx.elements(), text)
    }

    /// Compact control label text.
    pub fn control_label<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_control_label(cx.elements(), text)
    }

    /// Prose paragraph text.
    pub fn paragraph<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_paragraph(cx.elements(), text)
    }

    /// Prose paragraph text that may break long words, with an inherited foreground.
    pub fn paragraph_break_words_with_foreground<'a, Cx, T>(
        cx: &mut Cx,
        text: T,
        foreground: fret_core::Color,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_paragraph_break_words(cx.elements(), text)
            .inherit_foreground(foreground)
    }

    /// Compact prose paragraph text.
    pub fn compact_paragraph<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_compact_paragraph(cx.elements(), text)
    }

    /// List-row label text.
    pub fn list_row_label<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_list_row_label(cx.elements(), text)
    }

    /// List-row label text with an inherited foreground.
    pub fn list_row_label_with_foreground<'a, Cx, T>(
        cx: &mut Cx,
        text: T,
        foreground: fret_core::Color,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_list_row_label(cx.elements(), text)
            .inherit_foreground(foreground)
    }

    /// Chrome/header title text.
    pub fn chrome_title<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_chrome_title(cx.elements(), text)
    }

    /// Button/control label text.
    pub fn button_label<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_button_label(cx.elements(), text)
    }

    /// Dense table-cell text.
    pub fn table_cell<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_table_cell(cx.elements(), text)
    }

    /// Emphasized dense table-cell text.
    pub fn table_cell_emphasis<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_table_cell_emphasis(cx.elements(), text)
    }

    /// Attributed list-row label text with an inherited foreground.
    pub fn list_row_label_attributed_with_foreground<'a, Cx>(
        cx: &mut Cx,
        rich: fret_core::AttributedText,
        foreground: fret_core::Color,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_kit::declarative::text::text_list_row_label_attributed(cx.elements(), rich)
            .inherit_foreground(foreground)
    }

    /// Section/chrome label text.
    pub fn section_chrome_label<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_section_chrome_label(cx.elements(), text)
    }

    /// Compact chrome glyph text.
    pub fn chrome_glyph<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_chrome_glyph(cx.elements(), text)
    }

    /// Inline code label text.
    pub fn code_label<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_code_label(cx.elements(), text)
    }

    /// Block code text.
    pub fn code_block<'a, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_code_block(cx.elements(), text)
    }
}

/// App-facing pressable helpers for simple command-backed interactive regions.
///
/// Component crates can still use raw `PressableProps`; default app code should prefer these
/// helpers when it only needs a button-like role, command dispatch, and hover/press styling.
pub mod pressable {
    use std::sync::Arc;

    pub use fret_ui::element::PressableState;

    pub fn command_button<'a, Cx, L, I, T>(
        cx: &mut Cx,
        command: fret_runtime::CommandId,
        label: L,
        render: impl FnOnce(&mut crate::AppRenderCx<'_>, PressableState) -> I,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        L: Into<Arc<str>>,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;

        cx.elements().pressable(
            fret_ui::element::PressableProps {
                enabled: true,
                a11y: fret_ui::element::PressableA11y {
                    role: Some(fret_core::SemanticsRole::Button),
                    label: Some(label.into()),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, state| {
                cx.pressable_dispatch_command_if_enabled(command);
                render(cx, state)
                    .into_iter()
                    .map(|child| fret_ui_kit::IntoUiElement::into_element(child, cx))
                    .collect::<Vec<_>>()
            },
        )
    }
}

/// App-facing UI asset helpers for the default render lane.
///
/// These wrappers keep app and cookbook code on `AppUi` / `AppRenderContext` while still using
/// the ViewCache-safe observation hooks owned by `fret-ui-assets`.
#[cfg(feature = "ui-assets")]
pub mod ui_assets {
    pub use fret_core::{ImageColorSpace, ImageId};
    pub use fret_ui_assets::app::{configure_caches, configure_caches_with_budgets};
    pub use fret_ui_assets::image_asset_cache::{ImageAssetKey, ImageAssetStats};
    pub use fret_ui_assets::image_asset_state::ImageLoadingStatus;
    pub use fret_ui_assets::image_source::{ImageSource, ImageSourceOptions, ImageSourceState};
    pub use fret_ui_assets::svg_asset_cache::SvgAssetStats;
    pub use fret_ui_assets::ui::SvgAssetSourceState;
    pub use fret_ui_assets::ui_assets::{UiAssets, UiAssetsBudgets};

    pub fn rgba8_image_state<'a, Cx>(
        cx: &mut Cx,
        width: u32,
        height: u32,
        rgba: &[u8],
        color_space: fret_core::ImageColorSpace,
    ) -> (
        ImageAssetKey,
        Option<fret_core::ImageId>,
        ImageLoadingStatus,
    )
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::use_rgba8_image_state_in(cx, width, height, rgba, color_space)
    }

    pub fn image_source_state<'a, Cx>(cx: &mut Cx, source: &ImageSource) -> ImageSourceState
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::use_image_source_state_in(cx, source)
    }

    pub fn image_source_state_from_asset_request<'a, Cx>(
        cx: &mut Cx,
        request: &crate::assets::AssetRequest,
    ) -> ImageSourceState
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::use_image_source_state_from_asset_request_in(cx, request)
    }

    pub fn image_source_state_from_asset_locator<'a, Cx>(
        cx: &mut Cx,
        locator: crate::assets::AssetLocator,
    ) -> ImageSourceState
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::use_image_source_state_from_asset_locator_in(cx, locator)
    }

    pub fn svg_source_state_from_asset_request<'a, Cx>(
        cx: &mut Cx,
        request: &crate::assets::AssetRequest,
    ) -> SvgAssetSourceState
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::svg_source_state_from_asset_request_in(cx, request)
    }

    pub fn svg_source_state_from_asset_locator<'a, Cx>(
        cx: &mut Cx,
        locator: crate::assets::AssetLocator,
    ) -> SvgAssetSourceState
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::svg_source_state_from_asset_locator_in(cx, locator)
    }

    pub fn image_stats<'a, Cx>(cx: &mut Cx) -> ImageAssetStats
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::image_stats_in(cx)
    }

    pub fn svg_stats<'a, Cx>(cx: &mut Cx) -> SvgAssetStats
    where
        Cx: crate::app::AppRenderContext<'a>,
    {
        fret_ui_assets::ui::svg_stats_in(cx)
    }
}

/// Common imports for app code on the default authoring surface.
pub mod prelude;

/// Explicit bridge for app-facing widgets that only expose `on_activate(...)`.
///
/// This intentionally stays off `fret::app::prelude::*` so default app autocomplete remains
/// focused on native widget action slots. Import `use fret::app::AppActivateExt as _;`
/// explicitly at call sites that still need activation-only `.action(...)`,
/// `.action_payload(...)`, or `.listen(...)` sugar.
pub use crate::view::{AppActivateExt, AppActivateSurface};
