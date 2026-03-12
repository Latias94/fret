use super::*;

impl UiGalleryDriver {
    pub(crate) fn motion_preset_theme_patch(preset: &str) -> fret_ui::ThemeConfig {
        let mut cfg = fret_ui::ThemeConfig::default();

        let linear = fret_ui::theme::CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        };
        let shadcn_ease_default = fret_ui::theme::CubicBezier {
            x1: 0.22,
            y1: 1.0,
            x2: 0.36,
            y2: 1.0,
        };
        let shadcn_ease = if preset == "reduced" {
            linear
        } else {
            shadcn_ease_default
        };

        let (
            scale_100,
            scale_200,
            scale_300,
            scale_500,
            overlay_open,
            overlay_close,
            sidebar_toggle,
            toast_enter,
            toast_exit,
        ) = match preset {
            "reduced" => (0, 0, 0, 0, 0, 0, 0, 0, 0),
            "snappy" => (80, 160, 240, 320, 160, 140, 160, 140, 110),
            "bouncy" => (100, 200, 300, 500, 200, 200, 200, 160, 120),
            "gentle" => (120, 220, 320, 560, 220, 200, 220, 180, 140),
            _ => (100, 200, 300, 500, 200, 200, 200, 160, 120),
        };

        let toast_stack_shift = toast_enter;
        let toast_stack_shift_stagger = match preset {
            "reduced" => 0,
            "snappy" => 16,
            "bouncy" => 20,
            "gentle" => 24,
            _ => 20,
        };

        cfg.durations_ms
            .insert("duration.shadcn.motion.100".to_string(), scale_100);
        cfg.durations_ms
            .insert("duration.shadcn.motion.200".to_string(), scale_200);
        cfg.durations_ms
            .insert("duration.shadcn.motion.300".to_string(), scale_300);
        cfg.durations_ms
            .insert("duration.shadcn.motion.500".to_string(), scale_500);

        cfg.durations_ms.insert(
            "duration.shadcn.motion.overlay.open".to_string(),
            overlay_open,
        );
        cfg.durations_ms.insert(
            "duration.shadcn.motion.overlay.close".to_string(),
            overlay_close,
        );
        cfg.durations_ms.insert(
            "duration.shadcn.motion.sidebar.toggle".to_string(),
            sidebar_toggle,
        );
        cfg.durations_ms.insert(
            "duration.shadcn.motion.collapsible.toggle".to_string(),
            sidebar_toggle,
        );
        cfg.durations_ms.insert(
            "duration.shadcn.motion.toast.enter".to_string(),
            toast_enter,
        );
        cfg.durations_ms
            .insert("duration.shadcn.motion.toast.exit".to_string(), toast_exit);
        cfg.durations_ms.insert(
            "duration.shadcn.motion.toast.stack.shift".to_string(),
            toast_stack_shift,
        );
        cfg.durations_ms.insert(
            "duration.shadcn.motion.toast.stack.shift.stagger".to_string(),
            toast_stack_shift_stagger,
        );

        cfg.easings
            .insert("easing.shadcn.motion".to_string(), shadcn_ease);
        cfg.easings
            .insert("easing.shadcn.motion.overlay".to_string(), shadcn_ease);
        cfg.easings
            .insert("easing.shadcn.motion.sidebar".to_string(), linear);
        cfg.easings.insert(
            "easing.shadcn.motion.collapsible.toggle".to_string(),
            shadcn_ease,
        );
        cfg.easings
            .insert("easing.shadcn.motion.toast".to_string(), shadcn_ease);
        cfg.easings.insert(
            "easing.shadcn.motion.toast.stack.shift".to_string(),
            shadcn_ease,
        );

        let (drawer_settle_duration, drawer_settle_bounce, inertia_bounce_bounce) = match preset {
            "reduced" => (0, 0.0, 0.0),
            "snappy" => (210, 0.0, 0.2),
            "bouncy" => (260, 0.35, 0.4),
            "gentle" => (280, 0.1, 0.25),
            _ => (240, 0.0, 0.25),
        };

        cfg.durations_ms.insert(
            "duration.shadcn.motion.spring.drawer.settle".to_string(),
            drawer_settle_duration,
        );
        cfg.numbers.insert(
            "number.shadcn.motion.spring.drawer.settle.bounce".to_string(),
            drawer_settle_bounce,
        );
        cfg.durations_ms.insert(
            "duration.shadcn.motion.spring.drawer.inertia_bounce".to_string(),
            drawer_settle_duration,
        );
        cfg.numbers.insert(
            "number.shadcn.motion.spring.drawer.inertia_bounce.bounce".to_string(),
            inertia_bounce_bounce,
        );

        cfg.durations_ms
            .insert("duration.motion.presence.enter".to_string(), overlay_open);
        cfg.durations_ms
            .insert("duration.motion.presence.exit".to_string(), overlay_close);
        cfg.durations_ms.insert(
            "duration.motion.collapsible.toggle".to_string(),
            sidebar_toggle,
        );
        cfg.durations_ms
            .insert("duration.motion.layout.expand".to_string(), sidebar_toggle);
        cfg.durations_ms
            .insert("duration.motion.stack.shift".to_string(), toast_stack_shift);
        cfg.durations_ms.insert(
            "duration.motion.stack.shift.stagger".to_string(),
            toast_stack_shift_stagger,
        );
        cfg.durations_ms.insert(
            "duration.motion.spring.drag_release_settle".to_string(),
            drawer_settle_duration,
        );
        cfg.numbers.insert(
            "number.motion.spring.drag_release_settle.bounce".to_string(),
            inertia_bounce_bounce,
        );
        cfg.easings
            .insert("easing.motion.standard".to_string(), shadcn_ease);
        cfg.easings
            .insert("easing.motion.emphasized".to_string(), shadcn_ease);
        cfg.easings
            .insert("easing.motion.collapsible.toggle".to_string(), shadcn_ease);
        cfg.easings
            .insert("easing.motion.layout.expand".to_string(), shadcn_ease);
        cfg.easings
            .insert("easing.motion.stack.shift".to_string(), shadcn_ease);

        cfg
    }

    pub(crate) fn sync_shadcn_theme(app: &mut App, state: &mut UiGalleryWindowState) {
        let preset = app.models().get_cloned(&state.theme_preset).flatten();
        if preset.as_deref() == state.applied_theme_preset.as_deref() {
            return;
        }

        let Some(preset) = preset else {
            return;
        };

        let Some((base, scheme)) = preset.split_once('/') else {
            return;
        };

        let base = match base {
            "neutral" => shadcn::themes::ShadcnBaseColor::Neutral,
            "zinc" => shadcn::themes::ShadcnBaseColor::Zinc,
            "slate" => shadcn::themes::ShadcnBaseColor::Slate,
            "stone" => shadcn::themes::ShadcnBaseColor::Stone,
            "gray" => shadcn::themes::ShadcnBaseColor::Gray,
            _ => return,
        };

        let scheme = match scheme {
            "light" => shadcn::themes::ShadcnColorScheme::Light,
            "dark" => shadcn::themes::ShadcnColorScheme::Dark,
            _ => return,
        };

        shadcn::themes::apply_shadcn_new_york(app, base, scheme);

        #[cfg(feature = "gallery-material3")]
        fret_ui::Theme::with_global_mut(app, |theme| {
            let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
                fret_ui_material3::tokens::v30::TypographyOptions::default(),
                fret_ui_material3::tokens::v30::ColorSchemeOptions {
                    mode: match scheme {
                        shadcn::themes::ShadcnColorScheme::Light => {
                            fret_ui_material3::tokens::v30::SchemeMode::Light
                        }
                        shadcn::themes::ShadcnColorScheme::Dark => {
                            fret_ui_material3::tokens::v30::SchemeMode::Dark
                        }
                    },
                    ..Default::default()
                },
            );
            theme.extend_tokens_from_config(&cfg);
        });

        let _ = app
            .models_mut()
            .update(&state.theme_preset_open, |open| *open = false);
        let _ = app
            .models_mut()
            .update(&state.motion_preset_open, |open| *open = false);

        state.applied_theme_preset = Some(preset);
        state.applied_motion_preset_theme_preset = None;
    }

    pub(crate) fn sync_motion_preset(app: &mut App, state: &mut UiGalleryWindowState) {
        let theme_preset = app.models().get_cloned(&state.theme_preset).flatten();
        let preset = app
            .models()
            .get_cloned(&state.motion_preset)
            .flatten()
            .unwrap_or_else(|| Arc::from("theme"));

        let already_applied = Some(preset.as_ref()) == state.applied_motion_preset.as_deref()
            && theme_preset.as_deref() == state.applied_motion_preset_theme_preset.as_deref();
        if already_applied {
            return;
        }

        let patch = Self::motion_preset_theme_patch(preset.as_ref());
        fret_ui::Theme::with_global_mut(app, |theme| {
            theme.apply_config_patch(&patch);
        });

        let _ = app
            .models_mut()
            .update(&state.motion_preset_open, |open| *open = false);

        state.applied_motion_preset = Some(preset);
        state.applied_motion_preset_theme_preset = theme_preset;
    }
}
