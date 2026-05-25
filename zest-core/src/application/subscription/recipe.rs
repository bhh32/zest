//! The [`Recipe`] trait: a stable identity for a long-lived message source.

use crate::application::BoxFuture;
use core::{
    any::TypeId,
    hash::{Hash, Hasher},
};

/// A subscription recipe: a user-defined type whose `TypeId` and
/// `Hash` together form a stable, refactor-proof identity for a
/// long-lived message source.
///
/// Most users never write a `Recipe` directly. Instead they use
/// helpers (`zest::time::every`, etc.) that wrap built-in recipes. Power
/// users implementing custom event sources (websockets, GPIO,
/// interrupt handlers) implement this trait.
///
/// ## Identity
///
/// Identity = `TypeId::of::<Self>()` mixed with the `Hash` of `self`.
/// Two recipes of the same type with the same hashable fields are the
/// "same" subscription; the runtime keeps the existing future running
/// across refreshes. Changing the type or any hashed field replaces
/// the subscription.
///
/// ## Driving
///
/// [`next`](Self::next) returns the next future in the message stream.
/// The runtime calls it once on subscription, then again every time
/// the previously-returned future resolves. Returning `None` ends the
/// subscription.
pub trait Recipe: Hash + 'static {
    /// Message type produced by this recipe.
    type Message: 'static;

    /// Produce the next future in the message stream, or `None` to end.
    fn next(&mut self) -> Option<BoxFuture<Self::Message>>;
}

/// Compute the identity hash for a recipe: `TypeId::of::<R>()` mixed
/// with the recipe's own `Hash` impl. Returns a non-zero value (0 is
/// reserved for `Subscription::none`).
pub(crate) fn recipe_id<R: Recipe>(recipe: &R) -> u64 {
    let mut hasher = FnvHasher::default();
    TypeId::of::<R>().hash(&mut hasher);
    recipe.hash(&mut hasher);
    let id = hasher.finish();
    if id == 0 { 1 } else { id }
}

/// FNV-1a 64-bit hasher. Used for recipe identity.
struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}
