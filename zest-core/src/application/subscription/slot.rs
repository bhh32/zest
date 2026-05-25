//! [`Slot`]: one running subscription's identity and pending future.

use super::SubscriptionGen;
use crate::application::BoxFuture;

/// One active subscription: its stable `id`, the generator that produces
/// its futures, and the in-flight future awaiting its next message.
pub struct Slot<M> {
    pub(crate) id: u64,
    pub(crate) spawn: Option<SubscriptionGen<M>>,
    pub(crate) pending: Option<BoxFuture<M>>,
}
