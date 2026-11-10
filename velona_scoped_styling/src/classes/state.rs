use velona_core::masonry_core::core::Selector;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopedClassesState {
    pub is_hovered: Option<bool>,
    pub is_active: Option<bool>,
    pub is_disabled: Option<bool>,
    pub has_focus_target: Option<bool>,
}

impl From<ScopedClassesState> for Selector {
    fn from(value: ScopedClassesState) -> Self {
        let selector = Self::default();
        value.apply_to_selector(selector)
    }
}

impl ScopedClassesState {
    pub const HOVERED: Self = Self {
        is_hovered: Some(true),
        is_active: None,
        is_disabled: None,
        has_focus_target: None,
    };

    pub const ACTIVE: Self = Self {
        is_hovered: None,
        is_active: Some(true),
        is_disabled: None,
        has_focus_target: None,
    };

    pub const DISABLED: Self = Self {
        is_hovered: None,
        is_active: None,
        is_disabled: Some(true),
        has_focus_target: None,
    };

    pub const FOCUS: Self = Self {
        is_hovered: None,
        is_active: None,
        is_disabled: None,
        has_focus_target: Some(true),
    };

    pub fn hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = Some(is_hovered);
        self
    }
    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
    pub fn disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = Some(is_disabled);
        self
    }
    pub fn focus(mut self, has_focus_target: bool) -> Self {
        self.has_focus_target = Some(has_focus_target);
        self
    }
    pub fn apply_to_selector(self, mut selector: Selector) -> Selector {
        if let Some(hovered) = self.is_hovered {
            selector = selector.with_hovered(hovered);
        }
        if let Some(active) = self.is_active {
            selector = selector.with_active(active);
        }
        if let Some(disabled) = self.is_disabled {
            selector = selector.with_disabled(disabled);
        }
        if let Some(focus) = self.has_focus_target {
            selector = selector.with_focused(focus);
        }
        selector
    }
    pub fn into_selector_with_classes(self, classes: &[&str]) -> Selector {
        self.apply_to_selector(Selector::classes(classes))
    }
}
