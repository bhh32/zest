use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

/// Boxed future producing a value of type `T`. Shared between [`Task`]
/// and [`Subscription`].
///
/// [`Task`]: crate::application::Task
/// [`Subscription`]: crate::application::Subscription
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;
