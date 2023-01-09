use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::Poll,
};

use async_timer::Timer;
use futures::FutureExt;

use crate::{result::ReceiveResult, user_event::UserEvent};

use super::{inner::CompleteQImpl, sender::EventSender};

/// Event receiver endpoint.
///
/// We can create an associated [`EventSender`] instance from this instance.
pub struct EventReceiver<E: UserEvent, T: Timer> {
    max_len: usize,
    event_id: E::ID,
    pub(crate) receiver_id: usize,
    inner: Arc<Mutex<CompleteQImpl<E>>>,
    timer: Cell<Option<T>>,
}

impl<E: UserEvent, T: Timer> EventReceiver<E, T> {
    pub(crate) fn new(
        event_id: E::ID,
        max_len: usize,
        receiver_id_seq: Arc<AtomicUsize>,
        inner: Arc<Mutex<CompleteQImpl<E>>>,
        timer: Option<T>,
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
            inner,
            timer: Cell::new(timer),
        }
    }

    /// Get receiver bound event_id
    pub fn event_id(&self) -> E::ID {
        self.event_id.clone()
    }

    pub fn max_len(&self) -> usize {
        self.max_len
    }

    /// Create an associated [`Sender`](EventSender) instance
    pub fn sender(&self) -> EventSender<E> {
        EventSender::new(self.event_id.clone(), self.inner.clone())
    }
}

impl<E: UserEvent, T: Timer> Drop for EventReceiver<E, T> {
    fn drop(&mut self) {
        self.inner
            .lock()
            .unwrap()
            .close_channel(self.event_id.clone());
    }
}

impl<E: UserEvent, T: Timer + Unpin> std::future::Future for EventReceiver<E, T> {
    type Output = ReceiveResult<E>;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let timer = self.timer.take();

        if let Some(mut timer) = timer {
            match timer.poll_unpin(cx) {
                Poll::Pending => {
                    self.timer.set(Some(timer));
                }
                Poll::Ready(_) => {
                    // Remove pending poll operation .
                    self.inner
                        .lock()
                        .unwrap()
                        .remove_pending_poll(self.receiver_id, self.event_id.clone());

                    // Return timeout error
                    return Poll::Ready(ReceiveResult::Timeout);
                }
            }
        }

        self.inner.lock().unwrap().poll_once(
            self.receiver_id,
            self.event_id.clone(),
            cx.waker().clone(),
        )
    }
}
