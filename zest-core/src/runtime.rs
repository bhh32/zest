//! The async event loop that drives an [`Application`](crate::Application).

use crate::application::{Application, Subscription, Task};
use crate::dirty::DirtyRegion;
use crate::event::{ButtonState, InputEvent, Key, TouchPhase, UiAction};
use crate::focus::{FocusDirection, FocusState, WidgetId};
use crate::platform::Platform;
use crate::screen::ScreenView;
use crate::widget::Widget;
use alloc::vec::Vec;
use core::marker::PhantomData;
use embassy_futures::select::{Either3, select3};
use embedded_graphics::{prelude::*, primitives::Rectangle};

/// Drives an [`Application`](crate::Application)'s event loop on a
/// [`Platform`](crate::Platform).
pub struct Runtime<A: Application> {
    _ph: PhantomData<A>,
}

impl<A: Application> Runtime<A> {
    /// Create a runtime for application type `A`.
    #[must_use]
    pub fn new() -> Self {
        Self { _ph: PhantomData }
    }

    /// Run the application event loop on `platform`: poll input, drive
    /// `update` and subscriptions, and redraw on demand. Does not return.
    pub async fn run<P>(self, mut platform: P)
    where
        P: Platform<Color = A::Color>,
    {
        let (mut app, initial_task) = A::init();

        let mut viewport_size = platform.viewport();
        let mut pending: Task<A::Message> = initial_task;
        let mut subscription: Subscription<A::Message> = app.subscription();
        let mut focus = FocusState::new();
        let mut pressed_at: Option<Point> = None;
        let mut dirty = DirtyRegion::full();
        let capabilities = platform.capabilities();

        loop {
            let new_viewport = platform.viewport();
            if new_viewport != viewport_size {
                viewport_size = new_viewport;
                dirty = DirtyRegion::full();
            }
            let viewport_rect = Rectangle::new(Point::zero(), viewport_size);

            let msg_to_update = {
                let screen = app.view();
                let theme = screen.theme();
                let mut elem = screen.view();
                elem.arrange(viewport_rect);
                let mut focus_order = Vec::new();
                elem.collect_focusable(&mut focus_order);
                focus.reconcile(&focus_order);
                elem.sync_focus(focus.focused());
                if let Some(p) = pressed_at {
                    elem.mark_pressed(p);
                }

                let bg = theme.background.base;
                if !dirty.is_none() {
                    let _ = platform
                        .render_with_dirty(&dirty, |renderer| {
                            if capabilities.supports_clip
                                && let Some(rects) = dirty.rects()
                            {
                                for rect in rects {
                                    renderer.push_clip(*rect);
                                    let _ = renderer.fill_rect(viewport_rect, bg);
                                    let _ = elem.draw(renderer, theme);
                                    renderer.pop_clip();
                                }
                            } else {
                                let _ = renderer.fill_rect(viewport_rect, bg);
                                let _ = elem.draw(renderer, theme);
                            }
                            Ok(())
                        })
                        .await;
                }
                dirty = DirtyRegion::none();

                let outcome =
                    select3(platform.next_event(), pending.next(), subscription.next()).await;
                let previous_focus = focus.focused();

                match outcome {
                    Either3::First(None) => return,
                    Either3::First(Some(event)) => match event {
                        InputEvent::Touch(t) => {
                            if t.phase == TouchPhase::Down
                                && let Some(target) = elem.focus_at(t.point)
                            {
                                focus.set(Some(target));
                            }
                            let m = elem.handle_touch(t.point, t.phase);
                            pressed_at = match t.phase {
                                TouchPhase::Down | TouchPhase::Moved => Some(t.point),
                                TouchPhase::Up => None,
                            };
                            dirty = DirtyRegion::full();
                            m
                        }
                        InputEvent::Key(key) => match (key.key, key.state) {
                            (Key::Tab, ButtonState::Pressed | ButtonState::Repeated) => {
                                focus.advance(&focus_order, FocusDirection::Forward);
                                dirty = Self::focus_dirty(&elem, previous_focus, focus.focused());
                                None
                            }
                            (Key::Escape, ButtonState::Pressed) => {
                                let msg =
                                    Self::dispatch_action(&mut elem, &mut focus, UiAction::Cancel);
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            (Key::Enter, ButtonState::Pressed) => {
                                let msg = Self::dispatch_action(
                                    &mut elem,
                                    &mut focus,
                                    UiAction::Activate,
                                );
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            (Key::Left, ButtonState::Pressed | ButtonState::Repeated) => {
                                let msg = Self::dispatch_action(
                                    &mut elem,
                                    &mut focus,
                                    UiAction::NavigateLeft,
                                );
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            (Key::Right, ButtonState::Pressed | ButtonState::Repeated) => {
                                let msg = Self::dispatch_action(
                                    &mut elem,
                                    &mut focus,
                                    UiAction::NavigateRight,
                                );
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            (Key::Up, ButtonState::Pressed | ButtonState::Repeated) => {
                                let msg = Self::dispatch_action(
                                    &mut elem,
                                    &mut focus,
                                    UiAction::NavigateUp,
                                );
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            (Key::Down, ButtonState::Pressed | ButtonState::Repeated) => {
                                let msg = Self::dispatch_action(
                                    &mut elem,
                                    &mut focus,
                                    UiAction::NavigateDown,
                                );
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            _ => None,
                        },
                        InputEvent::Encoder(encoder) => {
                            if encoder.delta > 0 {
                                focus.advance(&focus_order, FocusDirection::Forward);
                            } else if encoder.delta < 0 {
                                focus.advance(&focus_order, FocusDirection::Backward);
                            }
                            dirty = Self::focus_dirty(&elem, previous_focus, focus.focused());
                            None
                        }
                        InputEvent::Action(action) => match action {
                            UiAction::FocusNext => {
                                focus.advance(&focus_order, FocusDirection::Forward);
                                dirty = Self::focus_dirty(&elem, previous_focus, focus.focused());
                                None
                            }
                            UiAction::FocusPrevious => {
                                focus.advance(&focus_order, FocusDirection::Backward);
                                dirty = Self::focus_dirty(&elem, previous_focus, focus.focused());
                                None
                            }
                            UiAction::Cancel => {
                                let msg =
                                    Self::dispatch_action(&mut elem, &mut focus, UiAction::Cancel);
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                            _ => {
                                let msg = Self::dispatch_action(&mut elem, &mut focus, action);
                                dirty = Self::dirty_after_action(
                                    &elem,
                                    previous_focus,
                                    focus.focused(),
                                    msg.is_some(),
                                );
                                msg
                            }
                        },
                    },
                    Either3::Second(m) => {
                        if m.is_some() {
                            dirty = DirtyRegion::full();
                        }
                        m
                    }
                    Either3::Third(m) => {
                        dirty = DirtyRegion::full();
                        Some(m)
                    }
                }
            };

            if let Some(m) = msg_to_update {
                let task = app.update(m);
                pending.extend(task);
                subscription.refresh(app.subscription());
                dirty = DirtyRegion::full();
            }
        }
    }

    fn dispatch_action(
        elem: &mut impl Widget<A::Color, A::Message>,
        focus: &mut FocusState,
        action: UiAction,
    ) -> Option<A::Message> {
        let target = focus.focused()?;
        if let Some(next) = elem.navigate_focus(target, action) {
            focus.set(Some(next));
            None
        } else {
            elem.route_action(target, action)
        }
    }

    fn focus_dirty(
        elem: &impl Widget<A::Color, A::Message>,
        previous: Option<WidgetId>,
        current: Option<WidgetId>,
    ) -> DirtyRegion {
        if previous == current {
            return DirtyRegion::none();
        }

        let mut dirty = DirtyRegion::none();
        if let Some(target) = previous
            && let Some(rect) = elem.focus_rect(target)
        {
            dirty.add_rect(rect);
        }
        if let Some(target) = current
            && let Some(rect) = elem.focus_rect(target)
        {
            dirty.add_rect(rect);
        }

        if dirty.is_none() {
            DirtyRegion::full()
        } else {
            dirty
        }
    }

    fn dirty_after_action(
        elem: &impl Widget<A::Color, A::Message>,
        previous: Option<WidgetId>,
        current: Option<WidgetId>,
        emitted_message: bool,
    ) -> DirtyRegion {
        if emitted_message {
            DirtyRegion::full()
        } else {
            Self::focus_dirty(elem, previous, current)
        }
    }
}

impl<A: Application> Default for Runtime<A> {
    fn default() -> Self {
        Self::new()
    }
}
