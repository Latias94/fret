use fret_core::{FontId, Px, TextStyle};
use fret_ui::Theme;

/// Component sizing vocabulary inspired by Tailwind/shadcn and gpui-component.
///
/// This is intentionally a component-ecosystem concept (not a `fret-ui` contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
}

impl Size {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::XSmall => "xs",
            Self::Small => "sm",
            Self::Medium => "md",
            Self::Large => "lg",
        }
    }

    fn metric(self, theme: &Theme, suffix: &'static str, fallback: Px) -> Px {
        let key = match (self, suffix) {
            (Self::XSmall, "control.text_px") => "component.size.xs.control.text_px",
            (Self::Small, "control.text_px") => "component.size.sm.control.text_px",
            (Self::Medium, "control.text_px") => "component.size.md.control.text_px",
            (Self::Large, "control.text_px") => "component.size.lg.control.text_px",

            (Self::XSmall, "control.radius") => "component.size.xs.control.radius",
            (Self::Small, "control.radius") => "component.size.sm.control.radius",
            (Self::Medium, "control.radius") => "component.size.md.control.radius",
            (Self::Large, "control.radius") => "component.size.lg.control.radius",

            (Self::XSmall, "input.px") => "component.size.xs.input.px",
            (Self::Small, "input.px") => "component.size.sm.input.px",
            (Self::Medium, "input.px") => "component.size.md.input.px",
            (Self::Large, "input.px") => "component.size.lg.input.px",

            (Self::XSmall, "input.py") => "component.size.xs.input.py",
            (Self::Small, "input.py") => "component.size.sm.input.py",
            (Self::Medium, "input.py") => "component.size.md.input.py",
            (Self::Large, "input.py") => "component.size.lg.input.py",

            (Self::XSmall, "input.h") => "component.size.xs.input.h",
            (Self::Small, "input.h") => "component.size.sm.input.h",
            (Self::Medium, "input.h") => "component.size.md.input.h",
            (Self::Large, "input.h") => "component.size.lg.input.h",

            (Self::XSmall, "button.px") => "component.size.xs.button.px",
            (Self::Small, "button.px") => "component.size.sm.button.px",
            (Self::Medium, "button.px") => "component.size.md.button.px",
            (Self::Large, "button.px") => "component.size.lg.button.px",

            (Self::XSmall, "button.py") => "component.size.xs.button.py",
            (Self::Small, "button.py") => "component.size.sm.button.py",
            (Self::Medium, "button.py") => "component.size.md.button.py",
            (Self::Large, "button.py") => "component.size.lg.button.py",

            (Self::XSmall, "button.h") => "component.size.xs.button.h",
            (Self::Small, "button.h") => "component.size.sm.button.h",
            (Self::Medium, "button.h") => "component.size.md.button.h",
            (Self::Large, "button.h") => "component.size.lg.button.h",

            (Self::XSmall, "icon_button.size") => "component.size.xs.icon_button.size",
            (Self::Small, "icon_button.size") => "component.size.sm.icon_button.size",
            (Self::Medium, "icon_button.size") => "component.size.md.icon_button.size",
            (Self::Large, "icon_button.size") => "component.size.lg.icon_button.size",

            (Self::XSmall, "list.px") => "component.size.xs.list.px",
            (Self::Small, "list.px") => "component.size.sm.list.px",
            (Self::Medium, "list.px") => "component.size.md.list.px",
            (Self::Large, "list.px") => "component.size.lg.list.px",

            (Self::XSmall, "list.py") => "component.size.xs.list.py",
            (Self::Small, "list.py") => "component.size.sm.list.py",
            (Self::Medium, "list.py") => "component.size.md.list.py",
            (Self::Large, "list.py") => "component.size.lg.list.py",

            (Self::XSmall, "list.row_h") => "component.size.xs.list.row_h",
            (Self::Small, "list.row_h") => "component.size.sm.list.row_h",
            (Self::Medium, "list.row_h") => "component.size.md.list.row_h",
            (Self::Large, "list.row_h") => "component.size.lg.list.row_h",

            _ => return fallback,
        };

        theme.metric_by_key(key).unwrap_or(fallback)
    }

    pub fn control_text_px(self, theme: &Theme) -> Px {
        let base = theme
            .metric_by_key("font.size")
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let fallback = match self {
            // Keep the current defaults when `metric.font.size == 13px`,
            // while allowing themes to scale typography globally.
            Self::XSmall => base - Px(1.0),
            Self::Small => base,
            Self::Medium => base,
            Self::Large => base + Px(1.0),
        };
        self.metric(theme, "control.text_px", fallback)
    }

    pub fn control_text_style(self, theme: &Theme) -> TextStyle {
        crate::typography::control_text_style_scaled(
            theme,
            FontId::ui(),
            self.control_text_px(theme),
        )
    }

    pub fn control_radius(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "control.radius",
            match self {
                Self::XSmall => theme.metric_token("metric.radius.sm"),
                Self::Small => theme.metric_token("metric.radius.sm"),
                Self::Medium => theme.metric_token("metric.radius.md"),
                Self::Large => theme.metric_token("metric.radius.md"),
            },
        )
    }

    pub fn input_px(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "input.px",
            match self {
                Self::XSmall => Px(8.0),
                Self::Small => Px(10.0),
                Self::Medium => Px(12.0),
                Self::Large => Px(14.0),
            },
        )
    }

    pub fn input_py(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "input.py",
            match self {
                Self::XSmall => Px(4.0),
                Self::Small => Px(5.0),
                Self::Medium => Px(6.0),
                Self::Large => Px(7.0),
            },
        )
    }

    pub fn input_h(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "input.h",
            match self {
                Self::XSmall => Px(24.0),
                Self::Small => Px(28.0),
                Self::Medium => Px(32.0),
                Self::Large => Px(36.0),
            },
        )
    }

    pub fn button_px(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "button.px",
            match self {
                Self::XSmall => Px(8.0),
                Self::Small => Px(10.0),
                Self::Medium => Px(12.0),
                Self::Large => Px(14.0),
            },
        )
    }

    pub fn button_py(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "button.py",
            match self {
                Self::XSmall => Px(4.0),
                Self::Small => Px(5.0),
                Self::Medium => Px(6.0),
                Self::Large => Px(7.0),
            },
        )
    }

    pub fn button_h(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "button.h",
            match self {
                Self::XSmall => Px(24.0),
                Self::Small => Px(28.0),
                Self::Medium => Px(32.0),
                Self::Large => Px(36.0),
            },
        )
    }

    pub fn icon_button_size(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "icon_button.size",
            match self {
                Self::XSmall => Px(24.0),
                Self::Small => Px(28.0),
                Self::Medium => Px(32.0),
                Self::Large => Px(36.0),
            },
        )
    }

    pub fn list_px(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "list.px",
            match self {
                // Align with the Tailwind-like scales used by gpui-component:
                // - `px-2` for dense lists,
                // - `px-3` for default/comfortable lists.
                Self::XSmall => Px(8.0),
                Self::Small => Px(8.0),
                Self::Medium => Px(12.0),
                Self::Large => Px(12.0),
            },
        )
    }

    pub fn list_py(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "list.py",
            match self {
                // Align with gpui-component list defaults (py-0.5/py-1/py-2).
                Self::XSmall => Px(2.0),
                Self::Small => Px(2.0),
                Self::Medium => Px(4.0),
                Self::Large => Px(8.0),
            },
        )
    }

    pub fn list_row_h(self, theme: &Theme) -> Px {
        self.metric(
            theme,
            "list.row_h",
            match self {
                Self::XSmall => Px(24.0),
                Self::Small => Px(28.0),
                Self::Medium => Px(32.0),
                Self::Large => Px(36.0),
            },
        )
    }
}

/// Shared component API for size configuration.
pub trait Sizable: Sized {
    fn with_size(self, size: Size) -> Self;

    fn xsmall(self) -> Self {
        self.with_size(Size::XSmall)
    }

    fn small(self) -> Self {
        self.with_size(Size::Small)
    }

    fn medium(self) -> Self {
        self.with_size(Size::Medium)
    }

    fn large(self) -> Self {
        self.with_size(Size::Large)
    }
}
