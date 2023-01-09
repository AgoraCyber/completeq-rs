use std::{
    cell::Cell,
    future::Future,
    sync::{Arc, Mutex},
    task::Poll,
};

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

    pub fn send(&self, event_arg: E::Argument) -> EventSend<E> {
        EventSend {
            argument: Cell::new(Some(event_arg)),
            event_id: self.event_id.clone(),
            inner: self.inner.clone(),
        }
    }
}

/// create by [send](EventSender::send) method
pub struct EventSend<E: UserEvent> {
    // Using [`Cell`] to modify data in std::pin::Pin<&mut Self>
    argument: Cell<Option<E::Argument>>,
    event_id: E::ID,
    inner: Arc<Mutex<CompleteQImpl<E>>>,
}

impl<E: UserEvent> Future for EventSend<E> {
    type Output = EmitResult;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let argument = self.argument.take();
        let result = self.inner.lock().unwrap().complete_one(
            self.event_id.clone(),
            argument.unwrap(),
            cx.waker().clone(),
        );

        match result {
            EmitInnerResult::Completed => Poll::Ready(EmitResult::Completed),
            EmitInnerResult::Closed => Poll::Ready(EmitResult::Closed),
            EmitInnerResult::Pending(argument) => {
                self.argument.set(Some(argument));
                return Poll::Pending;
            }
        }
    }
}
