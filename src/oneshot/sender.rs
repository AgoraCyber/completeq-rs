use std::sync::{Arc, Mutex};

use crate::{
    result::{EmitInnerResult, EmitResult},
    user_event::UserEvent,
};

use super::inner::CompleteQImpl;

/// Event sender endpoint, creating by [`super::EventReceiver`] instance.
pub struct EventSender<E: UserEvent> {
    event_id: E::ID,
    inner: Arc<Mutex<CompleteQImpl<E>>>,
}

impl<E: UserEvent> EventSender<E> {
    pub(crate) fn new(event_id: E::ID, inner: Arc<Mutex<CompleteQImpl<E>>>) -> Self {
        Self { event_id, inner }
    }

    pub fn send(&self, event_arg: E::Argument) -> EmitResult {
        let result = self
            .inner
            .lock()
            .unwrap()
            .complete_one(self.event_id.clone(), event_arg);

        match result {
            EmitInnerResult::Completed => EmitResult::Completed,
            EmitInnerResult::Closed => EmitResult::Closed,
            EmitInnerResult::Pending(_) => EmitResult::Closed,
        }
    }
}
