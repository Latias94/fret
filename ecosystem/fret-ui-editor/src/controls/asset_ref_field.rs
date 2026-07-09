//! UI-only asset reference field for editor inspectors.
//!
//! The control intentionally does not resolve, browse, or load assets. Callers own the asset
//! domain, async/query state, and all actions; this widget only provides a consistent editor
//! property-field shell.

use std::sync::Arc;

use fret_core::{Edges, Px};
use fret_icons::IconId;
use fret_ui::action::{ActionCx, OnActivate, UiActionHost};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::Size;

use crate::controls::field_status::{FieldStatus, FieldStatusBadge};
use crate::primitives::colors::{editor_foreground, editor_muted_foreground};
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, derived_test_id, editor_icon_button_segment,
    editor_icon_segment, editor_input_value_text,
    editor_joined_input_frame_segments_with_overrides,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::EditorFrameSemanticState;

pub type OnAssetRefFieldAction = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRefFieldValue {
    pub label: Arc<str>,
    pub path: Option<Arc<str>>,
    pub icon: Option<IconId>,
}

impl AssetRefFieldValue {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            path: None,
            icon: None,
        }
    }

    pub fn path(mut self, path: impl Into<Arc<str>>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn display_text(&self) -> Arc<str> {
        match self.path.as_deref() {
            Some(path) if path != self.label.as_ref() && !path.is_empty() => {
                Arc::from(format!("{} - {}", self.label, path))
            }
            _ => self.label.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AssetRefFieldOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Arc<str>,
    pub enabled: bool,
    pub test_id: Option<Arc<str>>,
    pub value_test_id: Option<Arc<str>>,
    pub choose_test_id: Option<Arc<str>>,
    pub reveal_test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,
    pub status: Option<FieldStatus>,
    pub on_choose: Option<OnAssetRefFieldAction>,
    pub on_reveal: Option<OnAssetRefFieldAction>,
    pub on_clear: Option<OnAssetRefFieldAction>,
}

impl Default for AssetRefFieldOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            size: Size::Small,
            placeholder: Arc::from("No asset assigned"),
            enabled: true,
            test_id: None,
            value_test_id: None,
            choose_test_id: None,
            reveal_test_id: None,
            clear_test_id: None,
            status: None,
            on_choose: None,
            on_reveal: None,
            on_clear: None,
        }
    }
}

#[derive(Clone)]
pub struct AssetRefField {
    value: Option<AssetRefFieldValue>,
    options: AssetRefFieldOptions,
}

impl AssetRefField {
    pub fn new(value: Option<AssetRefFieldValue>) -> Self {
        Self {
            value,
            options: AssetRefFieldOptions::default(),
        }
    }

    pub fn options(mut self, options: AssetRefFieldOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let AssetRefField { value, options } = self;

        let has_value = value.is_some();
        let value_icon = value.as_ref().and_then(|v| v.icon.clone()).unwrap_or({
            if has_value {
                fret_icons::ids::ui::FILE
            } else {
                fret_icons::ids::ui::FOLDER_OPEN
            }
        });
        let value_text = value
            .as_ref()
            .map(AssetRefFieldValue::display_text)
            .unwrap_or_else(|| options.placeholder.clone());
        let value_test_id = options
            .value_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "value"));

        let (density, frame_chrome, value_color) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let value_color = if has_value {
                editor_foreground(theme)
            } else {
                editor_muted_foreground(theme)
            };
            (style.density, style.frame_chrome(options.size), value_color)
        };

        let invalid = matches!(options.status, Some(FieldStatus::Error(_)));
        let enabled_for_paint = options.enabled;
        let choose = options.on_choose.clone();
        let reveal = has_value.then(|| options.on_reveal.clone()).flatten();
        let clear = has_value.then(|| options.on_clear.clone()).flatten();
        let status = options.status.clone();

        editor_joined_input_frame_segments_with_overrides(
            cx,
            options.layout,
            density,
            frame_chrome,
            enabled_for_paint,
            false,
            options.test_id.clone(),
            move |_cx, focused| EditorInputGroupFrameOverrides {
                semantic: Some(EditorFrameSemanticState {
                    typing: focused,
                    invalid,
                }),
                ..EditorInputGroupFrameOverrides::none()
            },
            move |cx| {
                vec![editor_icon_segment(
                    cx,
                    density,
                    value_icon.clone(),
                    Some(Px(14.0)),
                    None,
                )]
            },
            move |cx| {
                let mut text = editor_input_value_text(
                    cx,
                    density,
                    frame_chrome.text_px,
                    value_text.clone(),
                    value_color,
                    Length::Fill,
                );
                if let Some(test_id) = value_test_id.as_ref() {
                    text = text.test_id(test_id.clone());
                }
                text
            },
            move |cx| {
                let mut segments = Vec::new();
                if let Some(status) = status.clone() {
                    let badge = FieldStatusBadge::new(status)
                        .options(crate::controls::field_status::FieldStatusBadgeOptions {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Auto,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges::symmetric(Px(4.0), Px(0.0)),
                        })
                        .into_element(cx);
                    segments.push(badge);
                }
                if let Some(action) = choose.clone() {
                    segments.push(asset_ref_action_segment(
                        cx,
                        density,
                        enabled_for_paint,
                        Arc::from("Choose asset"),
                        fret_icons::ids::ui::FOLDER_OPEN,
                        options.choose_test_id.clone(),
                        action,
                    ));
                }
                if let Some(action) = reveal.clone() {
                    segments.push(asset_ref_action_segment(
                        cx,
                        density,
                        enabled_for_paint,
                        Arc::from("Reveal asset"),
                        fret_icons::ids::ui::FILE,
                        options.reveal_test_id.clone(),
                        action,
                    ));
                }
                if let Some(action) = clear.clone() {
                    segments.push(asset_ref_action_segment(
                        cx,
                        density,
                        enabled_for_paint,
                        Arc::from("Clear asset"),
                        fret_icons::ids::ui::CLOSE,
                        options.clear_test_id.clone(),
                        action,
                    ));
                }
                segments
            },
        )
    }
}

fn asset_ref_action_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: crate::primitives::EditorDensity,
    enabled: bool,
    a11y_label: Arc<str>,
    icon: IconId,
    test_id: Option<Arc<str>>,
    action: OnAssetRefFieldAction,
) -> AnyElement {
    let on_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
        action(host, action_cx);
        host.request_redraw(action_cx.window);
    });

    editor_icon_button_segment(
        cx,
        density,
        enabled,
        a11y_label,
        icon,
        Some(Px(12.0)),
        test_id,
        on_activate,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_ref_display_text_includes_path_without_defining_asset_semantics() {
        let value = AssetRefFieldValue::new("Base Color").path("textures/default/basecolor.ktx2");

        assert_eq!(
            value.display_text().as_ref(),
            "Base Color - textures/default/basecolor.ktx2"
        );
    }

    #[test]
    fn asset_ref_options_default_to_ui_only_shell() {
        let options = AssetRefFieldOptions::default();

        assert!(options.on_choose.is_none());
        assert!(options.on_reveal.is_none());
        assert!(options.on_clear.is_none());
        assert!(options.status.is_none());
        assert_eq!(options.placeholder.as_ref(), "No asset assigned");
    }
}
