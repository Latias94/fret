#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeColorKey {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
    RingOffsetBackground,
}

impl ThemeColorKey {
    pub const ALL: &'static [ThemeColorKey] = &[
        ThemeColorKey::Background,
        ThemeColorKey::Foreground,
        ThemeColorKey::Card,
        ThemeColorKey::CardForeground,
        ThemeColorKey::Popover,
        ThemeColorKey::PopoverForeground,
        ThemeColorKey::Primary,
        ThemeColorKey::PrimaryForeground,
        ThemeColorKey::Secondary,
        ThemeColorKey::SecondaryForeground,
        ThemeColorKey::Muted,
        ThemeColorKey::MutedForeground,
        ThemeColorKey::Accent,
        ThemeColorKey::AccentForeground,
        ThemeColorKey::Destructive,
        ThemeColorKey::DestructiveForeground,
        ThemeColorKey::Border,
        ThemeColorKey::Input,
        ThemeColorKey::Ring,
        ThemeColorKey::RingOffsetBackground,
    ];

    pub const fn canonical_name(self) -> &'static str {
        match self {
            ThemeColorKey::Background => "background",
            ThemeColorKey::Foreground => "foreground",
            ThemeColorKey::Card => "card",
            ThemeColorKey::CardForeground => "card-foreground",
            ThemeColorKey::Popover => "popover",
            ThemeColorKey::PopoverForeground => "popover-foreground",
            ThemeColorKey::Primary => "primary",
            ThemeColorKey::PrimaryForeground => "primary-foreground",
            ThemeColorKey::Secondary => "secondary",
            ThemeColorKey::SecondaryForeground => "secondary-foreground",
            ThemeColorKey::Muted => "muted",
            ThemeColorKey::MutedForeground => "muted-foreground",
            ThemeColorKey::Accent => "accent",
            ThemeColorKey::AccentForeground => "accent-foreground",
            ThemeColorKey::Destructive => "destructive",
            ThemeColorKey::DestructiveForeground => "destructive-foreground",
            ThemeColorKey::Border => "border",
            ThemeColorKey::Input => "input",
            ThemeColorKey::Ring => "ring",
            ThemeColorKey::RingOffsetBackground => "ring-offset-background",
        }
    }

    pub fn from_canonical_name(name: &str) -> Option<Self> {
        match name {
            "background" => Some(Self::Background),
            "foreground" => Some(Self::Foreground),
            "card" => Some(Self::Card),
            "card-foreground" => Some(Self::CardForeground),
            "popover" => Some(Self::Popover),
            "popover-foreground" => Some(Self::PopoverForeground),
            "primary" => Some(Self::Primary),
            "primary-foreground" => Some(Self::PrimaryForeground),
            "secondary" => Some(Self::Secondary),
            "secondary-foreground" => Some(Self::SecondaryForeground),
            "muted" => Some(Self::Muted),
            "muted-foreground" => Some(Self::MutedForeground),
            "accent" => Some(Self::Accent),
            "accent-foreground" => Some(Self::AccentForeground),
            "destructive" => Some(Self::Destructive),
            "destructive-foreground" => Some(Self::DestructiveForeground),
            "border" => Some(Self::Border),
            "input" => Some(Self::Input),
            "ring" => Some(Self::Ring),
            "ring-offset-background" => Some(Self::RingOffsetBackground),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeMetricKey {
    Radius,
    FontSize,
    FontLineHeight,
    MonoFontSize,
    MonoFontLineHeight,
}

impl ThemeMetricKey {
    pub const ALL: &'static [ThemeMetricKey] = &[
        ThemeMetricKey::Radius,
        ThemeMetricKey::FontSize,
        ThemeMetricKey::FontLineHeight,
        ThemeMetricKey::MonoFontSize,
        ThemeMetricKey::MonoFontLineHeight,
    ];

    pub const fn canonical_name(self) -> &'static str {
        match self {
            ThemeMetricKey::Radius => "radius",
            ThemeMetricKey::FontSize => "font.size",
            ThemeMetricKey::FontLineHeight => "font.line_height",
            ThemeMetricKey::MonoFontSize => "mono_font.size",
            ThemeMetricKey::MonoFontLineHeight => "mono_font.line_height",
        }
    }

    pub fn from_canonical_name(name: &str) -> Option<Self> {
        match name {
            "radius" => Some(Self::Radius),
            "font.size" => Some(Self::FontSize),
            "font.line_height" => Some(Self::FontLineHeight),
            "mono_font.size" => Some(Self::MonoFontSize),
            "mono_font.line_height" => Some(Self::MonoFontLineHeight),
            _ => None,
        }
    }
}
