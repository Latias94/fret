//! Property row reset affordance owner.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexItemStyle, LayoutStyle, Length, PressableA11y, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_foreground, editor_subtle_bg};
use crate::primitives::readout::editor_property_row_reset_glyph_text_props;
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

pub type OnPropertyRowReset = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct PropertyRowResetOptions {
    pub enabled: bool,
    pub glyph: Arc<str>,
    pub a11y_label: Arc<str>,
    /// Explicit identity source for reset button state and action hooks.
    ///
    /// Falls back to `test_id` when omitted, which keeps diagnostics-addressable resets stable in
    /// loop-built property grids.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyRowResetOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            // ASCII fallback (avoid missing-glyph tofu on default fonts).
            glyph: Arc::from("R"),
            a11y_label: Arc::from("Reset to default"),
            id_source: None,
            test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct PropertyRowReset {
    pub options: PropertyRowResetOptions,
    pub on_reset: OnPropertyRowReset,
}

impl PropertyRowReset {
    pub fn new(on_reset: OnPropertyRowReset) -> Self {
        Self {
            options: PropertyRowResetOptions::default(),
            on_reset,
        }
    }

    pub fn options(mut self, options: PropertyRowResetOptions) -> Self {
        self.options = options;
        self
    }
}

pub(super) fn property_row_reset_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    reset: Option<PropertyRowReset>,
    affordance_extent: Px,
    reset_fg: Color,
) -> Option<AnyElement> {
    let reset = reset?;
    if !reset.options.enabled {
        return None;
    }

    let glyph = reset.options.glyph.clone();
    let a11y_label = reset.options.a11y_label.clone();
    let id_source = reset
        .options
        .id_source
        .clone()
        .or_else(|| reset.options.test_id.clone());
    let test_id = reset.options.test_id.clone();
    let on_reset = reset.on_reset.clone();

    if let Some(id_source) = id_source {
        Some(cx.keyed(
            ("fret-ui-editor.property_row.reset", id_source),
            move |cx| {
                property_row_reset_pressable(
                    cx,
                    glyph,
                    a11y_label,
                    test_id,
                    on_reset,
                    affordance_extent,
                    reset_fg,
                )
            },
        ))
    } else {
        Some(property_row_reset_pressable(
            cx,
            glyph,
            a11y_label,
            test_id,
            on_reset,
            affordance_extent,
            reset_fg,
        ))
    }
}

pub(super) fn property_row_reset_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    glyph: Arc<str>,
    a11y_label: Arc<str>,
    test_id: Option<Arc<str>>,
    on_reset: OnPropertyRowReset,
    affordance_extent: Px,
    reset_fg: Color,
) -> AnyElement {
    cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(affordance_extent),
                    height: Length::Px(affordance_extent),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    order: 0,
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Px(affordance_extent),
                    align_self: None,
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(a11y_label),
                test_id,
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let on_activate: OnActivate = Arc::new({
                let on_reset = on_reset.clone();
                move |host, action_cx, _reason: ActivateReason| {
                    on_reset(host, action_cx);
                    host.notify(action_cx);
                }
            });
            cx.pressable_add_on_activate(on_activate);

            let theme = Theme::global(&*cx.app);
            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;
            let mut idle_bg = editor_subtle_bg(theme);
            idle_bg.a = (idle_bg.a * 0.35).clamp(0.0, 1.0);
            let idle_border = editor_border(theme);
            let bg = editor_icon_button_bg(theme, true, hovered, pressed).unwrap_or(idle_bg);
            let border =
                editor_icon_button_border(theme, true, hovered, pressed).unwrap_or(idle_border);
            let fg = if hovered || pressed {
                editor_foreground(theme)
            } else {
                reset_fg
            };

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
                    background: Some(bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(Px(6.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(editor_property_row_reset_glyph_text_props(
                        glyph.clone(),
                        fg,
                        affordance_extent,
                    ))]
                },
            )]
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{PropertyRowReset, PropertyRowResetOptions};

    #[test]
    fn reset_defaults_keep_ascii_glyph_and_accessible_label() {
        let reset = PropertyRowReset::new(Arc::new(|_, _| {}));

        assert!(reset.options.enabled);
        assert_eq!(reset.options.glyph.as_ref(), "R");
        assert_eq!(reset.options.a11y_label.as_ref(), "Reset to default");
        assert_eq!(reset.options.id_source, None);
        assert_eq!(reset.options.test_id, None);
    }

    #[test]
    fn reset_options_builder_replaces_defaults() {
        let options = PropertyRowResetOptions {
            enabled: false,
            glyph: Arc::from("R2"),
            a11y_label: Arc::from("Reset value"),
            id_source: Some(Arc::from("row.reset")),
            test_id: Some(Arc::from("row.reset.test")),
        };
        let reset = PropertyRowReset::new(Arc::new(|_, _| {})).options(options.clone());

        assert_eq!(reset.options.enabled, options.enabled);
        assert_eq!(reset.options.glyph, options.glyph);
        assert_eq!(reset.options.a11y_label, options.a11y_label);
        assert_eq!(reset.options.id_source, options.id_source);
        assert_eq!(reset.options.test_id, options.test_id);
    }
}
