use fret_core::{Color, Px, TextStyle};
use fret_ui::{Theme, ThemeSnapshot};

/// Background/theming configuration for the node graph canvas.
///
/// This is intentionally policy-light: it is a pure token/config bundle that can be
/// stored in a B-layer store and applied without touching interaction logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeGraphBackgroundStyle {
    pub background: Color,

    pub grid_pattern: NodeGraphBackgroundPattern,
    pub grid_spacing: f32,
    pub grid_minor_color: Color,
    pub grid_major_every: u32,
    pub grid_major_color: Color,
    /// Grid stroke thickness in screen pixels (XyFlow `BackgroundProps.lineWidth`).
    pub grid_line_width: f32,
    /// Dot diameter in canvas units at zoom=1 (XyFlow `BackgroundProps.size` for dots).
    pub grid_dot_size: f32,
    /// Cross size in canvas units at zoom=1 (XyFlow `BackgroundProps.size` for cross).
    pub grid_cross_size: f32,
}

/// Background grid pattern variant (XyFlow `BackgroundVariant`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphBackgroundPattern {
    Lines,
    Dots,
    Cross,
}

impl Default for NodeGraphBackgroundPattern {
    fn default() -> Self {
        Self::Lines
    }
}

/// Color mode override for the node graph canvas (XyFlow `colorMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphColorMode {
    /// Use the host UI theme (default).
    System,
    /// Force a light palette regardless of the host theme.
    Light,
    /// Force a dark palette regardless of the host theme.
    Dark,
}

impl Default for NodeGraphColorMode {
    fn default() -> Self {
        Self::System
    }
}

/// Visual tuning for the node graph canvas.
#[derive(Debug, Clone)]
pub struct NodeGraphStyle {
    pub background: Color,

    pub grid_pattern: NodeGraphBackgroundPattern,
    pub grid_spacing: f32,
    pub grid_minor_color: Color,
    pub grid_major_every: u32,
    pub grid_major_color: Color,
    /// Grid stroke thickness in screen pixels (XyFlow `BackgroundProps.lineWidth`).
    pub grid_line_width: f32,
    /// Dot diameter in canvas units at zoom=1 (XyFlow `BackgroundProps.size` for dots).
    pub grid_dot_size: f32,
    /// Cross size in canvas units at zoom=1 (XyFlow `BackgroundProps.size` for cross).
    pub grid_cross_size: f32,

    pub node_width: f32,
    pub node_header_height: f32,
    pub node_padding: f32,
    pub node_corner_radius: f32,
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

    /// Edge label padding in screen-space pixels (logical px).
    pub edge_label_padding: f32,
    /// Edge label corner radius in screen-space pixels (logical px).
    pub edge_label_corner_radius: f32,
    /// Edge label offset distance from the edge in screen-space pixels (logical px).
    pub edge_label_offset: f32,
    /// Edge label max width in screen-space pixels (logical px).
    pub edge_label_max_width: f32,
    /// Edge label background color.
    pub edge_label_background: Color,
    /// Edge label border color (used when `EdgeRenderHint.color` is not set).
    pub edge_label_border: Color,
    /// Edge label border width in screen-space pixels (logical px).
    pub edge_label_border_width: f32,
    /// Edge label text color.
    pub edge_label_text: Color,
    /// Edge label text style.
    pub edge_label_text_style: TextStyle,

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
            background: Color::from_srgb_hex_rgb(0x14_17_1a),

            grid_pattern: NodeGraphBackgroundPattern::Lines,
            grid_spacing: 64.0,
            grid_minor_color: Color::from_srgb_hex_rgb(0x24_26_29),
            grid_major_every: 4,
            grid_major_color: Color::from_srgb_hex_rgb(0x33_36_38),
            grid_line_width: 1.0,
            grid_dot_size: 1.0,
            grid_cross_size: 6.0,

            node_width: 220.0,
            node_header_height: 28.0,
            node_padding: 10.0,
            node_corner_radius: 8.0,
            pin_row_height: 22.0,
            pin_radius: 6.0,

            node_background: Color::from_srgb_hex_rgb(0x1f_21_24),
            node_border: Color::from_srgb_hex_rgb(0x3d_40_42),
            node_border_selected: Color::from_srgb_hex_rgb(0x33_8c_f2),

            group_background: Color {
                a: 0.25,
                ..Color::from_srgb_hex_rgb(0x1a_1c_1f)
            },
            group_border: Color {
                a: 0.90,
                ..Color::from_srgb_hex_rgb(0x3d_40_42)
            },

            resize_handle_size: 12.0,
            resize_handle_background: Color {
                a: 0.95,
                ..Color::from_srgb_hex_rgb(0x24_26_29)
            },
            resize_handle_border: Color {
                a: 0.90,
                ..Color::from_srgb_hex_rgb(0x99_9e_a3)
            },

            pin_color_data: Color::from_srgb_hex_rgb(0x33_8c_f2),
            pin_color_exec: Color::from_srgb_hex_rgb(0xf2_bf_33),

            wire_width: 3.0,
            wire_interaction_width: 14.0,
            wire_color_data: Color::from_srgb_hex_rgb(0x33_8c_f2),
            wire_color_exec: Color::from_srgb_hex_rgb(0xf2_bf_33),
            wire_color_preview: Color {
                a: 0.85,
                ..Color::from_srgb_hex_rgb(0xf2_f2_f2)
            },

            wire_width_selected_mul: 1.6,
            wire_width_hover_mul: 1.25,

            edge_label_padding: 6.0,
            edge_label_corner_radius: 8.0,
            edge_label_offset: 10.0,
            edge_label_max_width: 220.0,
            edge_label_background: Color {
                a: 0.98,
                ..Color::from_srgb_hex_rgb(0x24_26_29)
            },
            edge_label_border: Color {
                a: 0.90,
                ..Color::from_srgb_hex_rgb(0x99_9e_a3)
            },
            edge_label_border_width: 1.0,
            edge_label_text: Color::from_srgb_hex_rgb(0xeb_ed_f0),
            edge_label_text_style: TextStyle {
                size: Px(12.0),
                ..TextStyle::default()
            },

            marquee_fill: Color {
                a: 0.18,
                ..Color::from_srgb_hex_rgb(0x33_8c_f2)
            },
            marquee_border: Color {
                a: 0.90,
                ..Color::from_srgb_hex_rgb(0x33_8c_f2)
            },
            marquee_border_width: 1.0,

            snapline_color: Color {
                a: 0.90,
                ..Color::from_srgb_hex_rgb(0x33_8c_f2)
            },
            snapline_width: 1.0,

            context_menu_width: 200.0,
            context_menu_padding: 10.0,
            context_menu_item_height: 26.0,
            context_menu_corner_radius: 6.0,
            context_menu_background: Color {
                a: 0.98,
                ..Color::from_srgb_hex_rgb(0x1a_1c_1f)
            },
            context_menu_border: Color::from_srgb_hex_rgb(0x42_45_47),
            context_menu_hover_background: Color::from_srgb_hex_rgb(0x2b_2e_30),
            context_menu_text: Color::from_srgb_hex_rgb(0xeb_ed_f0),
            context_menu_text_disabled: Color::from_srgb_hex_rgb(0x99_9e_a3),
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
            controls_text: Color::from_srgb_hex_rgb(0xeb_ed_f0),
            controls_text_style: TextStyle {
                size: Px(12.0),
                ..TextStyle::default()
            },
            controls_hover_background: Color::from_srgb_hex_rgb(0x2b_2e_30),
            controls_active_background: Color {
                a: 0.25,
                ..Color::from_srgb_hex_rgb(0x33_8c_f2)
            },

            render_cull_margin_px: 256.0,

            min_zoom: 0.15,
            max_zoom: 4.0,
        }
    }
}

impl NodeGraphStyle {
    pub fn from_color_mode(theme: &Theme, mode: NodeGraphColorMode) -> Self {
        Self::from_snapshot_with_color_mode(theme.snapshot(), mode)
    }

    pub fn from_snapshot_with_color_mode(theme: ThemeSnapshot, mode: NodeGraphColorMode) -> Self {
        match mode {
            NodeGraphColorMode::System => Self::from_snapshot(theme),
            NodeGraphColorMode::Light => Self::light_defaults(),
            NodeGraphColorMode::Dark => Self::dark_defaults(),
        }
    }

    /// Light defaults tuned for node editor canvases.
    ///
    /// Note: This is intentionally neutral naming. See `docs/workstreams/xyflow-gap-analysis.md`
    /// for upstream comparisons.
    pub fn light_defaults() -> Self {
        let mut s = Self::default();

        s.background = Color::from_srgb_hex_rgb(0xfa_fa_fa);

        s.grid_minor_color = Color::from_srgb_hex_rgb(0xe6_e6_e6);
        s.grid_major_color = Color::from_srgb_hex_rgb(0xd6_d6_d6);

        s.node_background = Color::from_srgb_hex_rgb(0xff_ff_ff);
        s.node_border = Color::from_srgb_hex_rgb(0xc7_c7_c7);

        s.group_background = Color {
            a: 0.45,
            ..Color::from_srgb_hex_rgb(0xe6_e6_e6)
        };
        s.group_border = Color {
            a: 0.90,
            ..Color::from_srgb_hex_rgb(0xc7_c7_c7)
        };

        s.resize_handle_background = Color {
            a: 0.98,
            ..Color::from_srgb_hex_rgb(0xf5_f5_f5)
        };
        s.resize_handle_border = Color {
            a: 0.90,
            ..Color::from_srgb_hex_rgb(0xb3_b3_b3)
        };

        s.wire_color_preview = Color {
            a: 0.60,
            ..Color::from_srgb_hex_rgb(0x1a_1a_1a)
        };

        s.context_menu_background = Color {
            a: 0.98,
            ..Color::from_srgb_hex_rgb(0xff_ff_ff)
        };
        s.context_menu_border = Color::from_srgb_hex_rgb(0xc7_c7_c7);
        s.context_menu_hover_background = Color::from_srgb_hex_rgb(0xeb_f2_ff);
        s.context_menu_text = Color::from_srgb_hex_rgb(0x1f_1f_1f);
        s.context_menu_text_disabled = Color::from_srgb_hex_rgb(0x73_73_73);

        s.edge_label_background = s.context_menu_background;
        s.edge_label_border = s.context_menu_border;
        s.edge_label_text = s.context_menu_text;
        s.edge_label_text_style = s.context_menu_text_style.clone();

        s.controls_text = s.context_menu_text;
        s.controls_hover_background = s.context_menu_hover_background;
        s.controls_active_background = Color {
            a: 0.22,
            ..Color::from_srgb_hex_rgb(0x33_8c_f2)
        };

        s
    }

    pub fn dark_defaults() -> Self {
        Self::default()
    }

    /// Applies compact node style defaults (width/padding/radius/pin size/font size).
    ///
    /// This only touches node-related sizing/chrome fields. Colors remain unchanged so callers can
    /// combine it with theme-driven palettes or `colorMode` overrides.
    pub fn apply_compact_node_style(&mut self) {
        self.node_width = 150.0;
        self.node_padding = 10.0;
        self.node_corner_radius = 3.0;
        self.pin_radius = 3.0;
        self.context_menu_text_style.size = Px(12.0);
        self.edge_label_text_style.size = Px(12.0);
    }

    pub fn with_compact_node_style(mut self) -> Self {
        self.apply_compact_node_style();
        self
    }

    pub fn from_theme(theme: &Theme) -> Self {
        Self::from_snapshot(theme.snapshot())
    }

    pub fn from_snapshot(theme: ThemeSnapshot) -> Self {
        fn alpha(mut c: Color, a: f32) -> Color {
            c.a = a;
            c
        }

        let background = theme.color_token("background");
        let border = theme.color_token("border");
        let ring = theme.color_token("ring");
        let card = theme.color_token("card");
        let popover = theme.color_token("popover");
        let popover_border = theme.color_token("popover.border");
        let popover_foreground = theme.color_token("popover-foreground");
        let accent = theme.color_token("accent");
        let muted_foreground = theme.color_token("muted-foreground");

        let padding_sm = theme.metric_token("metric.padding.sm").0;
        let padding_md = theme.metric_token("metric.padding.md").0;
        let radius_sm = theme.metric_token("metric.radius.sm").0;
        let font_size = theme.metric_token("metric.font.size").0;

        let pin_color_data = theme.color_token("primary");
        let pin_color_exec = theme.colors.viewport_rotate_gizmo;

        Self {
            background,

            grid_pattern: NodeGraphBackgroundPattern::Lines,
            grid_spacing: 64.0,
            grid_minor_color: alpha(border, 0.32),
            grid_major_every: 4,
            grid_major_color: alpha(border, 0.52),
            grid_line_width: 1.0,
            grid_dot_size: 1.0,
            grid_cross_size: 6.0,

            node_width: 220.0,
            node_header_height: 28.0,
            node_padding: padding_md,
            node_corner_radius: radius_sm.max(8.0),
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
            wire_color_preview: alpha(theme.color_token("foreground"), 0.85),

            wire_width_selected_mul: 1.6,
            wire_width_hover_mul: 1.25,

            edge_label_padding: padding_sm.max(6.0),
            edge_label_corner_radius: radius_sm,
            edge_label_offset: 10.0,
            edge_label_max_width: 220.0,
            edge_label_background: alpha(popover, 0.98),
            edge_label_border: alpha(popover_border, 1.0),
            edge_label_border_width: 1.0,
            edge_label_text: popover_foreground,
            edge_label_text_style: TextStyle {
                size: Px(font_size),
                ..TextStyle::default()
            },

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

    pub fn background_style(&self) -> NodeGraphBackgroundStyle {
        NodeGraphBackgroundStyle {
            background: self.background,
            grid_pattern: self.grid_pattern,
            grid_spacing: self.grid_spacing,
            grid_minor_color: self.grid_minor_color,
            grid_major_every: self.grid_major_every,
            grid_major_color: self.grid_major_color,
            grid_line_width: self.grid_line_width,
            grid_dot_size: self.grid_dot_size,
            grid_cross_size: self.grid_cross_size,
        }
    }

    pub fn with_background_style(mut self, background: NodeGraphBackgroundStyle) -> Self {
        self.background = background.background;
        self.grid_pattern = background.grid_pattern;
        self.grid_spacing = background.grid_spacing;
        self.grid_minor_color = background.grid_minor_color;
        self.grid_major_every = background.grid_major_every;
        self.grid_major_color = background.grid_major_color;
        self.grid_line_width = background.grid_line_width;
        self.grid_dot_size = background.grid_dot_size;
        self.grid_cross_size = background.grid_cross_size;
        self
    }

    /// Fingerprint of geometry-affecting tokens.
    ///
    /// This is intended for cache invalidation decisions (derived geometry / spatial index).
    /// It must stay stable across platforms and should only include tokens that change node/port
    /// bounds or anchors.
    pub fn geometry_fingerprint(&self) -> u64 {
        fn mix_u64(mut state: u64, value: u64) -> u64 {
            state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
            state = state.rotate_left(7);
            state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
            state
        }

        fn mix_f32(state: u64, value: f32) -> u64 {
            mix_u64(state, u64::from(value.to_bits()))
        }

        let mut state = 0u64;
        state = mix_f32(state, self.node_width);
        state = mix_f32(state, self.node_header_height);
        state = mix_f32(state, self.node_padding);
        state = mix_f32(state, self.pin_row_height);
        state = mix_f32(state, self.pin_radius);
        state
    }
}
