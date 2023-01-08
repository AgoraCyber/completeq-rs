use std::{
    sync::{Arc, Mutex},
    task::Poll,
};

use crate::{result::ReceiveResult, user_event::UserEvent};

use super::{inner::CompleteQImpl, sender::EventSender};

/// Event receiver endpoint.
///
/// We can create an associated [`EventSender`] instance from this instance.
pub struct EventReceiver<E: UserEvent> {
    event_id: E::ID,
    inner: Arc<Mutex<CompleteQImpl<E>>>,
}

impl<E: UserEvent> EventReceiver<E> {
    pub(crate) fn new(event_id: E::ID, inner: Arc<Mutex<CompleteQImpl<E>>>) -> Self {
        // Open a new channel or reset an existing channel configuration data.
        _ = inner.lock().unwrap().open_channel(event_id.clone());

        Self { event_id, inner }
    }

    /// Get receiver bound event_id
    pub fn event_id(&self) -> E::ID {
        self.event_id.clone()
    }

    /// Create an associated [`Sender`](EventSender) instance
    pub fn sender(&self) -> EventSender<E> {
        EventSender::new(self.event_id.clone(), self.inner.clone())
    }
}

impl<E: UserEvent> Drop for EventReceiver<E> {
    fn drop(&mut self) {
        self.inner
            .lock()
            .unwrap()
            .close_channel(self.event_id.clone());
    }
}

impl<E: UserEvent> std::future::Future for EventReceiver<E> {
    type Output = ReceiveResult<E>;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner
            .lock()
            .unwrap()
            .poll_once(self.event_id.clone(), cx.waker().clone())
    }
}
