//! Long-lived message sources composed of one or more [`Recipe`]s.

pub mod recipe;
pub mod slot;
pub mod subscription_gen;

pub use recipe::Recipe;
pub use slot::Slot;
pub use subscription_gen::SubscriptionGen;

use recipe::recipe_id;

use alloc::{boxed::Box, vec::Vec};
use core::{
    future::{pending, poll_fn},
    pin::Pin,
    task::Poll,
};

/// A long-running message source built from one or more [`Recipe`]s.
///
/// Use [`Subscription::none`] for "no subscription". For active sources:
/// - [`Subscription::from_recipe`] wraps a single Recipe.
/// - [`Subscription::batch`] combines multiple Subscriptions into one
///   whose recipes fire concurrently.
///
/// The runtime refreshes subscriptions after every processed message and
/// diffs slot-by-slot by Recipe identity: recipes whose identity matches
/// across refreshes keep their pending futures intact; new recipes spawn;
/// missing recipes are cancelled.
pub struct Subscription<M: Clone + 'static> {
    pub(crate) slots: Vec<Slot<M>>,
}

impl<M: Clone + 'static> Subscription<M> {
    /// No subscription. Empty `slots`.
    #[must_use]
    pub fn none() -> Self {
        Self { slots: Vec::new() }
    }

    /// Build a subscription from a single [`Recipe`]. Identity =
    /// `TypeId::of::<R>()` ⊕ `Hash` of `recipe`.
    #[must_use]
    pub fn from_recipe<R>(mut recipe: R) -> Self
    where
        R: Recipe<Message = M>,
    {
        let id = recipe_id(&recipe);
        let spawn: SubscriptionGen<M> = Box::new(move || recipe.next());
        let mut slots = Vec::with_capacity(1);
        slots.push(Slot {
            id,
            spawn: Some(spawn),
            pending: None,
        });
        Self { slots }
    }

    /// Combine multiple subscriptions; each underlying recipe keeps its
    /// individual identity. Nested batches flatten.
    #[must_use]
    pub fn batch(subs: impl IntoIterator<Item = Subscription<M>>) -> Self {
        let mut slots = Vec::new();
        for sub in subs {
            slots.extend(sub.slots);
        }
        Self { slots }
    }

    /// Wait for the next message from any active recipe. Lazy-spawns
    /// each slot's pending future on first poll. If empty (or all
    /// recipes exhausted), pends forever — safe to use as a `select`
    /// arm.
    pub(crate) async fn next(&mut self) -> M {
        if self.slots.is_empty() {
            return pending().await;
        }

        poll_fn(|cx| {
            for slot in &mut self.slots {
                if slot.pending.is_none() {
                    let Some(spawn_fn) = slot.spawn.as_mut() else {
                        continue;
                    };
                    slot.pending = spawn_fn();
                    if slot.pending.is_none() {
                        slot.spawn = None; // recipe exhausted; mark dead
                        continue;
                    }
                }
                let fut = slot.pending.as_mut().expect("just spawned above");
                if let Poll::Ready(msg) = Pin::as_mut(fut).poll(cx) {
                    slot.pending = None;
                    return Poll::Ready(msg);
                }
            }
            Poll::Pending
        })
        .await
    }

    /// Diff `new` against `self` slot-by-slot. Slots with matching ids
    /// keep their pending futures intact; slots only in `new` start
    /// fresh; slots only in `self` are dropped (cancelled).
    pub(crate) fn refresh(&mut self, new: Subscription<M>) {
        let mut new_slots = new.slots;
        for new_slot in new_slots.iter_mut() {
            if let Some(pos) = self
                .slots
                .iter()
                .position(|slot| slot.id == new_slot.id)
            {
                let old = self.slots.swap_remove(pos);
                new_slot.pending = old.pending;
            }
        }
        // Anything left in self.slots had no match in new; dropped.
        self.slots = new_slots;
    }
}

impl<M: Clone + 'static> Default for Subscription<M> {
    fn default() -> Self {
        Self::none()
    }
}
