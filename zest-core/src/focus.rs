//! Focus identity and traversal state for transient widget trees.

/// Stable identity of a focusable widget across frames.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Construct a widget id from a stable numeric value.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying numeric value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Direction of focus traversal.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    /// Move to the next focusable widget.
    Forward,
    /// Move to the previous focusable widget.
    Backward,
}

/// Cross-frame focus state owned by the runtime.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FocusState {
    focused: Option<WidgetId>,
}

impl FocusState {
    /// A fresh focus state with no focused widget.
    #[must_use]
    pub const fn new() -> Self {
        Self { focused: None }
    }

    /// The currently-focused widget, if any.
    #[must_use]
    pub const fn focused(self) -> Option<WidgetId> {
        self.focused
    }

    /// Set the focused widget directly.
    pub fn set(&mut self, focused: Option<WidgetId>) {
        self.focused = focused;
    }

    /// Clear focus.
    pub fn clear(&mut self) {
        self.focused = None;
    }

    /// Drop focus if the currently-focused widget is no longer present.
    pub fn reconcile(&mut self, order: &[WidgetId]) {
        if let Some(id) = self.focused
            && !order.contains(&id)
        {
            self.focused = None;
        }
    }

    /// Advance focus through `order` in `direction`.
    pub fn advance(&mut self, order: &[WidgetId], direction: FocusDirection) {
        if order.is_empty() {
            self.focused = None;
            return;
        }

        let current = self
            .focused
            .and_then(|id| order.iter().position(|candidate| *candidate == id));

        let next = match (direction, current) {
            (FocusDirection::Forward, Some(idx)) => (idx + 1) % order.len(),
            (FocusDirection::Backward, Some(0)) => order.len() - 1,
            (FocusDirection::Backward, Some(idx)) => idx - 1,
            (FocusDirection::Forward | FocusDirection::Backward, None) => 0,
        };

        self.focused = Some(order[next]);
    }
}
