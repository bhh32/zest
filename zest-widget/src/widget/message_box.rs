//! Modal dialog: a dimmed full-screen scrim with a centered card holding a
//! title, body text, and a row of action buttons.
//!
//! Immediate-mode: the host owns visibility. Render a `MessageBox` only
//! while the dialog should be shown (e.g. `if self.show_dialog { ... }`),
//! and remove it from the tree to dismiss. Each button carries the message
//! the host wants when it is tapped — typically a "dismiss" message that
//! flips the host's visibility flag.
//!
//! ## Structure
//!
//! Built entirely from a [`Stack`](crate::widget::stack::Stack):
//!
//! 1. **Scrim** (bottom layer) — a full-bleed rect filled with the darkest
//!    neutral (`palette.neutral_10`) so the content reads as dimmed behind
//!    the modal (the renderer has no alpha). The scrim also
//!    catches every touch that misses the card, so the widgets behind the
//!    modal stay inert. A tap on the scrim optionally emits
//!    [`on_dismiss`](MessageBox::on_dismiss).
//! 2. **Card** (top layer) — a centered [`Column`] of the title, body, and a
//!    [`Row`] of [`Button`]s,
//!    painted over a panel rect. Pushed last so it draws on top of and is
//!    touched before the scrim.
//!
//! The card is composed lazily on the first lifecycle call so the chainable
//! builders only record configuration — the per-button messages and the
//! optional dismiss message are stored until the stack is assembled.

use super::{Widget, button::Button, column::Column, element::Element, row::Row, text::Text};
use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Horizontal, Length, RenderError, Renderer, TouchPhase, Vertical};
use zest_theme::Theme;

/// Default card width in pixels.
const CARD_W: u32 = 260;
/// Inner padding of the card in pixels.
const CARD_PAD: u32 = 12;
/// Height of the action-button row in pixels.
const BUTTON_H: u32 = 36;

/// A modal dialog over a dimmed scrim. The host owns visibility; the card
/// holds a title, body, and a row of action buttons, each emitting a
/// host-supplied message.
pub struct MessageBox<'a, C: PixelColor, M: Clone> {
    title: String,
    body: String,
    buttons: Vec<(String, M)>,
    on_dismiss: Option<M>,
    width: Length,
    height: Length,
    /// Composed stack (scrim + card), built on first lifecycle call.
    stack: Option<Element<'a, C, M>>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> MessageBox<'a, C, M> {
    /// New empty modal. Fills its parent region (the scrim covers the
    /// screen). Configure with the builders, then it composes itself into a
    /// [`Stack`](crate::Stack).
    pub fn new() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            buttons: Vec::new(),
            on_dismiss: None,
            width: Length::Fill,
            height: Length::Fill,
            stack: None,
        }
    }

    /// The card title (drawn in the heading font).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// The card body text.
    #[must_use]
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Append an action button. Repeatable — buttons are laid out
    /// left-to-right in a row at the bottom of the card. The given message
    /// is emitted when that button is tapped.
    #[must_use]
    pub fn button(mut self, label: impl Into<String>, message: M) -> Self {
        self.buttons.push((label.into(), message));
        self
    }

    /// Message emitted when the scrim (the area outside the card)
    /// is tapped. Without it, taps outside the card are swallowed but emit
    /// nothing (the modal stays open until a button is pressed).
    #[must_use]
    pub fn on_dismiss(mut self, message: M) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Width sizing intent of the whole modal region (default
    /// [`Length::Fill`]).
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent of the whole modal region (default
    /// [`Length::Fill`]).
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Compose the scrim and card into a [`Stack`](crate::Stack), draining
    /// the recorded config so each per-button message is owned once.
    fn build(&mut self) -> Element<'a, C, M> {
        // --- card body: title + body + button row -----------------------
        let mut card = Column::new()
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Shrink);

        card = card.push(
            Text::new(self.title.clone())
                .align_x(Horizontal::Center)
                .height(Length::Shrink),
        );
        card = card.push(
            Text::new(self.body.clone())
                .align_x(Horizontal::Center)
                .height(Length::Shrink),
        );

        let mut button_row = Row::new()
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fixed(BUTTON_H));
        for (label, msg) in core::mem::take(&mut self.buttons) {
            button_row = button_row.push(Button::new(label).on_press(msg));
        }
        card = card.push(button_row);

        // The panel behind the card content: a padded rect with a filled
        // background so the text/buttons read against the scrim. Sizes to a
        // fixed width and to its content height.
        let panel = MessageCard {
            rect: Rectangle::zero(),
            inner: Element::new(card),
            card_width: Length::Fixed(CARD_W),
            card_height: Length::Shrink,
            _color: PhantomData,
        };

        // --- assemble: scrim (bottom) + centered card (top) -------------
        let scrim = Scrim {
            rect: Rectangle::zero(),
            on_dismiss: self.on_dismiss.take(),
            _color: PhantomData,
        };

        let stack = super::stack::Stack::new()
            .width(self.width)
            .height(self.height)
            .push_aligned(scrim, Horizontal::Left, Vertical::Top)
            .push_aligned(panel, Horizontal::Center, Vertical::Center);

        Element::new(stack)
    }

    fn ensure_built(&mut self) {
        if self.stack.is_none() {
            self.stack = Some(self.build());
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for MessageBox<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for MessageBox<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        self.ensure_built();
        self.stack
            .as_mut()
            .map_or(Size::zero(), |s| s.measure(constraints))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.ensure_built();
        if let Some(stack) = self.stack.as_mut() {
            stack.arrange(rect);
        }
    }

    fn rect(&self) -> Rectangle {
        self.stack.as_ref().map_or(Rectangle::zero(), |s| s.rect())
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.stack
            .as_mut()
            .and_then(|s| s.handle_touch(point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(stack) = self.stack.as_mut() {
            stack.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        if let Some(stack) = &self.stack {
            stack.draw(renderer, theme)?;
        }
        Ok(())
    }
}

// ---- internal: the dimming scrim -------------------------------------------

/// Full-bleed dimming layer. Paints a solid tint over the whole region and
/// catches every touch that reaches it (i.e. anything that missed the card),
/// optionally emitting a dismiss message.
struct Scrim<C: PixelColor, M: Clone> {
    rect: Rectangle,
    on_dismiss: Option<M>,
    _color: PhantomData<C>,
}

impl<C: PixelColor, M: Clone> Scrim<C, M> {
    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for Scrim<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        constraints.max
    }

    fn preferred_size(&self) -> (Length, Length) {
        (Length::Fill, Length::Fill)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        // Swallow all touches over the modal region; only emit on release.
        if !self.hit_test(point) {
            return None;
        }
        match phase {
            TouchPhase::Up => self.on_dismiss.clone(),
            _ => None,
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        // Opaque dimming tint — the renderer has no alpha.
        renderer.fill_rect(self.rect, theme.palette.neutral_10)?;
        Ok(())
    }
}

// ---- internal: the card panel ----------------------------------------------

/// The card panel: a filled, bordered rect with padding around its inner
/// content (the title/body/button column). Sizes to its content height.
struct MessageCard<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    inner: Element<'a, C, M>,
    card_width: Length,
    card_height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for MessageCard<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let pad2 = CARD_PAD * 2;
        let inner_c = constraints.shrink(pad2, pad2);
        let inner = self.inner.measure(inner_c);
        let w = self.card_width.resolve(CARD_W, constraints.max.width);
        let h = self
            .card_height
            .resolve(inner.height + pad2, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.card_width, self.card_height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
        let pad = CARD_PAD as i32;
        let inner_rect = Rectangle::new(
            rect.top_left + Point::new(pad, pad),
            Size::new(
                rect.size.width.saturating_sub(CARD_PAD * 2),
                rect.size.height.saturating_sub(CARD_PAD * 2),
            ),
        );
        self.inner.arrange(inner_rect);
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.inner.handle_touch(point, phase)
    }

    fn mark_pressed(&mut self, point: Point) {
        self.inner.mark_pressed(point);
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        renderer.fill_rect(self.rect, theme.primary.base)?;
        renderer.stroke_rect(self.rect, theme.background.divider)?;
        self.inner.draw(renderer, theme)?;
        Ok(())
    }
}
