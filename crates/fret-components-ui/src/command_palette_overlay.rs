use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Px, Rect, SceneOp, Size,
};
use fret_runtime::CommandId;
use fret_ui::{
    Theme, UiHost,
    widget::{CommandCx, EventCx, LayoutCx, PaintCx, SemanticsCx, Widget},
};

#[derive(Debug, Clone)]
pub struct CommandPaletteStyle {
    pub backdrop_opacity: f32,
    pub max_width: Px,
    pub max_height: Px,
    pub top: Px,
    pub padding: Px,
    pub corner_radius: Px,
    pub border: Edges,
    pub background: Color,
    pub border_color: Color,
}

impl Default for CommandPaletteStyle {
    fn default() -> Self {
        Self {
            backdrop_opacity: 0.55,
            max_width: Px(560.0),
            max_height: Px(360.0),
            top: Px(72.0),
            padding: Px(10.0),
            corner_radius: Px(10.0),
            border: Edges::all(Px(1.0)),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// Modal command palette overlay layout shell.
///
/// This widget:
/// - paints a backdrop + a centered panel,
/// - lays out its children inside the panel (single content root recommended),
/// - closes on `Escape` and clicking outside the panel by dispatching `command_palette.close`.
///
/// Notes:
/// - The open/close + focus restoration policy is managed by `WindowOverlays`.
/// - The palette content (input + list) is provided by the app/component layer.
pub struct CommandPaletteOverlay {
    style: CommandPaletteStyle,
    close_command: CommandId,
    last_theme_revision: Option<u64>,
    panel_bounds: Rect,
}

impl CommandPaletteOverlay {
    pub fn new() -> Self {
        Self {
            style: CommandPaletteStyle::default(),
            close_command: CommandId::from("command_palette.close"),
            last_theme_revision: None,
            panel_bounds: Rect::default(),
        }
    }

    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = command;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.style.padding = theme.metrics.padding_md;
        self.style.corner_radius = theme.metrics.radius_lg;
        self.style.background = theme.colors.menu_background;
        self.style.border_color = theme.colors.menu_border;
    }

    fn compute_panel_bounds(&self, outer: Rect, available: Size) -> Rect {
        let max_w = self.style.max_width.0.max(0.0);
        let w = max_w.min(available.width.0).max(0.0);

        let max_h = self.style.max_height.0.max(0.0);
        let h = max_h
            .min((available.height.0 - self.style.top.0).max(0.0))
            .max(0.0);

        let x = outer.origin.x.0 + (available.width.0 - w) * 0.5;
        let y = outer.origin.y.0 + self.style.top.0;

        Rect::new(fret_core::Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }
}

impl Default for CommandPaletteOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for CommandPaletteOverlay {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if *key == KeyCode::Escape
                    && !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.meta
                {
                    cx.dispatch_command(self.close_command.clone());
                    cx.stop_propagation();
                }
            }
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button == MouseButton::Left && !self.panel_bounds.contains(*position) {
                        cx.dispatch_command(self.close_command.clone());
                        cx.stop_propagation();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());

        self.panel_bounds = self.compute_panel_bounds(cx.bounds, cx.available);

        let pad = self.style.padding.0.max(0.0);
        let inner = Rect::new(
            fret_core::Point::new(
                Px(self.panel_bounds.origin.x.0 + pad),
                Px(self.panel_bounds.origin.y.0 + pad),
            ),
            Size::new(
                Px((self.panel_bounds.size.width.0 - pad * 2.0).max(0.0)),
                Px((self.panel_bounds.size.height.0 - pad * 2.0).max(0.0)),
            ),
        );

        if let Some(&content) = cx.children.first() {
            let _ = cx.layout_in(content, inner);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());

        let base = cx.theme().colors.surface_background;
        let backdrop_opacity = self.style.backdrop_opacity.clamp(0.0, 1.0);
        let backdrop = Color {
            a: backdrop_opacity,
            ..base
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: backdrop,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: self.panel_bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: Corners::all(self.style.corner_radius),
        });

        if let Some(&content) = cx.children.first() {
            if let Some(bounds) = cx.child_bounds(content) {
                cx.paint(content, bounds);
            } else {
                cx.paint(content, cx.bounds);
            }
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        match command.as_str() {
            "command_palette.close" => {
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }

    fn cleanup_resources(&mut self, _text: &mut dyn fret_core::TextService) {
        self.last_theme_revision = None;
        self.panel_bounds = Rect::default();
    }

    fn semantics(&mut self, _cx: &mut SemanticsCx<'_, H>) {
        // Modal overlay semantics are app-specific; defer for now.
    }
}
