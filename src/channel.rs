use std::{
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::Duration,
};

use crate::{
    timer::Timer,
    user_event::{AutoIncEvent, UserEvent},
};

mod inner;
use inner::*;
mod receiver;
pub use receiver::*;
mod sender;
pub use sender::*;

/// CompleteQ structure is a central scheduler for certain types of completion events
///
/// The generic parameter `E` represents a user-defined event type
#[derive(Clone)]
pub struct CompleteQ<E: UserEvent> {
    event: E,
    /// receiver id generator
    receiver_id_seq: Arc<AtomicUsize>,

    inner: Arc<Mutex<CompleteQImpl<E>>>,
}

impl<E: UserEvent> CompleteQ<E>
where
    E: 'static,
{
    pub fn new() -> Self {
        Self {
            event: E::default(),
            receiver_id_seq: Arc::new(AtomicUsize::new(1)),
            inner: Default::default(),
        }
    }

    /// Create a new event receiver with provide event_id
    pub fn wait_for(&self, event_id: E::ID, max_len: usize) -> EventReceiver<E> {
        EventReceiver::new(
            event_id,
            max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            None,
        )
    }

    /// [`wait_for`](CompleteQ::wait_for)  operation with timeout
    pub fn wait_for_timeout<T: Timer>(
        &self,
        event_id: E::ID,
        max_len: usize,
        timeout: Duration,
    ) -> EventReceiver<E> {
        let receiver = EventReceiver::new(
            event_id,
            max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            Some(timeout),
        );

        let receiver_id = receiver.receiver_id;
        let event_id = receiver.event_id();
        let inner = self.inner.clone();

        T::interval(timeout, move || {
            inner
                .lock()
                .unwrap()
                .remove_pending_poll(receiver_id, event_id);
        });

        receiver
    }
}

impl<E: AutoIncEvent> CompleteQ<E>
where
    E: 'static,
{
    /// Create a new event receiver with automatic generate event_id
    pub fn wait_one(&mut self, max_len: usize) -> EventReceiver<E> {
        EventReceiver::new(
            self.event.next(),
            max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            None,
        )
    }

    /// [`wait_one`](CompleteQ::wait_one) operation with timeout
    pub fn wait_one_timeout<T: Timer>(
        &mut self,
        max_len: usize,
        timeout: Duration,
    ) -> EventReceiver<E> {
        let receiver = EventReceiver::new(
            self.event.next(),
            max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            Some(timeout),
        );

        let receiver_id = receiver.receiver_id;
        let event_id = receiver.event_id();
        let inner = self.inner.clone();

        T::interval(timeout, move || {
            inner
                .lock()
                .unwrap()
                .remove_pending_poll(receiver_id, event_id);
        });

        receiver
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::CompleteQError, user_event::RequestId};

    use super::CompleteQ;

    #[derive(Default)]
    struct NullArgument;

    type Event = RequestId<NullArgument>;

    #[async_std::test]
    async fn one_send_one_recv() -> anyhow::Result<()> {
        _ = pretty_env_logger::try_init();

        let mut q = CompleteQ::<Event>::new();

        let receiver = q.wait_one(10);

        let sender = receiver.sender();

        async_std::task::spawn(async move {
            sender.send(Default::default()).await.completed()?;

            Ok::<(), CompleteQError>(())
        });

        receiver.await.success()?;

        Ok(())
    }
}
