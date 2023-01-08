mod inner;
use inner::*;
mod receiver;
pub use receiver::*;
mod sender;
pub use sender::*;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    timer::Timer,
    user_event::{AutoIncEvent, UserEvent},
};

/// CompleteQ structure is a central scheduler for certain types of completion events
///
/// The generic parameter `E` represents a user-defined event type
#[derive(Clone)]
pub struct CompleteQ<E: UserEvent> {
    event: E,

    inner: Arc<Mutex<CompleteQImpl<E>>>,
}

impl<E: UserEvent> CompleteQ<E>
where
    E: 'static,
{
    pub fn new() -> Self {
        Self {
            event: E::default(),
            inner: Default::default(),
        }
    }

    /// Create a new event receiver with provide event_id
    pub fn wait_for(&self, event_id: E::ID) -> EventReceiver<E> {
        EventReceiver::new(event_id, self.inner.clone())
    }

    /// [`wait_for`](CompleteQ::wait_for)  operation with timeout
    pub fn wait_for_timeout<T: Timer>(
        &self,
        event_id: E::ID,
        timeout: Duration,
    ) -> EventReceiver<E> {
        let receiver = EventReceiver::new(event_id, self.inner.clone());

        let event_id = receiver.event_id();
        let inner = self.inner.clone();

        T::interval(timeout, move || {
            inner.lock().unwrap().remove_pending_poll(event_id);
        });

        receiver
    }
}

impl<E: AutoIncEvent> CompleteQ<E>
where
    E: 'static,
{
    /// Create a new event receiver with automatic generate event_id
    pub fn wait_one(&mut self) -> EventReceiver<E> {
        EventReceiver::new(self.event.next(), self.inner.clone())
    }

    /// [`wait_one`](CompleteQ::wait_one) operation with timeout
    pub fn wait_one_timeout<T: Timer>(&mut self, timeout: Duration) -> EventReceiver<E> {
        let receiver = EventReceiver::new(self.event.next(), self.inner.clone());

        let event_id = receiver.event_id();
        let inner = self.inner.clone();

        T::interval(timeout, move || {
            inner.lock().unwrap().remove_pending_poll(event_id);
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

        let receiver = q.wait_one();

        let sender = receiver.sender();

        async_std::task::spawn(async move {
            sender.send(Default::default()).completed()?;

            Ok::<(), CompleteQError>(())
        });

        receiver.await.success()?;

        Ok(())
    }
}
