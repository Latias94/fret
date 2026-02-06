use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    Push,
    Replace,
    Back,
    Forward,
}

impl fmt::Display for NavigationAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Push => "push",
            Self::Replace => "replace",
            Self::Back => "back",
            Self::Forward => "forward",
        };
        f.write_str(label)
    }
}

#[cfg(test)]
mod tests {
    use super::NavigationAction;

    #[test]
    fn display_is_stable() {
        assert_eq!(NavigationAction::Push.to_string(), "push");
        assert_eq!(NavigationAction::Replace.to_string(), "replace");
        assert_eq!(NavigationAction::Back.to_string(), "back");
        assert_eq!(NavigationAction::Forward.to_string(), "forward");
    }
}
