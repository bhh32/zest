use crate::application::BoxFuture;
use super::SubscriptionGen;

pub struct Slot<M> {
    pub(crate) id: u64,
    pub(crate) spawn: Option<SubscriptionGen<M>>,
    pub(crate) pending: Option<BoxFuture<M>>,
}
