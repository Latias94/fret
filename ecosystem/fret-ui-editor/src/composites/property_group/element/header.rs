//! PropertyGroup header pressable owner.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacerProps, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::icons::editor_icon;
use crate::primitives::readout::editor_property_group_header_text_props;
use crate::primitives::visuals::hover_overlay_bg;

use super::super::OnPropertyGroupToggle;

pub(super) struct PropertyGroupHeaderElementOptions {
    pub(super) label: Arc<str>,
    pub(super) enabled: bool,
    pub(super) collapsible: bool,
    pub(super) collapsed: bool,
    pub(super) collapsed_model: Model<bool>,
    pub(super) on_toggle: Option<OnPropertyGroupToggle>,
    pub(super) header_height: Px,
    pub(super) density: EditorDensity,
    pub(super) header_bg: Color,
    pub(super) header_border: Color,
    pub(super) radius: Px,
    pub(super) header_fg: Color,
    pub(super) test_id: Option<Arc<str>>,
}

pub(super) fn property_group_header_element<H, HeaderActions>(
    cx: &mut ElementContext<'_, H>,
    options: PropertyGroupHeaderElementOptions,
    header_actions: HeaderActions,
) -> AnyElement
where
    H: UiHost,
    HeaderActions: FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
{
    let PropertyGroupHeaderElementOptions {
        label,
        enabled,
        collapsible,
        collapsed,
        collapsed_model,
        on_toggle,
        header_height,
        density,
        header_bg,
        header_border,
        radius,
        header_fg,
        test_id,
    } = options;

    let disclosure_icon = collapsible.then_some({
        if collapsed {
            fret_icons::ids::ui::CHEVRON_RIGHT
        } else {
            fret_icons::ids::ui::CHEVRON_DOWN
        }
    });

    let header_label = label.clone();
    let collapsed_for_toggle = collapsed_model.clone();

    let mut header = cx.pressable(
        PressableProps {
            enabled: enabled && collapsible,
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    min_height: Some(Length::Px(Px(header_height.0.max(0.0)))),
                    ..Default::default()
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(Arc::from(format!("Toggle {header_label}"))),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let on_activate: OnActivate = Arc::new({
                let on_toggle = on_toggle.clone();
                let collapsed_for_toggle = collapsed_for_toggle.clone();
                move |host, action_cx: ActionCx, _reason: ActivateReason| {
                    let next = host
                        .models_mut()
                        .update(&collapsed_for_toggle, |b| {
                            *b = !*b;
                            *b
                        })
                        .unwrap_or(false);
                    if let Some(cb) = on_toggle.as_ref() {
                        cb(host, action_cx, next);
                    }
                    host.request_redraw(action_cx.window);
                }
            });
            cx.pressable_add_on_activate(on_activate);

            let theme = Theme::global(&*cx.app);
            let header_bg = hover_overlay_bg(theme, header_bg, st.hovered, st.pressed);

            let actions = header_actions(cx);
            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: Some(header_bg),
                    corner_radii: Corners {
                        top_left: radius,
                        top_right: radius,
                        bottom_right: Px(0.0),
                        bottom_left: Px(0.0),
                    },
                    border: Edges {
                        top: Px(0.0),
                        right: Px(0.0),
                        bottom: Px(1.0),
                        left: Px(0.0),
                    },
                    border_color: Some(header_border),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: SpacingLength::Px(Px(6.0)),
                                padding: Edges {
                                    top: Px(density.padding_y.0 + 2.0),
                                    right: density.padding_x,
                                    bottom: Px(density.padding_y.0 + 2.0),
                                    left: density.padding_x,
                                }
                                .into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                let mut out = Vec::new();
                                if let Some(icon) = disclosure_icon.clone() {
                                    out.push(editor_icon(cx, density, icon, Some(Px(12.0))));
                                }
                                out.push(cx.text_props(editor_property_group_header_text_props(
                                    header_label.clone(),
                                    header_fg,
                                    header_height,
                                )));
                                out.push(cx.spacer(SpacerProps::default()));
                                if let Some(actions) = actions {
                                    out.push(actions);
                                }
                                out
                            },
                        ),
                    ]
                },
            )]
        },
    );

    if let Some(test_id) = test_id.as_ref() {
        header = header.test_id(test_id.clone());
    }

    header
}
