use fret_core::Color;

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

    pub pin_color_data: Color,
    pub pin_color_exec: Color,

    pub wire_width: f32,
    pub wire_interaction_width: f32,
    pub wire_color_data: Color,
    pub wire_color_exec: Color,
    pub wire_color_preview: Color,

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

            min_zoom: 0.15,
            max_zoom: 4.0,
        }
    }
}
