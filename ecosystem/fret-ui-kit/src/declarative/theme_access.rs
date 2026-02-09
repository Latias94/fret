use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};

pub trait ElementContextThemeExt {
    fn with_theme<R>(&mut self, f: impl FnOnce(&Theme) -> R) -> R;

    fn theme_snapshot(&mut self) -> ThemeSnapshot;
}

impl<H: UiHost> ElementContextThemeExt for ElementContext<'_, H> {
    fn with_theme<R>(&mut self, f: impl FnOnce(&Theme) -> R) -> R {
        let theme = Theme::global(&*self.app);
        f(theme)
    }

    fn theme_snapshot(&mut self) -> ThemeSnapshot {
        Theme::global(&*self.app).snapshot()
    }
}
