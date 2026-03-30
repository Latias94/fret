//! Reusable editor icon button (policy-layer control).
//!
//! This is intentionally small: it provides consistent chrome for icon-only pressables and is
//! used by multiple editor controls (clear buttons, row actions, etc.).

use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px};
use fret_icons::IconId;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::ColorRef;

use crate::primitives::EditorDensity;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::icons::editor_icon_with;
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

pub type OnIconButtonActivate = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct IconButtonOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub width: Option<Px>,
    pub height: Option<Px>,
    pub corner_radius: Option<Px>,
    pub icon_size: Option<Px>,
    pub icon_color: Option<ColorRef>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for IconButtonOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: false,
            width: None,
            height: None,
            corner_radius: Some(Px(6.0)),
            icon_size: None,
            icon_color: None,
            a11y_label: None,
            test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct IconButton {
    icon: IconId,
    on_activate: OnIconButtonActivate,
    options: IconButtonOptions,
}

impl IconButton {
    pub fn new(icon: IconId, on_activate: OnIconButtonActivate) -> Self {
        Self {
            icon,
            on_activate,
            options: IconButtonOptions::default(),
        }
    }

    pub fn options(mut self, options: IconButtonOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let density = {
            let theme = Theme::global(&*cx.app);
            EditorDensity::resolve(theme)
        };
        let affordance_extent = density.affordance_extent();

        let w = self.options.width.unwrap_or(affordance_extent);
        let h = self.options.height.unwrap_or(affordance_extent);
        let radius = self.options.corner_radius.unwrap_or(Px(6.0));
        let icon_size = self.options.icon_size.or(Some(density.icon_size));

        let enabled = self.options.enabled;
        let focusable = self.options.focusable;
        let a11y_label = self.options.a11y_label.clone();
        let icon = self.icon.clone();
        let on_activate = self.on_activate.clone();
        let icon_color = self.options.icon_color.clone();

        let mut el = cx.pressable(
            PressableProps {
                enabled,
                focusable,
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(w),
                        height: Length::Px(h),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                a11y: PressableA11y {
                    label: a11y_label,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, st| {
                let on_activate: OnActivate = Arc::new({
                    let on_activate = on_activate.clone();
                    move |host, action_cx: ActionCx, _reason: ActivateReason| {
                        on_activate(host, action_cx);
                        host.request_redraw(action_cx.window);
                    }
                });
                cx.pressable_add_on_activate(on_activate);

                let hovered = st.hovered || st.hovered_raw;
                let pressed = st.pressed;

                let (bg, border, fg) = {
                    let theme = Theme::global(&*cx.app);
                    (
                        editor_icon_button_bg(theme, enabled, hovered, pressed),
                        editor_icon_button_border(theme, enabled, hovered, pressed),
                        editor_muted_foreground(theme),
                    )
                };
                let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

                let icon_color = icon_color.clone().unwrap_or(ColorRef::Color(fg));

                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        background: bg,
                        border: Edges::all(border_width),
                        border_color: border,
                        corner_radii: Corners::all(radius),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: SpacingLength::Px(Px(0.0)),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Center,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                vec![editor_icon_with(
                                    cx,
                                    density,
                                    icon.clone(),
                                    icon_size,
                                    Some(icon_color.clone()),
                                )]
                            },
                        )]
                    },
                )]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}
