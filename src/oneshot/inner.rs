use std::{
    collections::HashMap,
    task::{Poll, Waker},
};

use crate::{
    error::CompleteQError,
    result::{EmitInnerResult, ReceiveResult},
    user_event::UserEvent,
};

struct Channel<Argument> {
    /// Pending receiver [`waker`](Waker), maybe [`None`]
    receiver: Option<Waker>,
    /// Pending message, maybe [`None`]
    pending_msg: Option<Argument>,
}

impl<Argument> Default for Channel<Argument> {
    fn default() -> Self {
        Self {
            receiver: None,
            pending_msg: None,
        }
    }
}

/// CompleteQ inner implementation.
pub(crate) struct CompleteQImpl<E: UserEvent> {
    channels: HashMap<E::ID, Channel<E::Argument>>,
}

impl<E: UserEvent> CompleteQImpl<E> {
    pub(crate) fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }
    /// Send one completed event
    ///
    /// If there are no surviving receiver, will return [`Closed`](super::emit::EmitInnerResult::Closed)
    pub fn complete_one(&mut self, event_id: E::ID, event_arg: E::Argument) -> EmitInnerResult<E> {
        if let Some(channel) = self.channels.get_mut(&event_id) {
            channel.pending_msg = Some(event_arg);

            if let Some(receiver) = channel.receiver.take() {
                receiver.wake_by_ref();
            }

            return EmitInnerResult::Completed;
        }

        log::trace!("complete_one event_id({}) -- closed", event_id);

        EmitInnerResult::Closed
    }

    /// Try poll one completed event.
    ///
    /// If the channel's waiting queue length is zero,
    /// this method will return [`Pending`](Poll::Pending) and cache [`waker`](Waker) parameter.
    pub fn poll_once(&mut self, event_id: E::ID, waker: Waker) -> Poll<ReceiveResult<E>> {
        let channel = self.channels.get_mut(&event_id);

        if let Some(channel) = channel {
            if let Some(argument) = channel.pending_msg.take() {
                return Poll::Ready(ReceiveResult::Success(Some(argument)));
            } else {
                channel.receiver = Some(waker);
                return Poll::Pending;
            }
        } else {
            return Poll::Ready(ReceiveResult::Success(None));
        }
    }

    pub fn open_channel(&mut self, event_id: E::ID) -> Result<(), CompleteQError> {
        if self.channels.contains_key(&event_id) {
            Err(CompleteQError::OpenChannelTwice)
        } else {
            self.channels.insert(event_id, Default::default());

            Ok(())
        }
    }

    pub fn close_channel(&mut self, event_id: E::ID) {
        self.channels.remove(&event_id);
    }

    pub fn remove_pending_poll(&mut self, event_id: E::ID) {
        let channel = self
            .channels
            .get_mut(&event_id)
            .expect("Call open_channel first");

        if let Some(waker) = channel.receiver.take() {
            waker.wake_by_ref();
        }
    }

    pub fn cancel_all(&mut self) {
        for channel in self.channels.values_mut() {
            if let Some(waker) = &channel.receiver {
                waker.wake_by_ref();
            }
        }

        self.channels.clear();
    }
}
