use std::hash::Hash;

/// User-defined event type `MUST` implement this trait.
pub trait UserEvent: Default {
    /// Event id
    /// [`crate::channel::CompleteQ`] use this id to track event message.
    type ID: ?Sized + Hash + Eq + Send + Sync + Clone + Default + 'static;
    /// Event message payload structure.
    type Argument: Sized + Send + Default + 'static;
}

/// Generally if user-defined event system do not care about event id's semantics,
/// `CAN` implement this trait to reduce usage complexity
pub trait AutoIncEvent: UserEvent {
    /// Generate a new event id
    fn next(&mut self) -> Self::ID;
}
