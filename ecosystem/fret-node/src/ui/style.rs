use fret_core::{Color, Px, TextStyle};
use fret_ui::{Theme, ThemeSnapshot};

/// Visual tuning for the node graph canvas.
#[derive(Debug, Clone)]
pub struct NodeGraphStyle {
    pub background: Color,

    pub grid_spacing: f32,
    pub grid_minor_color: Color,
    pub grid_major_every: u32,
    pub grid_major_color: Color,

    pub node_width: f32,
    pub node_header_height: f32,
    pub node_padding: f32,
    pub pin_row_height: f32,
    pub pin_radius: f32,

    pub node_background: Color,
    pub node_border: Color,
    pub node_border_selected: Color,

    pub group_background: Color,
    pub group_border: Color,

    pub resize_handle_size: f32,
    pub resize_handle_background: Color,
    pub resize_handle_border: Color,

    pub pin_color_data: Color,
    pub pin_color_exec: Color,

    pub wire_width: f32,
    pub wire_interaction_width: f32,
    pub wire_color_data: Color,
    pub wire_color_exec: Color,
    pub wire_color_preview: Color,

    pub wire_width_selected_mul: f32,
    pub wire_width_hover_mul: f32,

    pub marquee_fill: Color,
    pub marquee_border: Color,
    pub marquee_border_width: f32,

    pub snapline_color: Color,
    pub snapline_width: f32,

    pub context_menu_width: f32,
    pub context_menu_padding: f32,
    pub context_menu_item_height: f32,
    pub context_menu_corner_radius: f32,
    pub context_menu_background: Color,
    pub context_menu_border: Color,
    pub context_menu_hover_background: Color,
    pub context_menu_text: Color,
    pub context_menu_text_disabled: Color,
    pub context_menu_text_style: TextStyle,

    /// Minimap overlay width in screen pixels.
    pub minimap_width: f32,
    /// Minimap overlay height in screen pixels.
    pub minimap_height: f32,
    /// Minimap margin from window edge in screen pixels.
    pub minimap_margin: f32,
    /// Extra padding around computed world bounds in canvas units.
    pub minimap_world_padding: f32,

    /// Controls overlay button size in screen pixels.
    pub controls_button_size: f32,
    /// Gap between control buttons in screen pixels.
    pub controls_gap: f32,
    /// Controls margin from window edge in screen pixels.
    pub controls_margin: f32,
    /// Controls panel padding in screen pixels.
    pub controls_padding: f32,
    /// Controls text color.
    pub controls_text: Color,
    /// Controls text style.
    pub controls_text_style: TextStyle,
    /// Controls hover background color.
    pub controls_hover_background: Color,
    /// Controls pressed background color.
    pub controls_active_background: Color,

    /// Extra render culling margin in screen-space pixels (logical px).
    ///
    /// This is used to avoid emitting `SceneOp`s for far-offscreen nodes/edges in large graphs,
    /// while keeping a small prefetch band to reduce pop-in during panning.
    pub render_cull_margin_px: f32,

    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for NodeGraphStyle {
    fn default() -> Self {
        Self {
            background: Color {
                r: 0.08,
                g: 0.09,
                b: 0.10,
                a: 1.0,
            },

            grid_spacing: 64.0,
            grid_minor_color: Color {
                r: 0.14,
                g: 0.15,
                b: 0.16,
                a: 1.0,
            },
            grid_major_every: 4,
            grid_major_color: Color {
                r: 0.20,
                g: 0.21,
                b: 0.22,
                a: 1.0,
            },

            node_width: 220.0,
            node_header_height: 28.0,
            node_padding: 10.0,
            pin_row_height: 22.0,
            pin_radius: 6.0,

            node_background: Color {
                r: 0.12,
                g: 0.13,
                b: 0.14,
                a: 1.0,
            },
            node_border: Color {
                r: 0.24,
                g: 0.25,
                b: 0.26,
                a: 1.0,
            },
            node_border_selected: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },

            group_background: Color {
                r: 0.10,
                g: 0.11,
                b: 0.12,
                a: 0.25,
            },
            group_border: Color {
                r: 0.24,
                g: 0.25,
                b: 0.26,
                a: 0.90,
            },

            resize_handle_size: 12.0,
            resize_handle_background: Color {
                r: 0.14,
                g: 0.15,
                b: 0.16,
                a: 0.95,
            },
            resize_handle_border: Color {
                r: 0.60,
                g: 0.62,
                b: 0.64,
                a: 0.90,
            },

            pin_color_data: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
            pin_color_exec: Color {
                r: 0.95,
                g: 0.75,
                b: 0.20,
                a: 1.0,
            },

            wire_width: 3.0,
            wire_interaction_width: 14.0,
            wire_color_data: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
            wire_color_exec: Color {
                r: 0.95,
                g: 0.75,
                b: 0.20,
                a: 1.0,
            },
            wire_color_preview: Color {
                r: 0.95,
                g: 0.95,
                b: 0.95,
                a: 0.85,
            },

            wire_width_selected_mul: 1.6,
            wire_width_hover_mul: 1.25,

            marquee_fill: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.18,
            },
            marquee_border: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.90,
            },
            marquee_border_width: 1.0,

            snapline_color: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.90,
            },
            snapline_width: 1.0,

            context_menu_width: 200.0,
            context_menu_padding: 10.0,
            context_menu_item_height: 26.0,
            context_menu_corner_radius: 6.0,
            context_menu_background: Color {
                r: 0.10,
                g: 0.11,
                b: 0.12,
                a: 0.98,
            },
            context_menu_border: Color {
                r: 0.26,
                g: 0.27,
                b: 0.28,
                a: 1.0,
            },
            context_menu_hover_background: Color {
                r: 0.17,
                g: 0.18,
                b: 0.19,
                a: 1.0,
            },
            context_menu_text: Color {
                r: 0.92,
                g: 0.93,
                b: 0.94,
                a: 1.0,
            },
            context_menu_text_disabled: Color {
                r: 0.60,
                g: 0.62,
                b: 0.64,
                a: 1.0,
            },
            context_menu_text_style: TextStyle {
                size: Px(13.0),
                ..TextStyle::default()
            },

            minimap_width: 220.0,
            minimap_height: 140.0,
            minimap_margin: 12.0,
            minimap_world_padding: 48.0,

            controls_button_size: 30.0,
            controls_gap: 6.0,
            controls_margin: 12.0,
            controls_padding: 6.0,
            controls_text: Color {
                r: 0.92,
                g: 0.93,
                b: 0.94,
                a: 1.0,
            },
            controls_text_style: TextStyle {
                size: Px(12.0),
                ..TextStyle::default()
            },
            controls_hover_background: Color {
                r: 0.17,
                g: 0.18,
                b: 0.19,
                a: 1.0,
            },
            controls_active_background: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.25,
            },

            render_cull_margin_px: 256.0,

            min_zoom: 0.15,
            max_zoom: 4.0,
        }
    }
}

impl NodeGraphStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self::from_snapshot(theme.snapshot())
    }

    pub fn from_snapshot(theme: ThemeSnapshot) -> Self {
        fn alpha(mut c: Color, a: f32) -> Color {
            c.a = a;
            c
        }

        let background = theme.color_required("background");
        let border = theme.color_required("border");
        let ring = theme.color_required("ring");
        let card = theme.color_required("card");
        let popover = theme.color_required("popover");
        let popover_border = theme.color_required("popover.border");
        let popover_foreground = theme.color_required("popover-foreground");
        let accent = theme.color_required("accent");
        let muted_foreground = theme.color_required("muted-foreground");

        let padding_sm = theme.metric_required("metric.padding.sm").0;
        let padding_md = theme.metric_required("metric.padding.md").0;
        let radius_sm = theme.metric_required("metric.radius.sm").0;
        let font_size = theme.metric_required("metric.font.size").0;

        let pin_color_data = theme.color_required("primary");
        let pin_color_exec = theme.colors.viewport_rotate_gizmo;

        Self {
            background,

            grid_spacing: 64.0,
            grid_minor_color: alpha(border, 0.32),
            grid_major_every: 4,
            grid_major_color: alpha(border, 0.52),

            node_width: 220.0,
            node_header_height: 28.0,
            node_padding: padding_md,
            pin_row_height: 22.0,
            pin_radius: radius_sm,

            node_background: card,
            node_border: border,
            node_border_selected: alpha(ring, 1.0),

            group_background: alpha(card, 0.25),
            group_border: alpha(border, 0.90),

            resize_handle_size: 12.0,
            resize_handle_background: alpha(popover, 0.95),
            resize_handle_border: alpha(border, 0.90),

            pin_color_data,
            pin_color_exec,

            wire_width: 3.0,
            wire_interaction_width: 14.0,
            wire_color_data: pin_color_data,
            wire_color_exec: pin_color_exec,
            wire_color_preview: alpha(theme.color_required("foreground"), 0.85),

            wire_width_selected_mul: 1.6,
            wire_width_hover_mul: 1.25,

            marquee_fill: theme.colors.viewport_selection_fill,
            marquee_border: theme.colors.viewport_selection_stroke,
            marquee_border_width: 1.0,

            snapline_color: theme.colors.viewport_marker,
            snapline_width: 1.0,

            context_menu_width: 200.0,
            context_menu_padding: padding_sm.max(6.0),
            context_menu_item_height: 26.0,
            context_menu_corner_radius: radius_sm,
            context_menu_background: alpha(popover, 0.98),
            context_menu_border: alpha(popover_border, 1.0),
            context_menu_hover_background: accent,
            context_menu_text: popover_foreground,
            context_menu_text_disabled: muted_foreground,
            context_menu_text_style: TextStyle {
                size: Px(font_size),
                ..TextStyle::default()
            },

            minimap_width: 220.0,
            minimap_height: 140.0,
            minimap_margin: padding_md.max(10.0),
            minimap_world_padding: 48.0,

            controls_button_size: 30.0,
            controls_gap: padding_sm.max(6.0),
            controls_margin: padding_md.max(10.0),
            controls_padding: padding_sm.max(6.0),
            controls_text: popover_foreground,
            controls_text_style: TextStyle {
                size: Px(font_size),
                ..TextStyle::default()
            },
            controls_hover_background: accent,
            controls_active_background: alpha(ring, 0.22),

            render_cull_margin_px: 256.0,

            min_zoom: 0.15,
            max_zoom: 4.0,
        }
    }
}
