use crate::user_event::UserEvent;

/// Send complete event result status.
pub enum EmitResult {
    /// Fire completed event success
    Completed,

    /// No surviving receivers, channel closed.
    Closed,
}

/// Receiver poll return value.
pub enum ReceiveResult<E: UserEvent> {
    /// Success receive event completed
    Success(E::Argument),
    /// Receiver waiting canceld by [`super::channel::CompleteQ`][``]
    Canceled,
}

pub(crate) enum EmitInnerResult<E: UserEvent> {
    /// Fire completed event success
    Completed,
    /// Sending quene pending, return unsuccessful delivery message
    Pending(E::Argument),

    /// No surviving receivers, channel closed.
    Closed,
}
