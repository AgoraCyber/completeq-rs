use std::{
    fmt::Display,
    hash::Hash,
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// User-defined event type `MUST` implement this trait.
pub trait UserEvent: Default {
    /// Event id
    /// [`crate::channel::CompleteQ`] use this id to track event message.
    type ID: ?Sized + Hash + Eq + Send + Sync + Clone + 'static + Display;
    /// Event message payload structure.
    type Argument: Sized + Send + 'static;
}

/// Generally if user-defined event system do not care about event id's semantics,
/// `CAN` implement this trait to reduce usage complexity
pub trait AutoIncEvent: UserEvent {
    /// Generate a new event id
    fn next(&mut self) -> Self::ID;
}

#[derive(Clone)]
/// user-defined event for `RPC` like system.
pub struct RPCResponser<Argument>(Arc<AtomicUsize>, PhantomData<Argument>);

impl<Argument> Default for RPCResponser<Argument> {
    fn default() -> Self {
        Self(Arc::new(AtomicUsize::new(1)), PhantomData)
    }
}

impl<Argument> UserEvent for RPCResponser<Argument>
where
    Argument: Sized + Send + 'static,
{
    type ID = usize;
    type Argument = Argument;
}

impl<Argument> AutoIncEvent for RPCResponser<Argument>
where
    Argument: Sized + Send + 'static,
{
    fn next(&mut self) -> Self::ID {
        self.0.fetch_add(1, Ordering::SeqCst)
    }
}
