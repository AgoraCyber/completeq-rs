mod inner;
use async_timer::{Timer, TimerWithContext};
use inner::*;
mod receiver;
pub use receiver::*;
mod sender;
pub use sender::*;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::user_event::{AutoIncEvent, UserEvent};

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
    pub fn wait_for(&self, event_id: E::ID) -> EventReceiver<E, async_timer::hashed::Timeout> {
        EventReceiver::new(event_id, self.inner.clone(), None)
    }

    /// [`wait_for`](CompleteQ::wait_for) with timer to trigger waiting timeout
    pub fn wait_for_with_timer<T: Timer>(&self, event_id: E::ID, timer: T) -> EventReceiver<E, T> {
        let receiver = EventReceiver::new(event_id, self.inner.clone(), Some(timer));

        receiver
    }
    /// This function takes a timeout interval and creates a timer object internally
    ///
    /// See [`wait_for_with_timer`](CompleteQ::wait_for_with_timer) for more information
    pub fn wait_for_timeout<T: Timer>(
        &self,
        event_id: E::ID,
        duration: Duration,
    ) -> EventReceiver<E, T> {
        EventReceiver::new(event_id, self.inner.clone(), Some(T::new(duration)))
    }

    /// Compared to function [`wait_for_timeout`](CompleteQ::wait_for_timeout),
    /// this function provides a [`C`](TimerWithContext::Context) configuration
    /// parameter to timer creation
    ///
    /// See [`wait_for_with_timer`](CompleteQ::wait_for_with_timer) for more information
    pub fn wait_for_timeout_with_context<T: TimerWithContext, C>(
        &self,
        event_id: E::ID,
        duration: Duration,
        context: C,
    ) -> EventReceiver<E, T>
    where
        C: AsMut<T::Context>,
    {
        self.wait_for_with_timer(event_id, T::new_with_context(duration, context))
    }
}

impl<E: AutoIncEvent> CompleteQ<E>
where
    E: 'static,
{
    /// Create a new event receiver with automatic generate event_id
    pub fn wait_one(&mut self) -> EventReceiver<E, async_timer::hashed::Timeout> {
        let event_id = self.event.next();
        self.wait_for(event_id)
    }

    /// [`wait_one`](CompleteQ::wait_one) operation with timeout
    pub fn wait_one_timeout<T: Timer>(&mut self, timeout: Duration) -> EventReceiver<E, T> {
        let event_id = self.event.next();

        self.wait_for_timeout(event_id, timeout)
    }

    pub fn wait_one_timeout_with_context<T: TimerWithContext, C>(
        &mut self,
        timeout: Duration,
        context: C,
    ) -> EventReceiver<E, T>
    where
        C: AsMut<T::Context>,
    {
        let event_id = self.event.next();

        self.wait_for_timeout_with_context(event_id, timeout, context)
    }

    pub fn wait_one_with_timer<T: Timer>(&mut self, timer: T) -> EventReceiver<E, T> {
        let event_id = self.event.next();

        self.wait_for_with_timer(event_id, timer)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_timer::hashed::Timeout;

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

    #[async_std::test]
    async fn rec_timeout() -> anyhow::Result<()> {
        _ = pretty_env_logger::try_init();

        use std::time::SystemTime;

        let mut q = CompleteQ::<Event>::new();

        let now = SystemTime::now();

        let receiver = q.wait_one_timeout::<Timeout>(Duration::from_secs(2));

        assert!(receiver.await.is_timeout());

        assert_eq!(now.elapsed()?.as_secs(), 2);

        Ok(())
    }
}
