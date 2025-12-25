use crate::{ChromeRefinement, Space};

pub trait RefineStyle: Sized {
    fn refine_style(self, style: ChromeRefinement) -> Self;
}

pub trait Stylable: Sized {
    fn apply_style(self, style: ChromeRefinement) -> Self;
}

impl<T: RefineStyle> Stylable for T {
    fn apply_style(self, style: ChromeRefinement) -> Self {
        RefineStyle::refine_style(self, style)
    }
}

pub struct Styled<T> {
    inner: T,
    style: ChromeRefinement,
}

impl<T> Styled<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            style: ChromeRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.style = self.style.merge(style);
        self
    }

    pub fn px_1(mut self) -> Self {
        self.style = self.style.px_1();
        self
    }

    pub fn px_0p5(mut self) -> Self {
        self.style = self.style.px_0p5();
        self
    }

    pub fn px_0(mut self) -> Self {
        self.style = self.style.px_0();
        self
    }

    pub fn px_1p5(mut self) -> Self {
        self.style = self.style.px_1p5();
        self
    }

    pub fn px_2(mut self) -> Self {
        self.style = self.style.px_2();
        self
    }

    pub fn px_2p5(mut self) -> Self {
        self.style = self.style.px_2p5();
        self
    }

    pub fn px_3(mut self) -> Self {
        self.style = self.style.px_3();
        self
    }

    pub fn px_4(mut self) -> Self {
        self.style = self.style.px_4();
        self
    }

    pub fn py_1(mut self) -> Self {
        self.style = self.style.py_1();
        self
    }

    pub fn py_0p5(mut self) -> Self {
        self.style = self.style.py_0p5();
        self
    }

    pub fn py_0(mut self) -> Self {
        self.style = self.style.py_0();
        self
    }

    pub fn py_1p5(mut self) -> Self {
        self.style = self.style.py_1p5();
        self
    }

    pub fn py_2(mut self) -> Self {
        self.style = self.style.py_2();
        self
    }

    pub fn py_2p5(mut self) -> Self {
        self.style = self.style.py_2p5();
        self
    }

    pub fn py_3(mut self) -> Self {
        self.style = self.style.py_3();
        self
    }

    pub fn py_4(mut self) -> Self {
        self.style = self.style.py_4();
        self
    }

    pub fn pt(mut self, space: Space) -> Self {
        self.style = self.style.pt(space);
        self
    }

    pub fn pr(mut self, space: Space) -> Self {
        self.style = self.style.pr(space);
        self
    }

    pub fn pb(mut self, space: Space) -> Self {
        self.style = self.style.pb(space);
        self
    }

    pub fn pl(mut self, space: Space) -> Self {
        self.style = self.style.pl(space);
        self
    }

    pub fn p_1(mut self) -> Self {
        self.style = self.style.p_1();
        self
    }

    pub fn p_0(mut self) -> Self {
        self.style = self.style.p_0();
        self
    }

    pub fn p_0p5(mut self) -> Self {
        self.style = self.style.p_0p5();
        self
    }

    pub fn p_1p5(mut self) -> Self {
        self.style = self.style.p_1p5();
        self
    }

    pub fn p_2(mut self) -> Self {
        self.style = self.style.p_2();
        self
    }

    pub fn p_2p5(mut self) -> Self {
        self.style = self.style.p_2p5();
        self
    }

    pub fn p_3(mut self) -> Self {
        self.style = self.style.p_3();
        self
    }

    pub fn p_4(mut self) -> Self {
        self.style = self.style.p_4();
        self
    }

    pub fn rounded_md(mut self) -> Self {
        self.style = self.style.rounded_md();
        self
    }

    pub fn border_1(mut self) -> Self {
        self.style = self.style.border_1();
        self
    }
}

impl<T: Stylable> Styled<T> {
    pub fn finish(self) -> T {
        self.inner.apply_style(self.style)
    }
}

pub trait StyledExt: RefineStyle + Sized {
    fn styled(self) -> Styled<Self> {
        Styled::new(self)
    }
}

impl<T: RefineStyle> StyledExt for T {}

macro_rules! impl_refine_style {
    ($ty:path) => {
        impl RefineStyle for $ty {
            fn refine_style(self, style: ChromeRefinement) -> Self {
                <$ty>::refine_style(self, style)
            }
        }
    };
}

impl_refine_style!(crate::button::Button);
impl_refine_style!(crate::combobox::Combobox);
impl_refine_style!(crate::dropdown_menu::DropdownMenuButton);
impl_refine_style!(crate::frame::Frame);
impl_refine_style!(crate::scroll_area::ScrollArea);
impl_refine_style!(crate::select::Select);
impl_refine_style!(crate::text_area_field::TextAreaField);
impl_refine_style!(crate::text_field::TextField);
impl_refine_style!(crate::toolbar::Toolbar);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MetricRef, Space};

    #[derive(Debug, Default, Clone)]
    struct Dummy {
        style: ChromeRefinement,
    }

    impl RefineStyle for Dummy {
        fn refine_style(mut self, style: ChromeRefinement) -> Self {
            self.style = style;
            self
        }
    }

    #[test]
    fn styled_ext_builds_a_refinement_chain() {
        let dummy = Dummy::default()
            .styled()
            .px_3()
            .py_2()
            .border_1()
            .rounded_md()
            .finish();

        let padding = dummy.style.padding.expect("expected padding refinement");
        match padding.left {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N3.token_key()),
            _ => panic!("expected left padding token"),
        }
        match padding.right {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N3.token_key()),
            _ => panic!("expected right padding token"),
        }
        match padding.top {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N2.token_key()),
            _ => panic!("expected top padding token"),
        }
        match padding.bottom {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N2.token_key()),
            _ => panic!("expected bottom padding token"),
        }
        assert!(dummy.style.border_width.is_some());
        assert!(dummy.style.radius.is_some());
    }
}
