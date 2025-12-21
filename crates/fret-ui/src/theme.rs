use fret_core::{Color, Px};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub colors: HashMap<String, String>,
    pub metrics: HashMap<String, f32>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            author: None,
            url: None,
            colors: HashMap::new(),
            metrics: HashMap::new(),
        }
    }
}

impl ThemeConfig {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeMetrics {
    pub radius_sm: Px,
    pub radius_md: Px,
    pub radius_lg: Px,
    pub padding_sm: Px,
    pub padding_md: Px,
    pub scrollbar_width: Px,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub surface_background: Color,
    pub panel_background: Color,
    pub panel_border: Color,

    pub text_primary: Color,
    pub text_muted: Color,
    pub text_disabled: Color,

    pub accent: Color,
    pub selection_background: Color,
    pub hover_background: Color,
    pub focus_ring: Color,

    pub menu_background: Color,
    pub menu_border: Color,
    pub menu_item_hover: Color,
    pub menu_item_selected: Color,

    pub list_background: Color,
    pub list_border: Color,
    pub list_row_hover: Color,
    pub list_row_selected: Color,

    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_thumb_hover: Color,

    pub viewport_selection_fill: Color,
    pub viewport_selection_stroke: Color,
    pub viewport_marker: Color,
    pub viewport_drag_line_pan: Color,
    pub viewport_drag_line_orbit: Color,
    pub viewport_gizmo_x: Color,
    pub viewport_gizmo_y: Color,
    pub viewport_gizmo_handle_background: Color,
    pub viewport_gizmo_handle_border: Color,
    pub viewport_rotate_gizmo: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeSnapshot {
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
    pub revision: u64,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
    revision: u64,
}

impl Theme {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn snapshot(&self) -> ThemeSnapshot {
        ThemeSnapshot {
            colors: self.colors,
            metrics: self.metrics,
            revision: self.revision,
        }
    }

    pub fn global(app: &fret_app::App) -> &Theme {
        if let Some(theme) = app.global::<Theme>() {
            theme
        } else {
            default_theme()
        }
    }

    pub fn global_mut(app: &mut fret_app::App) -> &mut Theme {
        if app.global::<Theme>().is_none() {
            app.set_global(default_theme().clone());
        }
        app.global_mut::<Theme>().expect("theme global exists")
    }

    pub fn apply_config(&mut self, cfg: &ThemeConfig) {
        self.name = cfg.name.clone();
        self.author = cfg.author.clone();
        self.url = cfg.url.clone();

        let mut changed = false;

        macro_rules! apply_color {
            ($key:literal, $field:expr) => {
                if let Some(v) = cfg.colors.get($key) {
                    if let Some(c) = parse_hex_srgb_to_linear(v) {
                        if $field != c {
                            $field = c;
                            changed = true;
                        }
                    }
                }
            };
        }

        macro_rules! apply_metric {
            ($key:literal, $field:expr) => {
                if let Some(v) = cfg.metrics.get($key).copied() {
                    let px = Px(v);
                    if $field != px {
                        $field = px;
                        changed = true;
                    }
                }
            };
        }

        apply_color!("color.surface.background", self.colors.surface_background);
        apply_color!("color.panel.background", self.colors.panel_background);
        apply_color!("color.panel.border", self.colors.panel_border);

        apply_color!("color.text.primary", self.colors.text_primary);
        apply_color!("color.text.muted", self.colors.text_muted);
        apply_color!("color.text.disabled", self.colors.text_disabled);

        apply_color!("color.accent", self.colors.accent);
        apply_color!(
            "color.selection.background",
            self.colors.selection_background
        );
        apply_color!("color.hover.background", self.colors.hover_background);
        apply_color!("color.focus.ring", self.colors.focus_ring);

        apply_color!("color.menu.background", self.colors.menu_background);
        apply_color!("color.menu.border", self.colors.menu_border);
        apply_color!("color.menu.item.hover", self.colors.menu_item_hover);
        apply_color!("color.menu.item.selected", self.colors.menu_item_selected);

        apply_color!("color.list.background", self.colors.list_background);
        apply_color!("color.list.border", self.colors.list_border);
        apply_color!("color.list.row.hover", self.colors.list_row_hover);
        apply_color!("color.list.row.selected", self.colors.list_row_selected);

        apply_color!("color.scrollbar.track", self.colors.scrollbar_track);
        apply_color!("color.scrollbar.thumb", self.colors.scrollbar_thumb);
        apply_color!(
            "color.scrollbar.thumb.hover",
            self.colors.scrollbar_thumb_hover
        );

        apply_color!(
            "color.viewport.selection.fill",
            self.colors.viewport_selection_fill
        );
        apply_color!(
            "color.viewport.selection.stroke",
            self.colors.viewport_selection_stroke
        );
        apply_color!("color.viewport.marker", self.colors.viewport_marker);
        apply_color!(
            "color.viewport.drag_line.pan",
            self.colors.viewport_drag_line_pan
        );
        apply_color!(
            "color.viewport.drag_line.orbit",
            self.colors.viewport_drag_line_orbit
        );
        apply_color!("color.viewport.gizmo.x", self.colors.viewport_gizmo_x);
        apply_color!("color.viewport.gizmo.y", self.colors.viewport_gizmo_y);
        apply_color!(
            "color.viewport.gizmo.handle.background",
            self.colors.viewport_gizmo_handle_background
        );
        apply_color!(
            "color.viewport.gizmo.handle.border",
            self.colors.viewport_gizmo_handle_border
        );
        apply_color!(
            "color.viewport.rotate_gizmo",
            self.colors.viewport_rotate_gizmo
        );

        apply_metric!("metric.radius.sm", self.metrics.radius_sm);
        apply_metric!("metric.radius.md", self.metrics.radius_md);
        apply_metric!("metric.radius.lg", self.metrics.radius_lg);
        apply_metric!("metric.padding.sm", self.metrics.padding_sm);
        apply_metric!("metric.padding.md", self.metrics.padding_md);
        apply_metric!("metric.scrollbar.width", self.metrics.scrollbar_width);

        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }
}

fn default_theme() -> &'static Theme {
    static DEFAULT: OnceLock<Theme> = OnceLock::new();
    DEFAULT.get_or_init(|| Theme {
        name: "Default".to_string(),
        author: None,
        url: None,
        revision: 1,
        metrics: ThemeMetrics {
            radius_sm: Px(6.0),
            radius_md: Px(8.0),
            radius_lg: Px(10.0),
            padding_sm: Px(8.0),
            padding_md: Px(10.0),
            scrollbar_width: Px(10.0),
        },
        colors: ThemeColors {
            surface_background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            panel_background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            panel_border: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            text_primary: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            text_muted: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.7,
            },
            text_disabled: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.45,
            },
            accent: Color {
                r: 0.20,
                g: 0.75,
                b: 1.0,
                a: 1.0,
            },
            selection_background: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            hover_background: Color {
                r: 0.16,
                g: 0.17,
                b: 0.22,
                a: 0.95,
            },
            focus_ring: Color {
                r: 0.20,
                g: 0.75,
                b: 1.0,
                a: 0.85,
            },
            menu_background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            menu_border: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.40,
            },
            menu_item_hover: Color {
                r: 0.16,
                g: 0.17,
                b: 0.22,
                a: 0.95,
            },
            menu_item_selected: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            list_background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            list_border: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            list_row_hover: Color {
                r: 0.16,
                g: 0.17,
                b: 0.22,
                a: 0.95,
            },
            list_row_selected: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            scrollbar_track: Color {
                r: 0.10,
                g: 0.10,
                b: 0.11,
                a: 0.90,
            },
            scrollbar_thumb: Color {
                r: 0.42,
                g: 0.42,
                b: 0.45,
                a: 0.90,
            },
            scrollbar_thumb_hover: Color {
                r: 0.55,
                g: 0.55,
                b: 0.58,
                a: 0.90,
            },

            viewport_selection_fill: Color {
                r: 0.20,
                g: 0.45,
                b: 0.95,
                a: 0.16,
            },
            viewport_selection_stroke: Color {
                r: 0.20,
                g: 0.45,
                b: 0.95,
                a: 0.85,
            },
            viewport_marker: Color {
                r: 0.20,
                g: 0.45,
                b: 0.95,
                a: 0.95,
            },
            viewport_drag_line_pan: Color {
                r: 0.25,
                g: 0.92,
                b: 0.55,
                a: 0.85,
            },
            viewport_drag_line_orbit: Color {
                r: 1.0,
                g: 0.82,
                b: 0.28,
                a: 0.85,
            },
            viewport_gizmo_x: Color {
                r: 0.92,
                g: 0.28,
                b: 0.30,
                a: 1.0,
            },
            viewport_gizmo_y: Color {
                r: 0.25,
                g: 0.88,
                b: 0.40,
                a: 1.0,
            },
            viewport_gizmo_handle_background: Color {
                r: 0.08,
                g: 0.08,
                b: 0.10,
                a: 1.0,
            },
            viewport_gizmo_handle_border: Color {
                r: 0.92,
                g: 0.92,
                b: 0.95,
                a: 1.0,
            },
            viewport_rotate_gizmo: Color {
                r: 0.98,
                g: 0.82,
                b: 0.28,
                a: 1.0,
            },
        },
    })
}

fn parse_hex_srgb_to_linear(s: &str) -> Option<Color> {
    let s = s.trim();
    let hex = s.strip_prefix('#').unwrap_or(s);
    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };

    fn srgb_channel_to_linear(u: u8) -> f32 {
        let c = u as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    Some(Color {
        r: srgb_channel_to_linear(r),
        g: srgb_channel_to_linear(g),
        b: srgb_channel_to_linear(b),
        a: a as f32 / 255.0,
    })
}
