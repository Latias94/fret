//! Color-edit swatch popup activation owner.

use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};

use super::super::ColorEditPopupOptions;

pub(super) struct ColorSwatchActivateInput {
    pub(super) model: Model<Color>,
    pub(super) open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) popup_has_visible_content: bool,
    pub(super) popup_options: ColorEditPopupOptions,
}

#[allow(clippy::arc_with_non_send_sync)]
pub(super) fn color_swatch_activate(input: ColorSwatchActivateInput) -> OnActivate {
    let ColorSwatchActivateInput {
        model,
        open,
        copy_menu_open,
        reference,
        popup_has_visible_content,
        popup_options,
    } = input;

    Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
        if !popup_has_visible_content {
            return;
        }
        let prev = host.models_mut().get_copied(&open).unwrap_or(false);
        let opening = !prev;
        if opening && popup_options.side_preview.shows_original() {
            let current = host
                .models_mut()
                .get_copied(&model)
                .unwrap_or(Color::TRANSPARENT);
            let _ = host
                .models_mut()
                .update(&reference, |reference| *reference = Some(current));
        }
        let _ = host.models_mut().update(&open, |v| *v = opening);
        close_copy_menu_if_open(host, &copy_menu_open);
        host.request_redraw(action_cx.window);
    })
}

fn close_copy_menu_if_open<H: UiActionHost + ?Sized>(
    host: &mut H,
    copy_menu_open: &Model<bool>,
) -> bool {
    if !host
        .models_mut()
        .get_copied(copy_menu_open)
        .unwrap_or(false)
    {
        return false;
    }

    host.models_mut()
        .update(copy_menu_open, |value| *value = false)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{AppWindowId, Color};
    use fret_ui::GlobalElementId;
    use fret_ui::action::{ActionCx, ActivateReason, UiActionHostAdapter};

    use super::*;

    #[test]
    fn swatch_activate_does_not_reclose_closed_copy_menu_model() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let model = app
            .models_mut()
            .insert(Color::from_srgb_hex_rgb(0x33_66_99));
        let open = app.models_mut().insert(false);
        let copy_menu_open = app.models_mut().insert(false);
        let reference = app.models_mut().insert(None::<Color>);
        let copy_revision = copy_menu_open.revision(&app);
        let activate = color_swatch_activate(ColorSwatchActivateInput {
            model,
            open,
            copy_menu_open: copy_menu_open.clone(),
            reference,
            popup_has_visible_content: true,
            popup_options: ColorEditPopupOptions::default(),
        });

        {
            let mut host = UiActionHostAdapter { app: &mut app };
            activate(
                &mut host,
                ActionCx {
                    window,
                    target: GlobalElementId(7),
                },
                ActivateReason::Pointer,
            );
        }

        assert_eq!(copy_menu_open.revision(&app), copy_revision);
        assert_eq!(app.models_mut().get_copied(&copy_menu_open), Some(false));
    }

    #[test]
    fn swatch_activate_closes_open_copy_menu_model() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let model = app
            .models_mut()
            .insert(Color::from_srgb_hex_rgb(0x33_66_99));
        let open = app.models_mut().insert(false);
        let copy_menu_open = app.models_mut().insert(true);
        let reference = app.models_mut().insert(None::<Color>);
        let copy_revision = copy_menu_open.revision(&app);
        let activate = color_swatch_activate(ColorSwatchActivateInput {
            model,
            open,
            copy_menu_open: copy_menu_open.clone(),
            reference,
            popup_has_visible_content: true,
            popup_options: ColorEditPopupOptions::default(),
        });

        {
            let mut host = UiActionHostAdapter { app: &mut app };
            activate(
                &mut host,
                ActionCx {
                    window,
                    target: GlobalElementId(7),
                },
                ActivateReason::Pointer,
            );
        }

        assert_ne!(copy_menu_open.revision(&app), copy_revision);
        assert_eq!(app.models_mut().get_copied(&copy_menu_open), Some(false));
    }
}
