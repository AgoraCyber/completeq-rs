use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::Poll,
    time::Duration,
};

use crate::{result::ReceiveResult, user_event::UserEvent};

use super::{inner::CompleteQImpl, sender::EventSender};

/// Event receiver endpoint.
///
/// We can create an associated [`EventSender`] instance from this instance.
pub struct EventReceiver<E: UserEvent> {
    max_len: usize,
    event_id: E::ID,
    pub(crate) receiver_id: usize,
    receiver_id_seq: Arc<AtomicUsize>,
    inner: Arc<Mutex<CompleteQImpl<E>>>,
    timeout: Option<Duration>,
}

impl<E: UserEvent> EventReceiver<E> {
    pub(crate) fn new(
        event_id: E::ID,
        max_len: usize,
        receiver_id_seq: Arc<AtomicUsize>,
        inner: Arc<Mutex<CompleteQImpl<E>>>,
        timeout: Option<Duration>,
    ) -> Self {
        let id = receiver_id_seq.fetch_add(1, Ordering::SeqCst);

        // Open a new channel or reset an existing channel configuration data.
        inner
            .lock()
            .unwrap()
            .open_channel(event_id.clone(), max_len);

        Self {
            max_len,
            event_id,
            receiver_id: id,
            receiver_id_seq,
            inner,
            timeout,
        }
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

impl<E: UserEvent> Clone for EventReceiver<E> {
    fn clone(&self) -> Self {
        // Construct new receiver instance
        EventReceiver::new(
            self.event_id.clone(),
            self.max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            self.timeout.clone(),
        )
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
        self.inner.lock().unwrap().poll_once(
            self.receiver_id,
            self.event_id.clone(),
            cx.waker().clone(),
        )
    }
}
