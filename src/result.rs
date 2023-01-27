use crate::{error::CompleteQError, user_event::UserEvent};

/// Send complete event result status.
pub enum EmitResult {
    /// Fire completed event success
    Completed,

    /// No surviving receivers, channel closed.
    Closed,
}

impl EmitResult {
    /// Convert `EmitResult` structure into [`Result<(),CompleteQError>`]
    ///
    /// If channel closed this method will return [`Err(CompleteQError::PipeBroken)`]
    pub fn completed(self) -> Result<(), CompleteQError> {
        match self {
            Self::Completed => Ok(()),
            Self::Closed => Err(CompleteQError::PipeBroken),
        }
    }

    /// Helper method to detect if `EmitResult` enum is [`EmitResult::Closed`]
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Completed => false,
            Self::Closed => true,
        }
    }
}

/// Receiver poll return value.
pub enum ReceiveResult<E: UserEvent> {
    /// Success received one event message
    Success(Option<E::Argument>),
    /// Receive event message timeout
    Timeout,
}

impl<E: UserEvent> ReceiveResult<E> {
    /// Convert `ReceiveResult` structure into [`Result<(),CompleteQError>`]
    ///
    /// If timeout this method will return [`Err(CompleteQError::Timeout)`]
    pub fn success(self) -> Result<Option<E::Argument>, CompleteQError> {
        match self {
            Self::Success(argument) => Ok(argument),
            Self::Timeout => Err(CompleteQError::Timeout),
        }
    }

    /// Helper method to detect if `ReceiveResult` enum is [`ReceiveResult::Timeout`]
    pub fn is_timeout(&self) -> bool {
        match self {
            Self::Timeout => true,
            _ => false,
        }
    }
}

pub(crate) enum EmitInnerResult<E: UserEvent> {
    /// Fire completed event success
    Completed,
    /// Sending quene pending, return unsuccessful delivery message
    Pending(E::Argument),

    /// No surviving receivers, channel closed.
    Closed,
}
