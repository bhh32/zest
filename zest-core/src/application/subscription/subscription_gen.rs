use alloc::boxed::Box;

use crate::application::BoxFuture;

/// Closure that produces the next future in a Subscription.
pub type SubscriptionGen<M> = Box<dyn FnMut() -> Option<BoxFuture<M>> + 'static>;
