use crate::{NavigationAction, RouteLocation};

#[derive(Debug, Clone)]
pub struct MemoryHistory {
    entries: Vec<RouteLocation>,
    index: usize,
}

impl MemoryHistory {
    pub fn new(initial: RouteLocation) -> Self {
        let mut initial = initial;
        initial.canonicalize();
        Self {
            entries: vec![initial],
            index: 0,
        }
    }

    pub fn current(&self) -> &RouteLocation {
        self.entries
            .get(self.index)
            .expect("memory history index should be in bounds")
    }

    pub fn entries(&self) -> &[RouteLocation] {
        self.entries.as_slice()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn can_back(&self) -> bool {
        self.index > 0
    }

    pub fn can_forward(&self) -> bool {
        self.index + 1 < self.entries.len()
    }

    pub fn push(&mut self, mut location: RouteLocation) -> bool {
        location.canonicalize();
        if self.current() == &location {
            return false;
        }

        self.entries.truncate(self.index + 1);
        self.entries.push(location);
        self.index = self.entries.len() - 1;
        true
    }

    pub fn replace(&mut self, mut location: RouteLocation) -> bool {
        location.canonicalize();
        if self.current() == &location {
            return false;
        }

        self.entries[self.index] = location;
        true
    }

    pub fn back(&mut self) -> bool {
        if !self.can_back() {
            return false;
        }

        self.index -= 1;
        true
    }

    pub fn forward(&mut self) -> bool {
        if !self.can_forward() {
            return false;
        }

        self.index += 1;
        true
    }

    pub fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool {
        match action {
            NavigationAction::Push => target.is_some_and(|location| self.push(location)),
            NavigationAction::Replace => target.is_some_and(|location| self.replace(location)),
            NavigationAction::Back => self.back(),
            NavigationAction::Forward => self.forward(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{NavigationAction, RouteLocation};

    use super::MemoryHistory;

    #[test]
    fn push_truncates_forward_stack() {
        let mut history = MemoryHistory::new(RouteLocation::from_path("/"));

        assert!(history.push(RouteLocation::from_path("/a")));
        assert!(history.push(RouteLocation::from_path("/b")));
        assert_eq!(history.current().path, "/b");

        assert!(history.back());
        assert_eq!(history.current().path, "/a");

        assert!(history.push(RouteLocation::from_path("/c")));
        assert_eq!(history.len(), 3);
        assert_eq!(history.current().path, "/c");
        assert!(!history.can_forward());
    }

    #[test]
    fn duplicate_push_is_noop() {
        let mut history = MemoryHistory::new(RouteLocation::parse("/users/42?tab=profile"));
        assert!(!history.push(RouteLocation::parse("users///42/?tab=profile#")));
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn replace_updates_current_without_changing_len() {
        let mut history = MemoryHistory::new(RouteLocation::from_path("/"));
        assert!(history.push(RouteLocation::from_path("/docs")));
        assert!(history.replace(RouteLocation::from_path("/docs/getting-started")));
        assert_eq!(history.current().path, "/docs/getting-started");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn navigate_routes_actions() {
        let mut history = MemoryHistory::new(RouteLocation::from_path("/"));

        assert!(history.navigate(
            NavigationAction::Push,
            Some(RouteLocation::from_path("/settings"))
        ));
        assert!(history.navigate(NavigationAction::Back, None));
        assert_eq!(history.current().path, "/");
        assert!(history.navigate(NavigationAction::Forward, None));
        assert_eq!(history.current().path, "/settings");
        assert!(history.navigate(
            NavigationAction::Replace,
            Some(RouteLocation::from_path("/settings/profile"))
        ));
        assert_eq!(history.current().path, "/settings/profile");
    }
}
