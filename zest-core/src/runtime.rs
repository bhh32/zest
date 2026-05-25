//! The async event loop that drives an [`Application`](crate::Application).

use crate::application::{Application, Subscription, Task};
use crate::event::{InputEvent, TouchPhase};
use crate::platform::Platform;
use crate::screen::ScreenView;
use crate::widget::Widget;
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
        let mut pressed_at: Option<Point> = None;

        loop {
            let new_viewport = platform.viewport();
            if new_viewport != viewport_size {
                viewport_size = new_viewport;
            }
            let viewport_rect = Rectangle::new(Point::zero(), viewport_size);

            let msg_to_update = {
                let screen = app.view();
                let theme = screen.theme();
                let mut elem = screen.view();
                elem.arrange(viewport_rect);
                if let Some(p) = pressed_at {
                    elem.mark_pressed(p);
                }

                let bg = theme.background.base;
                let _ = platform
                    .render_with(|renderer| {
                        let _ = renderer.fill_rect(viewport_rect, bg);
                        let _ = elem.draw(renderer, theme);
                        Ok(())
                    })
                    .await;

                let outcome =
                    select3(platform.next_event(), pending.next(), subscription.next()).await;

                match outcome {
                    Either3::First(None) => return,
                    Either3::First(Some(InputEvent::Touch(t))) => {
                        let m = elem.handle_touch(t.point, t.phase);
                        pressed_at = match t.phase {
                            TouchPhase::Down | TouchPhase::Moved => Some(t.point),
                            TouchPhase::Up => None,
                        };
                        m
                    }
                    Either3::Second(m) => m,
                    Either3::Third(m) => Some(m),
                }
            };

            if let Some(m) = msg_to_update {
                let task = app.update(m);
                pending.extend(task);
                subscription.refresh(app.subscription());
            }
        }
    }
}

impl<A: Application> Default for Runtime<A> {
    fn default() -> Self {
        Self::new()
    }
}
